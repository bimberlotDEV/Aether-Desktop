use serde::{Deserialize, Serialize};
use std::io::Read;

// Provider-neutral types

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionResponse {
    pub id: Option<String>,
    pub model: Option<String>,
    pub choices: Vec<Choice>,
    pub usage: Option<Usage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Choice {
    pub index: u32,
    pub message: Option<ChoiceMessage>,
    pub delta: Option<ChoiceDelta>,
    #[serde(rename = "finish_reason")]
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChoiceMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChoiceDelta {
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub role: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: Option<u32>,
    pub completion_tokens: Option<u32>,
    pub total_tokens: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub provider: String,
    pub supports_streaming: bool,
    pub supports_thinking: bool,
}

// Provider trait

pub trait AiProvider: Send + Sync {
    fn name(&self) -> &str;
    fn chat_completion(&self, request: &ChatCompletionRequest) -> Result<ChatCompletionResponse, String>;
    fn chat_completion_stream(&self, request: &ChatCompletionRequest) -> Result<Box<dyn Iterator<Item = Result<String, String>> + '_>, String>;
    fn test_connection(&self) -> Result<(), String>;
    fn available_models(&self) -> Vec<ModelInfo>;
}

// Provider configuration

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub provider: String,
    pub api_key: String,
    pub model: String,
    pub base_url: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub top_p: Option<f32>,
    pub thinking_enabled: Option<bool>,
}

impl ProviderConfig {
    pub fn default_deepseek(api_key: String) -> Self {
        Self {
            provider: "deepseek".to_string(),
            api_key,
            model: "deepseek-chat".to_string(),
            base_url: Some("https://api.deepseek.com/v1".to_string()),
            temperature: Some(0.1),
            max_tokens: Some(4096),
            top_p: Some(1.0),
            thinking_enabled: Some(true),
        }
    }
}

// DeepSeek Provider

pub struct DeepSeekProvider {
    config: ProviderConfig,
}

impl DeepSeekProvider {
    pub fn new(config: ProviderConfig) -> Self {
        Self { config }
    }

    fn build_chat_request(&self, request: &ChatCompletionRequest) -> ChatCompletionRequest {
        let mut req = request.clone();
        if req.model.is_empty() {
            req.model = self.config.model.clone();
        }
        if req.temperature.is_none() {
            req.temperature = self.config.temperature;
        }
        if req.max_tokens.is_none() {
            req.max_tokens = self.config.max_tokens;
        }
        if req.top_p.is_none() {
            req.top_p = self.config.top_p;
        }
        req
    }
}

impl AiProvider for DeepSeekProvider {
    fn name(&self) -> &str {
        "deepseek"
    }

    fn chat_completion(&self, request: &ChatCompletionRequest) -> Result<ChatCompletionResponse, String> {
        let req = self.build_chat_request(request);
        let body = serde_json::to_string(&req)
            .map_err(|e| format!("Serialization error: {}", e))?;

        let url = format!(
            "{}/chat/completions",
            self.config.base_url.as_deref().unwrap_or("https://api.deepseek.com/v1")
        );

        let response = ureq::post(&url)
            .set("Content-Type", "application/json")
            .set("Authorization", &format!("Bearer {}", self.config.api_key))
            .send_string(&body)
            .map_err(|e| format!("HTTP error: {}", e))?;

        let status = response.status();
        let resp_body = response.into_string()
            .map_err(|e| format!("Read error: {}", e))?;

        if status != 200 {
            return Err(format!("API error {}: {}", status, resp_body));
        }

        serde_json::from_str(&resp_body)
            .map_err(|e| format!("Parse error: {}", e))
    }

    fn chat_completion_stream(&self, request: &ChatCompletionRequest) -> Result<Box<dyn Iterator<Item = Result<String, String>> + '_>, String> {
        let mut req = self.build_chat_request(request);
        req.stream = Some(true);

        let body = serde_json::to_string(&req)
            .map_err(|e| format!("Serialization error: {}", e))?;

        let url = format!(
            "{}/chat/completions",
            self.config.base_url.as_deref().unwrap_or("https://api.deepseek.com/v1")
        );

        let response = ureq::post(&url)
            .set("Content-Type", "application/json")
            .set("Authorization", &format!("Bearer {}", self.config.api_key))
            .send_string(&body)
            .map_err(|e| format!("HTTP error: {}", e))?;

        let status = response.status();
        if status != 200 {
            let resp_body = response.into_string().unwrap_or_default();
            return Err(format!("API error {}: {}", status, resp_body));
        }

        let reader = response.into_reader();
        Ok(Box::new(SseIterator { reader, buffer: String::new() }))
    }

    fn test_connection(&self) -> Result<(), String> {
        let request = ChatCompletionRequest {
            model: self.config.model.clone(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: "Hi".to_string(),
            }],
            temperature: None,
            max_tokens: Some(1),
            top_p: None,
            stream: None,
            thinking: None,
        };
        self.chat_completion(&request)?;
        Ok(())
    }

    fn available_models(&self) -> Vec<ModelInfo> {
        vec![
            ModelInfo {
                id: "deepseek-chat".to_string(),
                provider: "deepseek".to_string(),
                supports_streaming: true,
                supports_thinking: true,
            },
            ModelInfo {
                id: "deepseek-reasoner".to_string(),
                provider: "deepseek".to_string(),
                supports_streaming: true,
                supports_thinking: true,
            },
        ]
    }
}

// SSE Iterator

struct SseIterator {
    reader: Box<dyn Read + 'static>,
    buffer: String,
}

impl Iterator for SseIterator {
    type Item = Result<String, String>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut chunk = [0u8; 4096];
        loop {
            match self.reader.read(&mut chunk) {
                Ok(0) => {
                    if self.buffer.contains("[DONE]") {
                        return None;
                    }
                    if self.buffer.is_empty() {
                        return None;
                    }
                    return self.process_buffer();
                }
                Ok(n) => {
                    self.buffer.push_str(&String::from_utf8_lossy(&chunk[..n]));
                    if let Some(result) = self.process_buffer() {
                        return Some(result);
                    }
                }
                Err(e) => return Some(Err(format!("Read error: {}", e))),
            }
        }
    }
}

impl SseIterator {
    fn process_buffer(&mut self) -> Option<Result<String, String>> {
        let separator = "\n\n";
        if let Some(pos) = self.buffer.find(separator) {
            let message = self.buffer[..pos].to_string();
            self.buffer = self.buffer[pos + 2..].to_string();

            for line in message.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with(':') {
                    continue;
                }
                if line == "data: [DONE]" {
                    return None;
                }
                if let Some(data) = line.strip_prefix("data: ") {
                    match serde_json::from_str::<ChatCompletionResponse>(data) {
                        Ok(response) => {
                            for choice in &response.choices {
                                if let Some(ref delta) = choice.delta {
                                    if let Some(ref content) = delta.content {
                                        return Some(Ok(content.clone()));
                                    }
                                }
                            }
                            return Some(Ok(String::new()));
                        }
                        Err(e) => {
                            log::warn!("SSE parse error: {}", e);
                            continue;
                        }
                    }
                }
            }
            Some(Ok(String::new()))
        } else {
            None
        }
    }
}

// Factory

pub fn create_provider(config: ProviderConfig) -> Result<Box<dyn AiProvider>, String> {
    match config.provider.as_str() {
        "deepseek" => Ok(Box::new(DeepSeekProvider::new(config))),
        other => Err(format!("Unknown provider: {}", other)),
    }
}
