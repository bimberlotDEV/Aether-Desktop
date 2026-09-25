use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{
    capabilities::{
        ExecutionLocation, ModelCapabilities, ReasoningTier, StructuredOutputSupport,
        ToolCallingSupport,
    },
    privacy::{self, DataPolicySnapshot, PrivacyDecision},
    settings::{AiRoutingSettings, AutomaticCloudFallback, RoutingMode},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TokenMeasurement {
    Exact,
    Estimated,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextBudget {
    pub estimated_input_tokens: u32,
    pub reserved_output_tokens: u32,
    pub reserved_tool_tokens: u32,
    pub measurement: TokenMeasurement,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CapabilityRequirements {
    pub text_generation: bool,
    pub streaming: bool,
    pub tool_calling: ToolCallingSupport,
    pub structured_output: StructuredOutputSupport,
    pub vision: bool,
    pub minimum_reasoning_tier: ReasoningTier,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LatencyPreference {
    Fast,
    Balanced,
    Quality,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskProfile {
    pub response_mode: String,
    pub reasoning_tier: ReasoningTier,
    pub latency_preference: LatencyPreference,
    pub requires_tools: bool,
    pub requires_structured_output: bool,
    pub requires_vision: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolScope {
    pub allowed_tool_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoutingPreferences {
    pub preferred_local_runtime: Option<String>,
    pub preferred_local_model: Option<String>,
    pub preferred_cloud_provider: Option<String>,
    pub preferred_cloud_model: Option<String>,
    pub automatic_cloud_fallback: AutomaticCloudFallback,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentSnapshot {
    pub offline: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteRequest {
    pub request_id: String,
    pub routing_mode: RoutingMode,
    pub task_profile: TaskProfile,
    pub capability_requirements: CapabilityRequirements,
    pub context_budget: ContextBudget,
    pub data_policy: DataPolicySnapshot,
    pub tool_scope: ToolScope,
    pub preferences: RoutingPreferences,
    pub environment: EnvironmentSnapshot,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteCandidate {
    pub backend_id: String,
    pub model_id: String,
    pub registered: bool,
    pub enabled: bool,
    pub available: bool,
    pub capabilities: ModelCapabilities,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RouteReasonCode {
    PreferredLocalModelEligible,
    AutomaticLocalPrivacyPreferred,
    AutomaticCloudRequiredForCapability,
    AutomaticCloudRequiredForContext,
    AutomaticCloudNoLocalRuntime,
    CloudOnlyPreferredModel,
    CloudOnlyRegistryOrder,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FallbackState {
    NotApplicable,
    LocalUnavailableCloudSelected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteDecision {
    pub decision_id: String,
    pub routing_mode: RoutingMode,
    pub backend_id: String,
    pub model_id: String,
    pub execution_location: ExecutionLocation,
    pub reason_code: RouteReasonCode,
    pub presentation_reason: String,
    pub capability_snapshot: ModelCapabilities,
    pub privacy_decision: PrivacyDecision,
    pub fallback_state: FallbackState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RouteFailureCode {
    NoLocalBackend,
    NoEligibleModel,
    OfflineCloudUnavailable,
    CapabilityMissing,
    ContextTooLarge,
    PrivacyBlocksCloud,
    CloudApprovalRequired,
    ProviderUnavailable,
    ModelUnavailable,
    CloudFallbackDisabled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteFailure {
    pub code: RouteFailureCode,
    pub message: String,
}

impl std::fmt::Display for RouteFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

pub fn preferences(settings: &AiRoutingSettings) -> RoutingPreferences {
    RoutingPreferences {
        preferred_local_runtime: settings.preferred_local_runtime.clone(),
        preferred_local_model: settings.preferred_local_model.clone(),
        preferred_cloud_provider: settings.preferred_cloud_provider.clone(),
        preferred_cloud_model: settings.preferred_cloud_model.clone(),
        automatic_cloud_fallback: settings.automatic_cloud_fallback,
    }
}

pub fn effective_mode(configured: RoutingMode, conversation_provider: &str) -> RoutingMode {
    if configured == RoutingMode::Automatic && conversation_provider != "auto" {
        RoutingMode::CloudOnly
    } else {
        configured
    }
}

pub fn select_route(
    request: &RouteRequest,
    candidates: &[RouteCandidate],
    disclosure_policy: super::settings::CloudDisclosurePolicy,
) -> Result<RouteDecision, RouteFailure> {
    let local_registered = candidates.iter().any(|candidate| {
        candidate.registered
            && candidate.capabilities.execution_location == ExecutionLocation::OnDevice
    });
    let mut eligible: Vec<&RouteCandidate> = candidates
        .iter()
        .filter(|candidate| candidate.registered && candidate.enabled)
        .filter(|candidate| {
            mode_allows(
                request.routing_mode,
                candidate.capabilities.execution_location,
            )
        })
        .filter(|candidate| candidate.available)
        .filter(|candidate| {
            capabilities_satisfy(&candidate.capabilities, &request.capability_requirements)
        })
        .filter(|candidate| context_fits(&candidate.capabilities, &request.context_budget))
        .filter(|candidate| {
            candidate.capabilities.execution_location == ExecutionLocation::OnDevice
                || request.data_policy.cloud_permitted
        })
        .collect();

    if request.routing_mode == RoutingMode::LocalOnly {
        if !local_registered {
            return failure(
                RouteFailureCode::NoLocalBackend,
                "No local AI runtime is configured. Local only never falls back to cloud.",
            );
        }
        return choose(request, &mut eligible, disclosure_policy, true, None);
    }

    if request.environment.offline {
        eligible.retain(|candidate| {
            candidate.capabilities.execution_location != ExecutionLocation::Cloud
        });
        if request.routing_mode == RoutingMode::CloudOnly || eligible.is_empty() {
            return failure(
                RouteFailureCode::OfflineCloudUnavailable,
                "Cloud AI is unavailable while Aether is offline.",
            );
        }
    }

    if request.routing_mode == RoutingMode::Automatic {
        let mut locals: Vec<&RouteCandidate> = eligible
            .iter()
            .copied()
            .filter(|candidate| {
                candidate.capabilities.execution_location == ExecutionLocation::OnDevice
            })
            .collect();
        if !locals.is_empty() {
            return choose(request, &mut locals, disclosure_policy, true, None);
        }
        if request.preferences.automatic_cloud_fallback == AutomaticCloudFallback::Never {
            return failure(
                RouteFailureCode::CloudFallbackDisabled,
                "Automatic cloud fallback is disabled and no eligible local model is available.",
            );
        }
        if request.preferences.automatic_cloud_fallback == AutomaticCloudFallback::PromptOnly
            && request.data_policy.highest_class >= super::privacy::DataClass::Sensitive
        {
            return failure(
                RouteFailureCode::CloudFallbackDisabled,
                "Automatic cloud fallback is limited to prompt-only requests.",
            );
        }
    }

    if !request.data_policy.cloud_permitted {
        if request.data_policy.cloud_approval_required
            && !request.data_policy.cloud_approval_present
        {
            return failure(
                RouteFailureCode::CloudApprovalRequired,
                "Cloud disclosure approval is required for the selected Aether context.",
            );
        }
        return failure(
            RouteFailureCode::PrivacyBlocksCloud,
            "Privacy policy does not permit this request to leave the device.",
        );
    }
    if eligible.is_empty() {
        return Err(diagnose_empty(request, candidates));
    }
    let automatic_cloud_reason = if request.routing_mode == RoutingMode::Automatic {
        let local_available: Vec<&RouteCandidate> = candidates
            .iter()
            .filter(|candidate| {
                candidate.registered
                    && candidate.enabled
                    && candidate.available
                    && candidate.capabilities.execution_location == ExecutionLocation::OnDevice
            })
            .collect();
        Some(if local_available.is_empty() {
            RouteReasonCode::AutomaticCloudNoLocalRuntime
        } else if !local_available.iter().any(|candidate| {
            capabilities_satisfy(&candidate.capabilities, &request.capability_requirements)
        }) {
            RouteReasonCode::AutomaticCloudRequiredForCapability
        } else if !local_available.iter().any(|candidate| {
            capabilities_satisfy(&candidate.capabilities, &request.capability_requirements)
                && context_fits(&candidate.capabilities, &request.context_budget)
        }) {
            RouteReasonCode::AutomaticCloudRequiredForContext
        } else {
            RouteReasonCode::AutomaticCloudNoLocalRuntime
        })
    } else {
        None
    };
    choose(
        request,
        &mut eligible,
        disclosure_policy,
        false,
        automatic_cloud_reason,
    )
}

fn choose(
    request: &RouteRequest,
    eligible: &mut Vec<&RouteCandidate>,
    disclosure_policy: super::settings::CloudDisclosurePolicy,
    local: bool,
    automatic_cloud_reason: Option<RouteReasonCode>,
) -> Result<RouteDecision, RouteFailure> {
    if eligible.is_empty() {
        return failure(
            classify_empty_failure(request),
            empty_message(classify_empty_failure(request)),
        );
    }
    let preferred = if local {
        request
            .preferences
            .preferred_local_runtime
            .as_ref()
            .zip(request.preferences.preferred_local_model.as_ref())
    } else {
        request
            .preferences
            .preferred_cloud_provider
            .as_ref()
            .zip(request.preferences.preferred_cloud_model.as_ref())
    };
    let selected = preferred
        .and_then(|(backend, model)| {
            eligible
                .iter()
                .find(|candidate| candidate.backend_id == *backend && candidate.model_id == *model)
                .copied()
        })
        .unwrap_or(eligible[0]);
    let location = selected.capabilities.execution_location;
    let privacy_decision = privacy::decision(location, disclosure_policy, &request.data_policy)
        .ok_or_else(|| RouteFailure {
            code: RouteFailureCode::PrivacyBlocksCloud,
            message: "Privacy policy does not permit this route.".into(),
        })?;
    let preferred_selected = preferred.is_some_and(|(backend, model)| {
        selected.backend_id == *backend && selected.model_id == *model
    });
    let reason_code = if local && preferred_selected {
        RouteReasonCode::PreferredLocalModelEligible
    } else if local {
        RouteReasonCode::AutomaticLocalPrivacyPreferred
    } else if request.routing_mode == RoutingMode::CloudOnly && preferred_selected {
        RouteReasonCode::CloudOnlyPreferredModel
    } else if request.routing_mode == RoutingMode::CloudOnly {
        RouteReasonCode::CloudOnlyRegistryOrder
    } else {
        automatic_cloud_reason.unwrap_or(RouteReasonCode::AutomaticCloudNoLocalRuntime)
    };
    Ok(RouteDecision {
        decision_id: Uuid::now_v7().to_string(),
        routing_mode: request.routing_mode,
        backend_id: selected.backend_id.clone(),
        model_id: selected.model_id.clone(),
        execution_location: location,
        reason_code,
        presentation_reason: presentation_reason(reason_code).into(),
        capability_snapshot: selected.capabilities.clone(),
        privacy_decision,
        fallback_state: if request.routing_mode == RoutingMode::Automatic && !local {
            FallbackState::LocalUnavailableCloudSelected
        } else {
            FallbackState::NotApplicable
        },
    })
}

fn mode_allows(mode: RoutingMode, location: ExecutionLocation) -> bool {
    match mode {
        RoutingMode::LocalOnly => location == ExecutionLocation::OnDevice,
        RoutingMode::CloudOnly => location == ExecutionLocation::Cloud,
        RoutingMode::Automatic => matches!(
            location,
            ExecutionLocation::OnDevice | ExecutionLocation::Cloud
        ),
    }
}

fn capabilities_satisfy(
    capabilities: &ModelCapabilities,
    required: &CapabilityRequirements,
) -> bool {
    (!required.text_generation || capabilities.text_generation)
        && (!required.streaming || capabilities.streaming)
        && tool_support(capabilities.tool_calling) >= tool_support(required.tool_calling)
        && structured_support(capabilities.structured_output)
            >= structured_support(required.structured_output)
        && (!required.vision || capabilities.vision)
        && capabilities.reasoning_tier >= required.minimum_reasoning_tier
}

fn context_fits(capabilities: &ModelCapabilities, budget: &ContextBudget) -> bool {
    let Some(window) = capabilities.context_window_tokens else {
        return false;
    };
    let Some(maximum_output) = capabilities.maximum_output_tokens else {
        return false;
    };
    budget.reserved_output_tokens <= maximum_output
        && budget
            .estimated_input_tokens
            .saturating_add(budget.reserved_output_tokens)
            .saturating_add(budget.reserved_tool_tokens)
            <= window
}

fn tool_support(value: ToolCallingSupport) -> u8 {
    match value {
        ToolCallingSupport::None => 0,
        ToolCallingSupport::Single => 1,
        ToolCallingSupport::Parallel => 2,
    }
}

fn structured_support(value: StructuredOutputSupport) -> u8 {
    match value {
        StructuredOutputSupport::None => 0,
        StructuredOutputSupport::JsonObject => 1,
        StructuredOutputSupport::JsonSchema => 2,
    }
}

fn classify_empty_failure(request: &RouteRequest) -> RouteFailureCode {
    if request.capability_requirements.vision
        || request.capability_requirements.tool_calling != ToolCallingSupport::None
    {
        RouteFailureCode::CapabilityMissing
    } else if request
        .context_budget
        .estimated_input_tokens
        .saturating_add(request.context_budget.reserved_output_tokens)
        .saturating_add(request.context_budget.reserved_tool_tokens)
        > 64_000
    {
        RouteFailureCode::ContextTooLarge
    } else {
        RouteFailureCode::NoEligibleModel
    }
}

fn diagnose_empty(request: &RouteRequest, candidates: &[RouteCandidate]) -> RouteFailure {
    if let Some((backend, model)) = request
        .preferences
        .preferred_cloud_provider
        .as_ref()
        .zip(request.preferences.preferred_cloud_model.as_ref())
    {
        if !candidates.iter().any(|candidate| {
            candidate.registered && candidate.backend_id == *backend && candidate.model_id == *model
        }) {
            return RouteFailure {
                code: RouteFailureCode::ModelUnavailable,
                message: "The preferred AI model is not available in the closed registry.".into(),
            };
        }
    }
    let permitted: Vec<&RouteCandidate> = candidates
        .iter()
        .filter(|candidate| {
            candidate.registered
                && candidate.enabled
                && mode_allows(
                    request.routing_mode,
                    candidate.capabilities.execution_location,
                )
        })
        .collect();
    if !permitted.is_empty() && permitted.iter().all(|candidate| !candidate.available) {
        return RouteFailure {
            code: RouteFailureCode::ProviderUnavailable,
            message: "The eligible AI provider is unavailable before dispatch.".into(),
        };
    }
    let code = classify_empty_failure(request);
    RouteFailure {
        code,
        message: empty_message(code).into(),
    }
}

fn empty_message(code: RouteFailureCode) -> &'static str {
    match code {
        RouteFailureCode::CapabilityMissing => {
            "No available model satisfies the required capabilities."
        }
        RouteFailureCode::ContextTooLarge => {
            "The request is too large for every eligible model. Reduce the attached context."
        }
        _ => "No eligible AI model is available for this request.",
    }
}

fn failure<T>(code: RouteFailureCode, message: &str) -> Result<T, RouteFailure> {
    Err(RouteFailure {
        code,
        message: message.into(),
    })
}

pub fn presentation_reason(code: RouteReasonCode) -> &'static str {
    match code {
        RouteReasonCode::PreferredLocalModelEligible => "Used the preferred eligible local model.",
        RouteReasonCode::AutomaticLocalPrivacyPreferred => {
            "Automatic used an eligible local model first."
        }
        RouteReasonCode::AutomaticCloudRequiredForCapability => {
            "Automatic used cloud because local models lacked a required capability."
        }
        RouteReasonCode::AutomaticCloudRequiredForContext => {
            "Automatic used cloud because local models could not fit the context."
        }
        RouteReasonCode::AutomaticCloudNoLocalRuntime => {
            "Automatic used cloud because no eligible local runtime is configured."
        }
        RouteReasonCode::CloudOnlyPreferredModel => {
            "Cloud only used the preferred eligible cloud model."
        }
        RouteReasonCode::CloudOnlyRegistryOrder => {
            "Cloud only used the first eligible configured cloud model."
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::{
        capabilities::{CapabilityEvidence, ModelCapabilities},
        privacy::{self, DataClass},
        settings::CloudDisclosurePolicy,
    };

    fn capabilities(location: ExecutionLocation, window: Option<u32>) -> ModelCapabilities {
        ModelCapabilities {
            text_generation: true,
            streaming: true,
            tool_calling: ToolCallingSupport::None,
            structured_output: StructuredOutputSupport::JsonObject,
            vision: false,
            context_window_tokens: window,
            maximum_output_tokens: Some(4_096),
            reasoning_tier: ReasoningTier::Advanced,
            execution_location: location,
            capability_evidence: CapabilityEvidence::AetherRegistry,
        }
    }

    fn candidate(backend: &str, model: &str, location: ExecutionLocation) -> RouteCandidate {
        RouteCandidate {
            backend_id: backend.into(),
            model_id: model.into(),
            registered: true,
            enabled: true,
            available: true,
            capabilities: capabilities(location, Some(64_000)),
        }
    }

    fn request(mode: RoutingMode) -> RouteRequest {
        RouteRequest {
            request_id: "request".into(),
            routing_mode: mode,
            task_profile: TaskProfile {
                response_mode: "ask".into(),
                reasoning_tier: ReasoningTier::Basic,
                latency_preference: LatencyPreference::Balanced,
                requires_tools: false,
                requires_structured_output: false,
                requires_vision: false,
            },
            capability_requirements: CapabilityRequirements {
                text_generation: true,
                streaming: true,
                tool_calling: ToolCallingSupport::None,
                structured_output: StructuredOutputSupport::None,
                vision: false,
                minimum_reasoning_tier: ReasoningTier::Basic,
            },
            context_budget: ContextBudget {
                estimated_input_tokens: 100,
                reserved_output_tokens: 4096,
                reserved_tool_tokens: 0,
                measurement: TokenMeasurement::Estimated,
            },
            data_policy: privacy::snapshot(
                privacy::classify(false),
                mode,
                CloudDisclosurePolicy::AskForAetherData,
                false,
            ),
            tool_scope: ToolScope::default(),
            preferences: RoutingPreferences {
                preferred_local_runtime: None,
                preferred_local_model: None,
                preferred_cloud_provider: Some("openai".into()),
                preferred_cloud_model: Some("gpt".into()),
                automatic_cloud_fallback: AutomaticCloudFallback::AskWhenNeeded,
            },
            environment: EnvironmentSnapshot { offline: false },
        }
    }

    #[test]
    fn local_only_is_truthful_without_a_local_runtime() {
        let result = select_route(
            &request(RoutingMode::LocalOnly),
            &[candidate("openai", "gpt", ExecutionLocation::Cloud)],
            CloudDisclosurePolicy::AskForAetherData,
        );
        assert_eq!(result.unwrap_err().code, RouteFailureCode::NoLocalBackend);
    }

    #[test]
    fn cloud_only_prefers_configured_model_and_automatic_is_local_first() {
        let candidates = [
            candidate("deepseek", "flash", ExecutionLocation::Cloud),
            candidate("openai", "gpt", ExecutionLocation::Cloud),
            candidate("local", "small", ExecutionLocation::OnDevice),
        ];
        assert_eq!(
            select_route(
                &request(RoutingMode::CloudOnly),
                &candidates,
                CloudDisclosurePolicy::AskForAetherData
            )
            .unwrap()
            .backend_id,
            "openai"
        );
        assert_eq!(
            select_route(
                &request(RoutingMode::Automatic),
                &candidates,
                CloudDisclosurePolicy::AskForAetherData
            )
            .unwrap()
            .backend_id,
            "local"
        );
    }

    #[test]
    fn legacy_explicit_conversations_remain_cloud_only_until_mode_is_overridden() {
        assert_eq!(
            effective_mode(RoutingMode::Automatic, "openai"),
            RoutingMode::CloudOnly
        );
        assert_eq!(
            effective_mode(RoutingMode::Automatic, "auto"),
            RoutingMode::Automatic
        );
        assert_eq!(
            effective_mode(RoutingMode::LocalOnly, "openai"),
            RoutingMode::LocalOnly
        );
    }

    #[test]
    fn automatic_cloud_fallback_offline_privacy_and_disabled_are_typed() {
        let cloud = [candidate("deepseek", "flash", ExecutionLocation::Cloud)];
        let mut value = request(RoutingMode::Automatic);
        value.preferences.automatic_cloud_fallback = AutomaticCloudFallback::Never;
        assert_eq!(
            select_route(&value, &cloud, CloudDisclosurePolicy::AskForAetherData)
                .unwrap_err()
                .code,
            RouteFailureCode::CloudFallbackDisabled
        );
        value.preferences.automatic_cloud_fallback = AutomaticCloudFallback::AskWhenNeeded;
        value.environment.offline = true;
        assert_eq!(
            select_route(&value, &cloud, CloudDisclosurePolicy::AskForAetherData)
                .unwrap_err()
                .code,
            RouteFailureCode::OfflineCloudUnavailable
        );
        value.environment.offline = false;
        value.data_policy = privacy::snapshot(
            vec![privacy::DataCategory {
                category: "credential".into(),
                data_class: DataClass::Prohibited,
            }],
            RoutingMode::Automatic,
            CloudDisclosurePolicy::AllowExplicitAttachments,
            true,
        );
        assert_eq!(
            select_route(
                &value,
                &cloud,
                CloudDisclosurePolicy::AllowExplicitAttachments
            )
            .unwrap_err()
            .code,
            RouteFailureCode::PrivacyBlocksCloud
        );
    }

    #[test]
    fn context_boundaries_unknown_metadata_and_stable_order_are_deterministic() {
        let candidates = [
            candidate("first", "a", ExecutionLocation::Cloud),
            candidate("second", "b", ExecutionLocation::Cloud),
        ];
        let mut value = request(RoutingMode::CloudOnly);
        value.preferences.preferred_cloud_provider = None;
        value.preferences.preferred_cloud_model = None;
        value.context_budget.estimated_input_tokens = 59_904;
        assert_eq!(
            select_route(&value, &candidates, CloudDisclosurePolicy::AskForAetherData)
                .unwrap()
                .backend_id,
            "first"
        );
        value.context_budget.estimated_input_tokens = 59_905;
        assert_eq!(
            select_route(&value, &candidates, CloudDisclosurePolicy::AskForAetherData)
                .unwrap_err()
                .code,
            RouteFailureCode::ContextTooLarge
        );
        value.context_budget.estimated_input_tokens = 100;
        let unknown = RouteCandidate {
            capabilities: capabilities(ExecutionLocation::Cloud, None),
            ..candidate("unknown", "u", ExecutionLocation::Cloud)
        };
        assert!(select_route(&value, &[unknown], CloudDisclosurePolicy::AskForAetherData).is_err());
    }

    #[test]
    fn presentation_reasons_are_closed_and_not_model_generated() {
        assert_eq!(
            presentation_reason(RouteReasonCode::CloudOnlyPreferredModel),
            "Cloud only used the preferred eligible cloud model."
        );
    }

    #[test]
    fn unavailable_provider_and_missing_preferred_model_are_typed() {
        let mut unavailable = candidate("deepseek", "flash", ExecutionLocation::Cloud);
        unavailable.available = false;
        let mut value = request(RoutingMode::CloudOnly);
        value.preferences.preferred_cloud_provider = None;
        value.preferences.preferred_cloud_model = None;
        assert_eq!(
            select_route(
                &value,
                &[unavailable],
                CloudDisclosurePolicy::AskForAetherData
            )
            .unwrap_err()
            .code,
            RouteFailureCode::ProviderUnavailable
        );
        value.preferences.preferred_cloud_provider = Some("openai".into());
        value.preferences.preferred_cloud_model = Some("missing".into());
        assert_eq!(
            select_route(&value, &[], CloudDisclosurePolicy::AskForAetherData)
                .unwrap_err()
                .code,
            RouteFailureCode::ModelUnavailable
        );
    }
}
