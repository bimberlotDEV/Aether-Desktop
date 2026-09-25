use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionLocation {
    OnDevice,
    Cloud,
    RemotePrivate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolCallingSupport {
    None,
    Single,
    Parallel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StructuredOutputSupport {
    None,
    JsonObject,
    JsonSchema,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReasoningTier {
    Basic,
    Standard,
    Advanced,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityEvidence {
    AetherRegistry,
    RuntimeReported,
    AetherVerified,
    UserOverride,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelCapabilities {
    pub text_generation: bool,
    pub streaming: bool,
    pub tool_calling: ToolCallingSupport,
    pub structured_output: StructuredOutputSupport,
    pub vision: bool,
    pub context_window_tokens: Option<u32>,
    pub maximum_output_tokens: Option<u32>,
    pub reasoning_tier: ReasoningTier,
    pub execution_location: ExecutionLocation,
    pub capability_evidence: CapabilityEvidence,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelDescriptor {
    pub id: String,
    pub display_name: String,
    pub backend_id: String,
    pub capabilities: ModelCapabilities,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackendDescriptor {
    pub id: String,
    pub display_name: String,
    pub execution_location: ExecutionLocation,
    pub models: Vec<ModelDescriptor>,
}

pub fn cloud_backend_descriptors() -> Vec<BackendDescriptor> {
    vec![
        backend(
            "deepseek",
            "DeepSeek",
            vec![
                model(
                    "deepseek",
                    "deepseek-v4-flash",
                    "DeepSeek V4 Flash",
                    ReasoningTier::Standard,
                ),
                model(
                    "deepseek",
                    "deepseek-v4-pro",
                    "DeepSeek V4 Pro",
                    ReasoningTier::Advanced,
                ),
            ],
        ),
        backend(
            "openai",
            "OpenAI",
            vec![
                model(
                    "openai",
                    "gpt-5-mini",
                    "GPT-5 mini",
                    ReasoningTier::Standard,
                ),
                model("openai", "gpt-5.2", "GPT-5.2", ReasoningTier::Advanced),
            ],
        ),
    ]
}

fn backend(id: &str, display_name: &str, models: Vec<ModelDescriptor>) -> BackendDescriptor {
    BackendDescriptor {
        id: id.into(),
        display_name: display_name.into(),
        execution_location: ExecutionLocation::Cloud,
        models,
    }
}

fn model(
    backend_id: &str,
    id: &str,
    display_name: &str,
    reasoning_tier: ReasoningTier,
) -> ModelDescriptor {
    ModelDescriptor {
        id: id.into(),
        display_name: display_name.into(),
        backend_id: backend_id.into(),
        capabilities: ModelCapabilities {
            text_generation: true,
            streaming: true,
            tool_calling: ToolCallingSupport::None,
            structured_output: StructuredOutputSupport::JsonObject,
            vision: false,
            context_window_tokens: Some(64_000),
            maximum_output_tokens: Some(4_096),
            reasoning_tier,
            execution_location: ExecutionLocation::Cloud,
            capability_evidence: CapabilityEvidence::AetherRegistry,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closed_registry_has_stable_cloud_descriptors() {
        let backends = cloud_backend_descriptors();
        assert_eq!(
            backends
                .iter()
                .map(|item| item.id.as_str())
                .collect::<Vec<_>>(),
            vec!["deepseek", "openai"]
        );
        assert!(backends
            .iter()
            .all(|backend| backend.execution_location == ExecutionLocation::Cloud));
        assert!(backends
            .iter()
            .flat_map(|backend| &backend.models)
            .all(|model| {
                model.capabilities.streaming
                    && !model.capabilities.vision
                    && model.capabilities.tool_calling == ToolCallingSupport::None
            }));
    }
}
