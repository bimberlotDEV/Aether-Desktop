use std::collections::HashMap;
use std::sync::Mutex;
use tokio_util::sync::CancellationToken;

#[derive(Default)]
pub struct AiRuntime {
    requests: Mutex<HashMap<String, CancellationToken>>,
}

impl AiRuntime {
    pub fn start(&self, request_id: &str) -> Result<CancellationToken, String> {
        let request_id = request_id.trim();
        if request_id.is_empty() || request_id.len() > 100 {
            return Err("Invalid AI request ID.".to_string());
        }
        let mut requests = self
            .requests
            .lock()
            .map_err(|_| "AI request registry is unavailable.".to_string())?;
        if requests.contains_key(request_id) {
            return Err("This AI request is already running.".to_string());
        }
        let token = CancellationToken::new();
        requests.insert(request_id.to_string(), token.clone());
        Ok(token)
    }

    pub fn cancel(&self, request_id: &str) -> Result<bool, String> {
        let requests = self
            .requests
            .lock()
            .map_err(|_| "AI request registry is unavailable.".to_string())?;
        let Some(token) = requests.get(request_id) else {
            return Ok(false);
        };
        token.cancel();
        Ok(true)
    }

    pub fn finish(&self, request_id: &str) {
        if let Ok(mut requests) = self.requests.lock() {
            requests.remove(request_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_ids_are_unique_and_cancellable() {
        let runtime = AiRuntime::default();
        let token = runtime.start("request-1").unwrap();
        assert!(runtime.start("request-1").is_err());
        assert!(runtime.cancel("request-1").unwrap());
        assert!(token.is_cancelled());
        runtime.finish("request-1");
        assert!(!runtime.cancel("request-1").unwrap());
    }
}
