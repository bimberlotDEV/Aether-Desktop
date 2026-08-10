use async_trait::async_trait;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tokio_util::sync::CancellationToken;

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
pub struct ModelInfo {
    pub id: String,
    pub provider: String,
    pub supports_streaming: bool,
    pub supports_thinking: bool,
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
    pub base_url: String,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub thinking_enabled: bool,
}

impl ProviderConfig {
    pub fn default_deepseek(api_key: String) -> Self {
        Self {
            provider: "deepseek".to_string(),
            api_key,
            model: "deepseek-v4-flash".to_string(),
            base_url: "https://api.deepseek.com".to_string(),
            temperature: Some(0.2),
            max_tokens: Some(4096),
            thinking_enabled: false,
        }
    }
}

#[async_trait]
pub trait AiProvider: Send + Sync {
    fn name(&self) -> &str;
    fn available_models(&self) -> Vec<ModelInfo>;
    async fn test_connection(&self) -> Result<(), ProviderError>;
    async fn stream_chat(
        &self,
        request: &ChatCompletionRequest,
        cancellation: CancellationToken,
        on_delta: &(dyn Fn(String) -> Result<(), String> + Send + Sync),
    ) -> Result<(), ProviderError>;
}

pub struct DeepSeekProvider {
    config: ProviderConfig,
    client: reqwest::Client,
}

impl DeepSeekProvider {
    pub fn new(config: ProviderConfig) -> Result<Self, ProviderError> {
        let client = reqwest::Client::builder()
            .connect_timeout(std::time::Duration::from_secs(15))
            .timeout(std::time::Duration::from_secs(300))
            .build()
            .map_err(|_| {
                ProviderError::new("provider_setup", "Could not initialize AI networking.")
            })?;
        Ok(Self { config, client })
    }

    fn build_request(
        &self,
        request: &ChatCompletionRequest,
        stream: bool,
    ) -> ChatCompletionRequest {
        let mut request = request.clone();
        if request.model.is_empty() {
            request.model.clone_from(&self.config.model);
        }
        if request.temperature.is_none() {
            request.temperature = self.config.temperature;
        }
        if request.max_tokens.is_none() {
            request.max_tokens = self.config.max_tokens;
        }
        request.stream = Some(stream);
        if request.thinking.is_none() {
            request.thinking = Some(ThinkingConfig {
                thinking_type: if self.config.thinking_enabled {
                    "enabled"
                } else {
                    "disabled"
                }
                .to_string(),
            });
        }
        request
    }

    async fn send(
        &self,
        request: &ChatCompletionRequest,
        stream: bool,
    ) -> Result<reqwest::Response, ProviderError> {
        let response = self
            .client
            .post(format!("{}/chat/completions", self.config.base_url))
            .bearer_auth(&self.config.api_key)
            .json(&self.build_request(request, stream))
            .send()
            .await
            .map_err(classify_network_error)?;
        if !response.status().is_success() {
            return Err(classify_status(response.status()));
        }
        Ok(response)
    }
}

#[async_trait]
impl AiProvider for DeepSeekProvider {
    fn name(&self) -> &str {
        "deepseek"
    }

    fn available_models(&self) -> Vec<ModelInfo> {
        ["deepseek-v4-flash", "deepseek-v4-pro"]
            .into_iter()
            .map(|id| ModelInfo {
                id: id.to_string(),
                provider: "deepseek".to_string(),
                supports_streaming: true,
                supports_thinking: true,
            })
            .collect()
    }

    async fn test_connection(&self) -> Result<(), ProviderError> {
        let request = ChatCompletionRequest {
            model: self.config.model.clone(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: "Reply with OK.".to_string(),
            }],
            temperature: Some(0.0),
            max_tokens: Some(2),
            top_p: None,
            stream: Some(false),
            thinking: Some(ThinkingConfig {
                thinking_type: "disabled".to_string(),
            }),
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
        let response = tokio::select! {
            _ = cancellation.cancelled() => return Err(cancelled()),
            response = self.send(request, true) => response?,
        };
        let mut stream = response.bytes_stream();
        let mut decoder = SseDecoder::default();
        loop {
            let next = tokio::select! {
                _ = cancellation.cancelled() => return Err(cancelled()),
                next = stream.next() => next,
            };
            let Some(chunk) = next else { break };
            let chunk = chunk.map_err(classify_network_error)?;
            for event in decoder.push(&chunk)? {
                match event {
                    SseEvent::Delta(content) => on_delta(content).map_err(|_| cancelled())?,
                    SseEvent::Done => return Ok(()),
                }
            }
        }
        Err(ProviderError::new(
            "network",
            "The DeepSeek stream ended before completion. Try again.",
        ))
    }
}

