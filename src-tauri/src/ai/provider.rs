use async_trait::async_trait;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tokio_util::sync::CancellationToken;

const DEEPSEEK_ENDPOINT: &str = "https://api.deepseek.com/chat/completions";
const OPENAI_ENDPOINT: &str = "https://api.openai.com/v1/chat/completions";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<ThinkingConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThinkingConfig {
    #[serde(rename = "type")]
    pub thinking_type: String,
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
}

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
        request: &ChatCompletionRequest,
        cancellation: CancellationToken,
        on_delta: &(dyn Fn(String) -> Result<(), String> + Send + Sync),
    ) -> Result<(), ProviderError>;
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
    fn build_body(&self, request: &ChatCompletionRequest, stream: bool) -> serde_json::Value {
        let model = if request.model.is_empty() {
            &self.config.model
        } else {
            &request.model
        };
        let mut body =
            serde_json::json!({ "model": model, "messages": request.messages, "stream": stream });
        let object = body.as_object_mut().expect("request body is an object");
        if let Some(top_p) = request.top_p {
            object.insert("top_p".into(), serde_json::json!(top_p));
        }
        let max_tokens = request.max_tokens.or(self.config.max_tokens);
        match self.protocol {
            Protocol::DeepSeek => {
                if let Some(temperature) = request.temperature.or(self.config.temperature) {
                    object.insert("temperature".into(), serde_json::json!(temperature));
                }
                if let Some(limit) = max_tokens {
                    object.insert("max_tokens".into(), serde_json::json!(limit));
                }
                let thinking = request
                    .thinking
                    .as_ref()
                    .map(|value| value.thinking_type.as_str())
                    .unwrap_or(if self.config.thinking_enabled {
                        "enabled"
                    } else {
                        "disabled"
                    });
                object.insert("thinking".into(), serde_json::json!({ "type": thinking }));
            }
            Protocol::OpenAi => {
                if let Some(limit) = max_tokens {
                    object.insert("max_completion_tokens".into(), serde_json::json!(limit));
                }
            }
        }
        body
    }
    async fn send(
        &self,
        request: &ChatCompletionRequest,
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

#[async_trait]
impl AiProvider for ChatCompletionsProvider {
    fn name(&self) -> &str {
        &self.config.provider
    }
    async fn test_connection(&self) -> Result<(), ProviderError> {
        let request = ChatCompletionRequest {
            model: self.config.model.clone(),
            messages: vec![ChatMessage {
                role: "user".into(),
                content: "Reply with OK.".into(),
            }],
            temperature: None,
            max_tokens: Some(8),
            top_p: None,
            stream: Some(false),
            thinking: None,
        };
        self.send(&request, false).await?;
        Ok(())
    }
    async fn stream_chat(
        &self,
        request: &ChatCompletionRequest,
        cancellation: CancellationToken,
        on_delta: &(dyn Fn(String) -> Result<(), String> + Send + Sync),
    ) -> Result<(), ProviderError> {
        let response = tokio::select! { _ = cancellation.cancelled() => return Err(cancelled()), response = self.send(request, true) => response?, };
        let mut stream = response.bytes_stream();
        let mut decoder = SseDecoder::default();
        loop {
            let next = tokio::select! { _ = cancellation.cancelled() => return Err(cancelled()), next = stream.next() => next, };
            let Some(chunk) = next else { break };
            let chunk = chunk.map_err(|error| classify_network_error(error, self.name()))?;
            for event in decoder.push(&chunk)? {
                match event {
                    SseEvent::Delta(content) => on_delta(content).map_err(|_| cancelled())?,
                    SseEvent::Done => return Ok(()),
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

fn provider_label(provider: &str) -> &'static str {
    match provider {
        "openai" => "OpenAI",
        _ => "DeepSeek",
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
                events.extend(response.choices.into_iter().filter_map(|choice| {
                    choice
                        .delta
                        .and_then(|delta| delta.content)
                        .filter(|content| !content.is_empty())
                        .map(SseEvent::Delta)
                }));
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
    fn request() -> ChatCompletionRequest {
        ChatCompletionRequest {
            model: String::new(),
            messages: vec![ChatMessage {
                role: "user".into(),
                content: "Hi".into(),
            }],
            temperature: None,
            max_tokens: None,
            top_p: None,
            stream: None,
            thinking: None,
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
    fn provider_protocols_shape_requests_without_cross_leaking_fields() {
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
    }
    #[test]
    fn decoder_handles_split_crlf_events_and_done() {
        let mut decoder = SseDecoder::default();
        assert!(decoder
            .push(b"data: {\"choices\":[{\"delta\":{\"content\":\"Hel")
            .unwrap()
            .is_empty());
        assert_eq!(
            decoder
                .push(b"lo\"}}]}\r\n\r\ndata: [DONE]\r\n\r\n")
                .unwrap(),
            vec![SseEvent::Delta("Hello".into()), SseEvent::Done]
        );
    }
    #[test]
    fn decoder_ignores_keep_alive_and_reasoning_only_chunks() {
        let mut decoder = SseDecoder::default();
        assert!(decoder.push(b": keep-alive\n\ndata: {\"choices\":[{\"delta\":{\"reasoning_content\":\"private\"}}]}\n\n").unwrap().is_empty());
    }
    #[test]
    fn decoder_preserves_utf8_split_between_network_chunks() {
        let payload = "data: {\"choices\":[{\"delta\":{\"content\":\"hé 👋\"}}]}\n\n".as_bytes();
        let split = payload.iter().position(|byte| *byte >= 0x80).unwrap() + 1;
        let mut decoder = SseDecoder::default();
        assert!(decoder.push(&payload[..split]).unwrap().is_empty());
        assert_eq!(
            decoder.push(&payload[split..]).unwrap(),
            vec![SseEvent::Delta("hé 👋".into())]
        );
    }
}
