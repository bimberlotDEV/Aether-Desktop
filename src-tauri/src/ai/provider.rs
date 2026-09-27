use async_trait::async_trait;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio_util::sync::CancellationToken;

use super::model::{ModelMessage, ModelToolRequest, ModelTurnOutcome, ModelTurnRequest};

const DEEPSEEK_ENDPOINT: &str = "https://api.deepseek.com/chat/completions";
const OPENAI_ENDPOINT: &str = "https://api.openai.com/v1/chat/completions";

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProviderInfo {
    pub id: String,
    pub display_name: String,
    pub remote: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ModelInfo {
    pub id: String,
    pub display_name: String,
    pub provider: String,
    pub supports_streaming: bool,
    pub supports_thinking: bool,
    pub supports_structured_output: bool,
}

pub fn provider_catalog() -> Vec<ProviderInfo> {
    vec![
        ProviderInfo {
            id: "deepseek".into(),
            display_name: "DeepSeek".into(),
            remote: true,
        },
        ProviderInfo {
            id: "openai".into(),
            display_name: "OpenAI".into(),
            remote: true,
        },
    ]
}

pub fn model_catalog() -> Vec<ModelInfo> {
    [
        ("deepseek-v4-flash", "DeepSeek V4 Flash", "deepseek", true),
        ("deepseek-v4-pro", "DeepSeek V4 Pro", "deepseek", true),
        ("gpt-5-mini", "GPT-5 mini", "openai", false),
        ("gpt-5.2", "GPT-5.2", "openai", false),
    ]
    .into_iter()
    .map(
        |(id, display_name, provider, supports_thinking)| ModelInfo {
            id: id.into(),
            display_name: display_name.into(),
            provider: provider.into(),
            supports_streaming: true,
            supports_thinking,
            supports_structured_output: true,
        },
    )
    .collect()
}

pub fn validate_provider_model(provider: &str, model: &str) -> Result<(), ProviderError> {
    if model_catalog()
        .iter()
        .any(|item| item.provider == provider && item.id == model)
    {
        Ok(())
    } else {
        Err(ProviderError::new(
            "unsupported_route",
            "This AI provider and model combination is not supported.",
        ))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProviderError {
    pub code: &'static str,
    pub message: String,
}

impl ProviderError {
    fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    pub(crate) fn public(code: &'static str, message: impl Into<String>) -> Self {
        Self::new(code, message)
    }
}

#[derive(Debug, Clone)]
pub struct ProviderConfig {
    pub provider: String,
    pub api_key: String,
    pub model: String,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub thinking_enabled: bool,
}

impl ProviderConfig {
    pub fn for_route(provider: &str, api_key: String, model: &str) -> Result<Self, ProviderError> {
        validate_provider_model(provider, model)?;
        Ok(Self {
            provider: provider.into(),
            api_key,
            model: model.into(),
            temperature: Some(0.2),
            max_tokens: Some(4096),
            thinking_enabled: false,
        })
    }
}

#[async_trait]
pub trait AiProvider: Send + Sync {
    fn name(&self) -> &str;
    async fn test_connection(&self) -> Result<(), ProviderError>;
    async fn stream_chat(
        &self,
        request: &ModelTurnRequest,
        cancellation: CancellationToken,
        on_delta: &(dyn Fn(String) -> Result<(), String> + Send + Sync),
    ) -> Result<ModelTurnOutcome, ProviderError>;
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Protocol {
    DeepSeek,
    OpenAi,
}

pub struct ChatCompletionsProvider {
    config: ProviderConfig,
    protocol: Protocol,
    client: reqwest::Client,
}

impl ChatCompletionsProvider {
    fn new(config: ProviderConfig, protocol: Protocol) -> Result<Self, ProviderError> {
        let client = reqwest::Client::builder()
            .connect_timeout(std::time::Duration::from_secs(15))
            .timeout(std::time::Duration::from_secs(300))
            .build()
            .map_err(|_| {
                ProviderError::new("provider_setup", "Could not initialize AI networking.")
            })?;
        Ok(Self {
            config,
            protocol,
            client,
        })
    }

    fn endpoint(&self) -> &'static str {
        match self.protocol {
            Protocol::DeepSeek => DEEPSEEK_ENDPOINT,
            Protocol::OpenAi => OPENAI_ENDPOINT,
        }
    }

    fn build_body(&self, request: &ModelTurnRequest, stream: bool) -> Value {
        let model = if request.model.is_empty() {
            &self.config.model
        } else {
            &request.model
        };
        let messages = request
            .messages
            .iter()
            .map(wire_message)
            .collect::<Vec<_>>();
        let mut body = json!({ "model": model, "messages": messages, "stream": stream });
        let object = body.as_object_mut().expect("request body is an object");
        if !request.tools.is_empty() {
            object.insert(
                "tools".into(),
                Value::Array(
                    request
                        .tools
                        .iter()
                        .map(|tool| {
                            json!({
                                "type": "function",
                                "function": {
                                    "name": wire_tool_name(&tool.name),
                                    "description": tool.description,
                                    "parameters": tool.input_schema
                                }
                            })
                        })
                        .collect(),
                ),
            );
            object.insert("tool_choice".into(), json!("auto"));
            object.insert("parallel_tool_calls".into(), json!(true));
        }
        if let Some(top_p) = request.top_p {
            object.insert("top_p".into(), json!(top_p));
        }
        let max_tokens = request.max_tokens.or(self.config.max_tokens);
        match self.protocol {
            Protocol::DeepSeek => {
                if let Some(temperature) = request.temperature.or(self.config.temperature) {
                    object.insert("temperature".into(), json!(temperature));
                }
                if let Some(limit) = max_tokens {
                    object.insert("max_tokens".into(), json!(limit));
                }
                object.insert("thinking".into(), json!({ "type": if request.thinking_enabled || self.config.thinking_enabled { "enabled" } else { "disabled" } }));
            }
            Protocol::OpenAi => {
                if let Some(limit) = max_tokens {
                    object.insert("max_completion_tokens".into(), json!(limit));
                }
            }
        }
        body
    }

    async fn send(
        &self,
        request: &ModelTurnRequest,
        stream: bool,
    ) -> Result<reqwest::Response, ProviderError> {
        let response = self
            .client
            .post(self.endpoint())
            .bearer_auth(&self.config.api_key)
            .json(&self.build_body(request, stream))
            .send()
            .await
            .map_err(|error| classify_network_error(error, self.name()))?;
        if !response.status().is_success() {
            return Err(classify_status(response.status(), self.name()));
        }
        Ok(response)
    }
}

fn wire_message(message: &ModelMessage) -> Value {
    match message {
        ModelMessage::System(content) => json!({ "role": "system", "content": content }),
        ModelMessage::User(content) => json!({ "role": "user", "content": content }),
        ModelMessage::Assistant(content) => json!({ "role": "assistant", "content": content }),
        ModelMessage::AssistantToolRequests { content, requests } => json!({
            "role": "assistant",
            "content": content,
            "tool_calls": requests.iter().map(|request| json!({
                "id": request.request_id,
                "type": "function",
                "function": { "name": wire_tool_name(&request.tool_name), "arguments": request.arguments_json }
            })).collect::<Vec<_>>()
        }),
        ModelMessage::ToolResult(result) => json!({
            "role": "tool",
            "tool_call_id": result.request_id,
            "content": serde_json::to_string(&json!({ "status": result.status, "output": result.output }))
                .unwrap_or_else(|_| "{\"status\":\"error\"}".into())
        }),
    }
}

#[async_trait]
impl AiProvider for ChatCompletionsProvider {
    fn name(&self) -> &str {
        &self.config.provider
    }

    async fn test_connection(&self) -> Result<(), ProviderError> {
        let request = ModelTurnRequest {
            model: self.config.model.clone(),
            messages: vec![ModelMessage::User("Reply with OK.".into())],
            tools: vec![],
            temperature: None,
            max_tokens: Some(8),
            top_p: None,
            thinking_enabled: false,
        };
        self.send(&request, false).await?;
        Ok(())
    }

    async fn stream_chat(
        &self,
        request: &ModelTurnRequest,
        cancellation: CancellationToken,
        on_delta: &(dyn Fn(String) -> Result<(), String> + Send + Sync),
    ) -> Result<ModelTurnOutcome, ProviderError> {
        let response = tokio::select! {
            _ = cancellation.cancelled() => return Err(cancelled()),
            response = self.send(request, true) => response?,
        };
        let mut stream = response.bytes_stream();
        let mut decoder = SseDecoder::default();
        let mut tool_calls: Vec<PartialToolCall> = Vec::new();
        loop {
            let next = tokio::select! {
                _ = cancellation.cancelled() => return Err(cancelled()),
                next = stream.next() => next,
            };
            let Some(chunk) = next else { break };
            let chunk = chunk.map_err(|error| classify_network_error(error, self.name()))?;
            for event in decoder.push(&chunk)? {
                match event {
                    SseEvent::Delta(content) => on_delta(content).map_err(|_| cancelled())?,
                    SseEvent::ToolDelta(delta) => merge_tool_delta(&mut tool_calls, delta)?,
                    SseEvent::Done => return finish_tool_calls(tool_calls),
                }
            }
        }
        Err(ProviderError::new(
            "network",
            format!(
                "The {} stream ended before completion. Try again.",
                provider_label(self.name())
            ),
        ))
    }
}

#[derive(Debug, Clone, Deserialize)]
struct ChatCompletionResponse {
    #[serde(default)]
    choices: Vec<Choice>,
}
#[derive(Debug, Clone, Deserialize)]
struct Choice {
    #[serde(default)]
    delta: Option<ChoiceDelta>,
}
#[derive(Debug, Clone, Deserialize)]
struct ChoiceDelta {
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    tool_calls: Vec<WireToolCallDelta>,
}
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
struct WireToolCallDelta {
    index: usize,
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    function: Option<WireFunctionDelta>,
}
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
struct WireFunctionDelta {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    arguments: Option<String>,
}

#[derive(Debug, Default)]
struct PartialToolCall {
    id: String,
    name: String,
    arguments: String,
}

fn merge_tool_delta(
    calls: &mut Vec<PartialToolCall>,
    delta: WireToolCallDelta,
) -> Result<(), ProviderError> {
    if delta.index >= 32 {
        return Err(invalid_tool_call());
    }
    while calls.len() <= delta.index {
        calls.push(PartialToolCall::default());
    }
    let call = &mut calls[delta.index];
    if let Some(id) = delta.id {
        call.id.push_str(&id);
    }
    if let Some(function) = delta.function {
        if let Some(name) = function.name {
            call.name.push_str(&name);
        }
        if let Some(arguments) = function.arguments {
            call.arguments.push_str(&arguments);
        }
    }
    Ok(())
}

fn finish_tool_calls(calls: Vec<PartialToolCall>) -> Result<ModelTurnOutcome, ProviderError> {
    let mut requests = Vec::with_capacity(calls.len());
    for call in calls {
        if call.id.is_empty()
            || call.name.is_empty()
            || serde_json::from_str::<Value>(&call.arguments).is_err()
        {
            return Err(invalid_tool_call());
        }
        requests.push(ModelToolRequest {
            request_id: call.id,
            tool_name: normalized_tool_name(&call.name),
            arguments_json: call.arguments,
        });
    }
    Ok(ModelTurnOutcome {
        tool_requests: requests,
    })
}

fn invalid_tool_call() -> ProviderError {
    ProviderError::new(
        "invalid_tool_call",
        "The AI provider returned a malformed tool request.",
    )
}

fn wire_tool_name(name: &str) -> String {
    name.replace('.', "__")
}

fn normalized_tool_name(name: &str) -> String {
    name.replace("__", ".")
}

fn provider_label(provider: &str) -> &'static str {
    if provider == "openai" {
        "OpenAI"
    } else {
        "DeepSeek"
    }
}
fn cancelled() -> ProviderError {
    ProviderError::new("cancelled", "The AI response was cancelled.")
}
fn classify_network_error(error: reqwest::Error, provider: &str) -> ProviderError {
    let label = provider_label(provider);
    if error.is_timeout() {
        ProviderError::new(
            "timeout",
            format!("{label} took too long to respond. Try again."),
        )
    } else if error.is_connect() {
        ProviderError::new(
            "offline",
            format!("Could not connect to {label}. Check your connection."),
        )
    } else {
        ProviderError::new(
            "network",
            format!("The connection to {label} was interrupted."),
        )
    }
}
fn classify_status(status: reqwest::StatusCode, provider: &str) -> ProviderError {
    let label = provider_label(provider);
    match status.as_u16() {
        401 | 403 => {
            ProviderError::new("invalid_api_key", format!("{label} rejected the API key."))
        }
        402 => ProviderError::new(
            "insufficient_balance",
            format!("The {label} account has insufficient credit."),
        ),
        429 => ProviderError::new(
            "rate_limited",
            format!("{label} is rate limiting requests. Try again shortly."),
        ),
        500..=599 => ProviderError::new(
            "provider_unavailable",
            format!("{label} is temporarily unavailable."),
        ),
        _ => ProviderError::new(
            "provider_error",
            format!("{label} returned HTTP {}.", status.as_u16()),
        ),
    }
}

#[derive(Debug, PartialEq)]
enum SseEvent {
    Delta(String),
    ToolDelta(WireToolCallDelta),
    Done,
}
#[derive(Default)]
struct SseDecoder {
    buffer: Vec<u8>,
}
impl SseDecoder {
    fn push(&mut self, bytes: &[u8]) -> Result<Vec<SseEvent>, ProviderError> {
        self.buffer.extend_from_slice(bytes);
        let mut events = Vec::new();
        while let Some((end, delimiter_len)) = event_boundary(&self.buffer) {
            let block = String::from_utf8(self.buffer[..end].to_vec()).map_err(|_| {
                ProviderError::new(
                    "invalid_response",
                    "The AI provider returned invalid UTF-8.",
                )
            })?;
            self.buffer.drain(..end + delimiter_len);
            for line in block.lines().map(str::trim) {
                let Some(data) = line.strip_prefix("data:").map(str::trim) else {
                    continue;
                };
                if data == "[DONE]" {
                    events.push(SseEvent::Done);
                    continue;
                }
                let response: ChatCompletionResponse =
                    serde_json::from_str(data).map_err(|_| {
                        ProviderError::new(
                            "invalid_response",
                            "The AI provider returned an invalid stream event.",
                        )
                    })?;
                for delta in response
                    .choices
                    .into_iter()
                    .filter_map(|choice| choice.delta)
                {
                    if let Some(content) = delta.content.filter(|value| !value.is_empty()) {
                        events.push(SseEvent::Delta(content));
                    }
                    events.extend(delta.tool_calls.into_iter().map(SseEvent::ToolDelta));
                }
            }
        }
        Ok(events)
    }
}

fn event_boundary(bytes: &[u8]) -> Option<(usize, usize)> {
    let lf = bytes
        .windows(2)
        .position(|window| window == b"\n\n")
        .map(|pos| (pos, 2));
    let crlf = bytes
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map(|pos| (pos, 4));
    match (lf, crlf) {
        (Some(left), Some(right)) => Some(if left.0 <= right.0 { left } else { right }),
        (left, right) => left.or(right),
    }
}

pub fn create_provider(config: ProviderConfig) -> Result<Box<dyn AiProvider>, ProviderError> {
    validate_provider_model(&config.provider, &config.model)?;
    let protocol = match config.provider.as_str() {
        "deepseek" => Protocol::DeepSeek,
        "openai" => Protocol::OpenAi,
        _ => {
            return Err(ProviderError::new(
                "unknown_provider",
                "Unknown AI provider.",
            ))
        }
    };
    Ok(Box::new(ChatCompletionsProvider::new(config, protocol)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::{
        model::{ModelToolDescriptor, ModelToolResult, ModelToolResultStatus},
        tools::NativeToolId,
    };

    fn request() -> ModelTurnRequest {
        ModelTurnRequest {
            model: String::new(),
            messages: vec![ModelMessage::User("Hi".into())],
            tools: vec![],
            temperature: None,
            max_tokens: None,
            top_p: None,
            thinking_enabled: false,
        }
    }

    #[test]
    fn registry_is_closed_and_models_belong_to_known_providers() {
        assert_eq!(
            provider_catalog()
                .iter()
                .map(|item| item.id.as_str())
                .collect::<Vec<_>>(),
            vec!["deepseek", "openai"]
        );
        assert!(model_catalog().iter().all(|model| provider_catalog()
            .iter()
            .any(|provider| provider.id == model.provider)));
        assert!(ProviderConfig::for_route("other", "secret".into(), "model").is_err());
        assert!(ProviderConfig::for_route("openai", "secret".into(), "deepseek-v4-pro").is_err());
    }

    #[test]
    fn both_protocols_serialize_tools_and_native_continuations() {
        let mut value = request();
        value.tools.push(ModelToolDescriptor {
            tool_id: NativeToolId::TasksGetOpen,
            name: "tasks.get_open".into(),
            description: "Read tasks".into(),
            input_schema: json!({"type":"object"}),
        });
        let tool_request = ModelToolRequest {
            request_id: "call-1".into(),
            tool_name: "tasks.get_open".into(),
            arguments_json: "{}".into(),
        };
        value.messages.push(ModelMessage::AssistantToolRequests {
            content: None,
            requests: vec![tool_request],
        });
        value
            .messages
            .push(ModelMessage::ToolResult(ModelToolResult {
                request_id: "call-1".into(),
                tool_name: "tasks.get_open".into(),
                tool_id: Some(NativeToolId::TasksGetOpen),
                status: ModelToolResultStatus::Ok,
                output: json!({"tasks":[]}),
            }));
        for (provider, model, protocol) in [
            ("deepseek", "deepseek-v4-flash", Protocol::DeepSeek),
            ("openai", "gpt-5-mini", Protocol::OpenAi),
        ] {
            let adapter = ChatCompletionsProvider::new(
                ProviderConfig::for_route(provider, "secret".into(), model).unwrap(),
                protocol,
            )
            .unwrap();
            let body = adapter.build_body(&value, true);
            assert_eq!(body["tools"][0]["function"]["name"], "tasks__get_open");
            assert_eq!(body["messages"][1]["tool_calls"][0]["id"], "call-1");
            assert_eq!(body["messages"][2]["tool_call_id"], "call-1");
        }
    }

    #[test]
    fn provider_protocols_preserve_non_tool_request_shaping() {
        let deepseek = ChatCompletionsProvider::new(
            ProviderConfig::for_route("deepseek", "secret".into(), "deepseek-v4-flash").unwrap(),
            Protocol::DeepSeek,
        )
        .unwrap();
        let deepseek_body = deepseek.build_body(&request(), true);
        assert_eq!(deepseek.endpoint(), DEEPSEEK_ENDPOINT);
        assert!(deepseek_body.get("thinking").is_some());
        assert!(deepseek_body.get("max_tokens").is_some());
        assert!(deepseek_body.get("max_completion_tokens").is_none());
        assert!(deepseek_body.get("tools").is_none());

        let openai = ChatCompletionsProvider::new(
            ProviderConfig::for_route("openai", "secret".into(), "gpt-5-mini").unwrap(),
            Protocol::OpenAi,
        )
        .unwrap();
        let openai_body = openai.build_body(&request(), true);
        assert_eq!(openai.endpoint(), OPENAI_ENDPOINT);
        assert!(openai_body.get("thinking").is_none());
        assert!(openai_body.get("temperature").is_none());
        assert!(openai_body.get("max_completion_tokens").is_some());
        assert!(openai_body.get("tools").is_none());
    }

    #[test]
    fn decoder_normalizes_parallel_tool_requests_in_stable_order() {
        let mut decoder = SseDecoder::default();
        let events = decoder.push(b"data: {\"choices\":[{\"delta\":{\"tool_calls\":[{\"index\":0,\"id\":\"a\",\"function\":{\"name\":\"tasks__get_open\",\"arguments\":\"{\"}},{\"index\":1,\"id\":\"b\",\"function\":{\"name\":\"calendar__get_next_event\",\"arguments\":\"{}\"}}]}}]}\n\ndata: {\"choices\":[{\"delta\":{\"tool_calls\":[{\"index\":0,\"function\":{\"arguments\":\"}\"}}]}}]}\n\ndata: [DONE]\n\n").unwrap();
        let mut calls = Vec::new();
        for event in events {
            if let SseEvent::ToolDelta(delta) = event {
                merge_tool_delta(&mut calls, delta).unwrap();
            }
        }
        let outcome = finish_tool_calls(calls).unwrap();
        assert_eq!(
            outcome
                .tool_requests
                .iter()
                .map(|call| call.request_id.as_str())
                .collect::<Vec<_>>(),
            vec!["a", "b"]
        );
        assert_eq!(outcome.tool_requests[0].arguments_json, "{}");
        assert_eq!(outcome.tool_requests[0].tool_name, "tasks.get_open");
    }

    #[test]
    fn malformed_tool_call_fails_closed() {
        assert_eq!(
            finish_tool_calls(vec![PartialToolCall {
                id: "a".into(),
                name: "tasks.get_open".into(),
                arguments: "{".into()
            }])
            .unwrap_err()
            .code,
            "invalid_tool_call"
        );
    }

    #[test]
    fn decoder_handles_text_utf8_and_done() {
        let payload =
            "data: {\"choices\":[{\"delta\":{\"content\":\"hé 👋\"}}]}\r\n\r\ndata: [DONE]\r\n\r\n"
                .as_bytes();
        let split = payload.iter().position(|byte| *byte >= 0x80).unwrap() + 1;
        let mut decoder = SseDecoder::default();
        assert!(decoder.push(&payload[..split]).unwrap().is_empty());
        assert_eq!(
            decoder.push(&payload[split..]).unwrap(),
            vec![SseEvent::Delta("hé 👋".into()), SseEvent::Done]
        );
    }

    #[test]
    fn decoder_ignores_keep_alive_and_reasoning_only_chunks() {
        let mut decoder = SseDecoder::default();
        assert!(decoder.push(b": keep-alive\n\ndata: {\"choices\":[{\"delta\":{\"reasoning_content\":\"private\"}}]}\n\n").unwrap().is_empty());
    }
}