fn cancelled() -> ProviderError {
    ProviderError::new("cancelled", "The AI response was cancelled.")
}

fn classify_network_error(error: reqwest::Error) -> ProviderError {
    if error.is_timeout() {
        ProviderError::new("timeout", "DeepSeek took too long to respond. Try again.")
    } else if error.is_connect() {
        ProviderError::new(
            "offline",
            "Could not connect to DeepSeek. Check your connection.",
        )
    } else {
        ProviderError::new("network", "The connection to DeepSeek was interrupted.")
    }
}

fn classify_status(status: reqwest::StatusCode) -> ProviderError {
    match status.as_u16() {
        401 | 403 => ProviderError::new("invalid_api_key", "DeepSeek rejected the API key."),
        402 => ProviderError::new(
            "insufficient_balance",
            "The DeepSeek account has insufficient credit.",
        ),
        429 => ProviderError::new(
            "rate_limited",
            "DeepSeek is rate limiting requests. Try again shortly.",
        ),
        500..=599 => ProviderError::new(
            "provider_unavailable",
            "DeepSeek is temporarily unavailable.",
        ),
        _ => ProviderError::new(
            "provider_error",
            format!("DeepSeek returned HTTP {}.", status.as_u16()),
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
                ProviderError::new("invalid_response", "DeepSeek returned invalid UTF-8.")
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
                            "DeepSeek returned an invalid stream event.",
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
    match config.provider.as_str() {
        "deepseek" => Ok(Box::new(DeepSeekProvider::new(config)?)),
        _ => Err(ProviderError::new(
            "unknown_provider",
            "Unknown AI provider.",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decoder_handles_split_crlf_events_and_done() {
        let mut decoder = SseDecoder::default();
        assert!(decoder
            .push(b"data: {\"choices\":[{\"delta\":{\"content\":\"Hel")
            .unwrap()
            .is_empty());
        let events = decoder
            .push(b"lo\"}}]}\r\n\r\ndata: [DONE]\r\n\r\n")
            .unwrap();
        assert_eq!(
            events,
            vec![SseEvent::Delta("Hello".to_string()), SseEvent::Done]
        );
    }

    #[test]
    fn decoder_ignores_keep_alive_and_reasoning_only_chunks() {
        let mut decoder = SseDecoder::default();
        let events = decoder.push(b": keep-alive\n\ndata: {\"choices\":[{\"delta\":{\"reasoning_content\":\"private\"}}]}\n\n").unwrap();
        assert!(events.is_empty());
    }

    #[test]
    fn decoder_preserves_utf8_split_between_network_chunks() {
        let payload = "data: {\"choices\":[{\"delta\":{\"content\":\"hé 👋\"}}]}\n\n".as_bytes();
        let split = payload.iter().position(|byte| *byte >= 0x80).unwrap() + 1;
        let mut decoder = SseDecoder::default();
        assert!(decoder.push(&payload[..split]).unwrap().is_empty());
        assert_eq!(
            decoder.push(&payload[split..]).unwrap(),
            vec![SseEvent::Delta("hé 👋".to_string())]
        );
    }

    #[test]
    fn current_models_replace_retired_aliases() {
        let provider =
            DeepSeekProvider::new(ProviderConfig::default_deepseek("secret".to_string())).unwrap();
        let ids: Vec<_> = provider
            .available_models()
            .into_iter()
            .map(|model| model.id)
            .collect();
        assert_eq!(ids, vec!["deepseek-v4-flash", "deepseek-v4-pro"]);
    }
}
