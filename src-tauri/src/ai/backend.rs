use async_trait::async_trait;
use tokio_util::sync::CancellationToken;

use super::{
    capabilities::{cloud_backend_descriptors, BackendDescriptor, ModelDescriptor},
    provider::{self, ChatCompletionRequest, ProviderConfig, ProviderError},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendHealth {
    Available,
}

#[async_trait]
pub trait ModelBackend: Send + Sync {
    fn descriptor(&self) -> &BackendDescriptor;
    async fn health(&self) -> Result<BackendHealth, ProviderError>;
    async fn discover_models(&self) -> Result<Vec<ModelDescriptor>, ProviderError>;
    async fn stream_turn(
        &self,
        request: &ChatCompletionRequest,
        cancellation: CancellationToken,
        on_delta: &(dyn Fn(String) -> Result<(), String> + Send + Sync),
    ) -> Result<(), ProviderError>;
}

struct CloudBackendAdapter {
    descriptor: BackendDescriptor,
    provider: Box<dyn provider::AiProvider>,
}

#[async_trait]
impl ModelBackend for CloudBackendAdapter {
    fn descriptor(&self) -> &BackendDescriptor {
        &self.descriptor
    }

    async fn health(&self) -> Result<BackendHealth, ProviderError> {
        self.provider.test_connection().await?;
        Ok(BackendHealth::Available)
    }

    async fn discover_models(&self) -> Result<Vec<ModelDescriptor>, ProviderError> {
        Ok(self.descriptor.models.clone())
    }

    async fn stream_turn(
        &self,
        request: &ChatCompletionRequest,
        cancellation: CancellationToken,
        on_delta: &(dyn Fn(String) -> Result<(), String> + Send + Sync),
    ) -> Result<(), ProviderError> {
        self.provider
            .stream_chat(request, cancellation, on_delta)
            .await
    }
}

pub fn create_backend(config: ProviderConfig) -> Result<Box<dyn ModelBackend>, ProviderError> {
    let descriptor = cloud_backend_descriptors()
        .into_iter()
        .find(|candidate| candidate.id == config.provider)
        .ok_or_else(|| ProviderError::public("unknown_provider", "Unknown AI provider."))?;
    let provider = provider::create_provider(config)?;
    Ok(Box::new(CloudBackendAdapter {
        descriptor,
        provider,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_cloud_adapters_use_the_same_backend_contract() {
        for (provider, model) in [("deepseek", "deepseek-v4-flash"), ("openai", "gpt-5-mini")] {
            let backend = create_backend(
                ProviderConfig::for_route(provider, "secret".into(), model).unwrap(),
            )
            .unwrap();
            assert_eq!(backend.descriptor().id, provider);
            assert!(backend
                .descriptor()
                .models
                .iter()
                .any(|item| item.id == model));
        }
    }
}
