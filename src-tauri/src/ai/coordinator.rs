use async_trait::async_trait;
use serde::Serialize;
use serde_json::{json, Value};
use tokio_util::sync::CancellationToken;

use super::{
    backend::ModelBackend,
    capabilities::ExecutionLocation,
    model::{
        ModelMessage, ModelToolDescriptor, ModelToolRequest, ModelToolResult,
        ModelToolResultStatus, ModelTurnRequest,
    },
    privacy::DataClass,
    provider::ProviderError,
    routing::RouteDecision,
    tools::{
        native_tool_registry, validate_native_ai_tool_request, NativeToolId, NativeToolResult,
        ToolError, ToolExecutionContext, ToolScope,
    },
};

pub const MAX_TOOL_ROUNDS: u32 = 4;
pub const MAX_TOOL_CALLS: u32 = 8;
pub const MAX_TURN_TOOL_RESULT_BYTES: usize = 64 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolTurnPhase {
    Generating,
    ToolRequested,
    ToolRunning,
    AwaitingDisclosureApproval,
    GeneratingAfterTool,
    Completed,
    Cancelled,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolResultInventoryItem {
    pub tool_id: NativeToolId,
    pub item_count: u32,
    pub serialized_bytes: usize,
    pub data_class: DataClass,
}

#[derive(Debug, Clone)]
pub struct DisclosureRequest {
    pub logical_request_id: String,
    pub backend_id: String,
    pub model_id: String,
    pub inventory: Vec<ToolResultInventoryItem>,
    pub tool_scope_json: String,
}

#[async_trait]
pub trait ToolDisclosureGate: Send + Sync {
    async fn approve(
        &self,
        request: DisclosureRequest,
        cancellation: CancellationToken,
    ) -> Result<bool, CoordinatorError>;
}

pub trait NativeToolExecutor: Send + Sync {
    fn execute(
        &self,
        tool_id: NativeToolId,
        arguments: Value,
        scope: &ToolScope,
        context: ToolExecutionContext,
    ) -> Result<NativeToolResult, ToolError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoordinatorError {
    pub code: &'static str,
    pub message: String,
    pub phase: ToolTurnPhase,
}

impl CoordinatorError {
    fn new(code: &'static str, message: impl Into<String>, phase: ToolTurnPhase) -> Self {
        Self {
            code,
            message: message.into(),
            phase,
        }
    }

    fn cancelled() -> Self {
        Self::new(
            "cancelled",
            "The AI response was cancelled.",
            ToolTurnPhase::Cancelled,
        )
    }
}

impl From<ProviderError> for CoordinatorError {
    fn from(error: ProviderError) -> Self {
        Self::new(error.code, error.message, ToolTurnPhase::Failed)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolTurnProvenance {
    pub tool_enabled: bool,
    pub tool_ids: Vec<NativeToolId>,
    pub tool_call_count: u32,
    pub tool_round_count: u32,
    pub result_classes: Vec<DataClass>,
    pub result_sizes: Vec<usize>,
    pub disclosure_approval_used: bool,
    pub terminal_phase: Option<ToolTurnPhase>,
}

#[derive(Debug)]
pub struct ToolTurnOutcome {
    pub content: String,
    pub provenance: ToolTurnProvenance,
}

pub struct ToolCoordinatorRequest<'a> {
    pub logical_request_id: &'a str,
    pub route: &'a RouteDecision,
    pub scope: &'a ToolScope,
    pub messages: Vec<ModelMessage>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub top_p: Option<f32>,
    pub thinking_enabled: bool,
    pub context: ToolExecutionContext,
}

pub struct ToolTurnObservers<'a> {
    pub on_delta: &'a (dyn Fn(String) -> Result<(), String> + Send + Sync),
    pub on_phase:
        &'a (dyn Fn(ToolTurnPhase, Option<NativeToolId>) -> Result<(), String> + Send + Sync),
    pub on_provenance: &'a (dyn Fn(&ToolTurnProvenance) -> Result<(), String> + Send + Sync),
}

pub fn advertised_tools(scope: &ToolScope) -> Vec<ModelToolDescriptor> {
    native_tool_registry()
        .into_iter()
        .filter(|item| scope.permits(item.id))
        .map(|item| ModelToolDescriptor {
            tool_id: item.id,
            name: item.public_name.into(),
            description: item.description.into(),
            input_schema: item.input_schema,
        })
        .collect()
}

pub async fn run_tool_turn(
    backend: &dyn ModelBackend,
    executor: &dyn NativeToolExecutor,
    disclosure_gate: &dyn ToolDisclosureGate,
    request: ToolCoordinatorRequest<'_>,
    cancellation: CancellationToken,
    observers: ToolTurnObservers<'_>,
) -> Result<ToolTurnOutcome, CoordinatorError> {
    let tools = advertised_tools(request.scope);
    let mut messages = request.messages;
    let mut content = String::new();
    let mut provenance = ToolTurnProvenance {
        tool_enabled: !tools.is_empty(),
        ..Default::default()
    };
    (observers.on_provenance)(&provenance).map_err(|_| CoordinatorError::cancelled())?;
    let mut aggregate_bytes = 0usize;

    loop {
        cancelled(&cancellation)?;
        let phase = if provenance.tool_round_count == 0 {
            ToolTurnPhase::Generating
        } else {
            ToolTurnPhase::GeneratingAfterTool
        };
        (observers.on_phase)(phase, None).map_err(|_| CoordinatorError::cancelled())?;
        let round_text = std::sync::Mutex::new(String::new());
        let turn_request = ModelTurnRequest {
            model: request.route.model_id.clone(),
            messages: messages.clone(),
            tools: tools.clone(),
            temperature: request.temperature,
            max_tokens: request.max_tokens,
            top_p: request.top_p,
            thinking_enabled: request.thinking_enabled,
        };
        let result = backend
            .stream_turn(&turn_request, cancellation.clone(), &|delta| {
                round_text
                    .lock()
                    .map_err(|_| "AI response buffer is unavailable.".to_string())?
                    .push_str(&delta);
                (observers.on_delta)(delta)
            })
            .await?;
        let round_text = round_text.into_inner().map_err(|_| {
            CoordinatorError::new(
                "response_buffer",
                "The AI response buffer is unavailable.",
                ToolTurnPhase::Failed,
            )
        })?;
        content.push_str(&round_text);
        if result.tool_requests.is_empty() {
            provenance.terminal_phase = Some(ToolTurnPhase::Completed);
            (observers.on_provenance)(&provenance).map_err(|_| CoordinatorError::cancelled())?;
            (observers.on_phase)(ToolTurnPhase::Completed, None)
                .map_err(|_| CoordinatorError::cancelled())?;
            return Ok(ToolTurnOutcome {
                content,
                provenance,
            });
        }

        (observers.on_phase)(ToolTurnPhase::ToolRequested, None)
            .map_err(|_| CoordinatorError::cancelled())?;
        provenance.tool_round_count += 1;
        if provenance.tool_round_count > MAX_TOOL_ROUNDS
            || provenance
                .tool_call_count
                .saturating_add(result.tool_requests.len() as u32)
                > MAX_TOOL_CALLS
        {
            return Err(CoordinatorError::new(
                "tool_budget_exceeded",
                "The AI tool budget was exceeded.",
                ToolTurnPhase::Failed,
            ));
        }
        provenance.tool_call_count += result.tool_requests.len() as u32;
        (observers.on_provenance)(&provenance).map_err(|_| CoordinatorError::cancelled())?;
        validate_request_ids(&result.tool_requests)?;
        let validated = result
            .tool_requests
            .iter()
            .map(|tool_request| validate_request(tool_request, request.scope, request.context))
            .collect::<Vec<_>>();
        let mut model_results = Vec::with_capacity(result.tool_requests.len());
        let mut inventory = Vec::new();

        for (tool_request, validated) in result.tool_requests.iter().zip(validated) {
            cancelled(&cancellation)?;
            match validated {
                Ok((tool_id, arguments)) => {
                    (observers.on_phase)(ToolTurnPhase::ToolRunning, Some(tool_id))
                        .map_err(|_| CoordinatorError::cancelled())?;
                    provenance.tool_ids.push(tool_id);
                    match executor.execute(tool_id, arguments, request.scope, request.context) {
                        Ok(result) => {
                            let output = serde_json::to_value(&result.output).map_err(|_| {
                                CoordinatorError::new(
                                    "tool_serialization_failed",
                                    "The native tool result could not be serialized.",
                                    ToolTurnPhase::Failed,
                                )
                            })?;
                            let bytes =
                                serialized_result_bytes(ModelToolResultStatus::Ok, &output)?;
                            aggregate_bytes =
                                aggregate_bytes.checked_add(bytes).ok_or_else(|| {
                                    CoordinatorError::new(
                                        "tool_budget_exceeded",
                                        "The AI tool result budget was exceeded.",
                                        ToolTurnPhase::Failed,
                                    )
                                })?;
                            if aggregate_bytes > MAX_TURN_TOOL_RESULT_BYTES {
                                return Err(CoordinatorError::new(
                                    "tool_budget_exceeded",
                                    "The AI tool result budget was exceeded.",
                                    ToolTurnPhase::Failed,
                                ));
                            }
                            let item_count = output
                                .get("events")
                                .or_else(|| output.get("tasks"))
                                .and_then(Value::as_array)
                                .map_or_else(
                                    || {
                                        u32::from(
                                            output
                                                .get("event")
                                                .is_some_and(|value| !value.is_null()),
                                        )
                                    },
                                    |items| items.len() as u32,
                                );
                            inventory.push(ToolResultInventoryItem {
                                tool_id,
                                item_count,
                                serialized_bytes: bytes,
                                data_class: result.privacy_class,
                            });
                            provenance.result_classes.push(result.privacy_class);
                            provenance.result_sizes.push(bytes);
                            (observers.on_provenance)(&provenance)
                                .map_err(|_| CoordinatorError::cancelled())?;
                            model_results.push(ModelToolResult {
                                request_id: tool_request.request_id.clone(),
                                tool_name: tool_request.tool_name.clone(),
                                tool_id: Some(tool_id),
                                status: ModelToolResultStatus::Ok,
                                output,
                            });
                        }
                        Err(error) => {
                            let result = tool_error_result(tool_request, Some(tool_id), error);
                            add_result_bytes(&mut aggregate_bytes, result.status, &result.output)?;
                            model_results.push(result);
                        }
                    }
                }
                Err(error) => {
                    let result = tool_error_result(
                        tool_request,
                        NativeToolId::from_public_name(&tool_request.tool_name).ok(),
                        error,
                    );
                    add_result_bytes(&mut aggregate_bytes, result.status, &result.output)?;
                    model_results.push(result);
                }
            }
        }

        cancelled(&cancellation)?;
        let sensitive = inventory
            .iter()
            .any(|item| item.data_class >= DataClass::Sensitive);
        if request.route.execution_location == ExecutionLocation::Cloud && sensitive {
            (observers.on_phase)(ToolTurnPhase::AwaitingDisclosureApproval, None)
                .map_err(|_| CoordinatorError::cancelled())?;
            let tool_scope_json = serde_json::to_string(request.scope).map_err(|_| {
                CoordinatorError::new(
                    "approval_unavailable",
                    "Cloud disclosure approval is unavailable.",
                    ToolTurnPhase::Failed,
                )
            })?;
            let approved = disclosure_gate
                .approve(
                    DisclosureRequest {
                        logical_request_id: request.logical_request_id.into(),
                        backend_id: request.route.backend_id.clone(),
                        model_id: request.route.model_id.clone(),
                        inventory: inventory.clone(),
                        tool_scope_json,
                    },
                    cancellation.clone(),
                )
                .await?;
            if !approved {
                return Err(CoordinatorError::new(
                    "cloud_disclosure_cancelled",
                    "Cloud disclosure was not approved.",
                    ToolTurnPhase::Cancelled,
                ));
            }
            provenance.disclosure_approval_used = true;
            (observers.on_provenance)(&provenance).map_err(|_| CoordinatorError::cancelled())?;
        }
        cancelled(&cancellation)?;
        messages.push(ModelMessage::AssistantToolRequests {
            content: (!round_text.is_empty()).then_some(round_text),
            requests: result.tool_requests,
        });
        messages.extend(model_results.into_iter().map(ModelMessage::ToolResult));
    }
}

fn validate_request_ids(requests: &[ModelToolRequest]) -> Result<(), CoordinatorError> {
    let mut ids = std::collections::HashSet::new();
    if requests
        .iter()
        .any(|request| request.request_id.is_empty() || !ids.insert(&request.request_id))
    {
        return Err(CoordinatorError::new(
            "invalid_tool_call",
            "The AI provider returned duplicate or empty tool request IDs.",
            ToolTurnPhase::Failed,
        ));
    }
    Ok(())
}

fn validate_request(
    request: &ModelToolRequest,
    scope: &ToolScope,
    context: ToolExecutionContext,
) -> Result<(NativeToolId, Value), ToolError> {
    let tool_id = NativeToolId::from_public_name(&request.tool_name)?;
    let arguments =
        serde_json::from_str::<Value>(&request.arguments_json).map_err(|_| ToolError {
            code: super::tools::ToolErrorCode::InvalidArguments,
            message: "The native tool arguments are invalid.".into(),
        })?;
    validate_native_ai_tool_request(tool_id, &arguments, scope, context)?;
    Ok((tool_id, arguments))
}

fn tool_error_result(
    request: &ModelToolRequest,
    tool_id: Option<NativeToolId>,
    error: ToolError,
) -> ModelToolResult {
    ModelToolResult { request_id: request.request_id.clone(), tool_name: request.tool_name.clone(), tool_id, status: ModelToolResultStatus::Error, output: serde_json::to_value(error).unwrap_or_else(|_| json!({"code":"temporarily_unavailable","message":"The native tool is temporarily unavailable."})) }
}

fn serialized_result_bytes(
    status: ModelToolResultStatus,
    output: &Value,
) -> Result<usize, CoordinatorError> {
    serde_json::to_vec(&json!({ "status": status, "output": output }))
        .map(|bytes| bytes.len())
        .map_err(|_| {
            CoordinatorError::new(
                "tool_serialization_failed",
                "The native tool result could not be serialized.",
                ToolTurnPhase::Failed,
            )
        })
}

fn add_result_bytes(
    aggregate_bytes: &mut usize,
    status: ModelToolResultStatus,
    output: &Value,
) -> Result<(), CoordinatorError> {
    *aggregate_bytes = aggregate_bytes
        .checked_add(serialized_result_bytes(status, output)?)
        .ok_or_else(|| {
            CoordinatorError::new(
                "tool_budget_exceeded",
                "The AI tool result budget was exceeded.",
                ToolTurnPhase::Failed,
            )
        })?;
    if *aggregate_bytes > MAX_TURN_TOOL_RESULT_BYTES {
        return Err(CoordinatorError::new(
            "tool_budget_exceeded",
            "The AI tool result budget was exceeded.",
            ToolTurnPhase::Failed,
        ));
    }
    Ok(())
}

fn cancelled(cancellation: &CancellationToken) -> Result<(), CoordinatorError> {
    if cancellation.is_cancelled() {
        Err(CoordinatorError::cancelled())
    } else {
        Ok(())
    }
}

pub fn inventory_categories(
    inventory: &[ToolResultInventoryItem],
    tool_scope_json: &str,
) -> Vec<super::privacy::DataCategory> {
    let mut categories = inventory
        .iter()
        .map(|item| super::privacy::DataCategory {
            category: format!(
                "tool_result:{}:{}:{}",
                item.tool_id.public_name(),
                item.item_count,
                item.serialized_bytes
            ),
            data_class: item.data_class,
        })
        .collect::<Vec<_>>();
    categories.push(super::privacy::DataCategory {
        category: format!(
            "tool_scope:{}",
            super::privacy::request_hash(&[tool_scope_json])
        ),
        data_class: DataClass::Sensitive,
    });
    categories
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use chrono::{DateTime, NaiveDate, Utc};

    use super::*;
    use crate::ai::{
        backend::BackendHealth,
        capabilities::{
            BackendDescriptor, CapabilityEvidence, ModelCapabilities, ReasoningTier,
            StructuredOutputSupport, ToolCallingSupport,
        },
        model::ModelTurnOutcome,
        privacy::PrivacyDecision,
        provider::ProviderError,
        routing::{FallbackState, RouteReasonCode},
        settings::RoutingMode,
        tools::{NativeToolOutput, TaskToolItem, TasksOutput},
    };

    struct Backend {
        descriptor: BackendDescriptor,
        outcomes: Mutex<Vec<Result<ModelTurnOutcome, ProviderError>>>,
        requests: Mutex<Vec<ModelTurnRequest>>,
    }

    #[async_trait]
    impl ModelBackend for Backend {
        fn descriptor(&self) -> &BackendDescriptor {
            &self.descriptor
        }
        async fn health(&self) -> Result<BackendHealth, ProviderError> {
            Ok(BackendHealth::Available)
        }
        async fn discover_models(
            &self,
        ) -> Result<Vec<crate::ai::capabilities::ModelDescriptor>, ProviderError> {
            Ok(self.descriptor.models.clone())
        }
        async fn stream_turn(
            &self,
            request: &ModelTurnRequest,
            _cancellation: CancellationToken,
            on_delta: &(dyn Fn(String) -> Result<(), String> + Send + Sync),
        ) -> Result<ModelTurnOutcome, ProviderError> {
            self.requests.lock().unwrap().push(request.clone());
            on_delta("text".into()).map_err(|_| ProviderError::public("cancelled", "cancelled"))?;
            self.outcomes.lock().unwrap().remove(0)
        }
    }

    struct Executor {
        bytes: usize,
        calls: Mutex<Vec<NativeToolId>>,
    }
    impl NativeToolExecutor for Executor {
        fn execute(
            &self,
            tool_id: NativeToolId,
            _arguments: Value,
            _scope: &ToolScope,
            _context: ToolExecutionContext,
        ) -> Result<NativeToolResult, ToolError> {
            self.calls.lock().unwrap().push(tool_id);
            Ok(NativeToolResult {
                tool_id,
                output_schema_version: 1,
                privacy_class: DataClass::Sensitive,
                output: NativeToolOutput::Tasks(TasksOutput {
                    schema_version: 1,
                    tasks: vec![TaskToolItem {
                        id: "task".into(),
                        title: "x".repeat(self.bytes),
                        due_date: None,
                        status: "inbox".into(),
                        priority: "none".into(),
                        completed: false,
                    }],
                }),
            })
        }
    }

    struct Gate {
        approved: bool,
        calls: Mutex<Vec<DisclosureRequest>>,
    }
    #[async_trait]
    impl ToolDisclosureGate for Gate {
        async fn approve(
            &self,
            request: DisclosureRequest,
            cancellation: CancellationToken,
        ) -> Result<bool, CoordinatorError> {
            self.calls.lock().unwrap().push(request);
            if cancellation.is_cancelled() {
                return Err(CoordinatorError::cancelled());
            }
            Ok(self.approved)
        }
    }

    fn backend(outcomes: Vec<ModelTurnOutcome>) -> Backend {
        backend_results(outcomes.into_iter().map(Ok).collect())
    }

    fn backend_results(outcomes: Vec<Result<ModelTurnOutcome, ProviderError>>) -> Backend {
        Backend {
            descriptor: BackendDescriptor {
                id: "openai".into(),
                display_name: "OpenAI".into(),
                execution_location: ExecutionLocation::Cloud,
                models: vec![],
            },
            outcomes: Mutex::new(outcomes),
            requests: Mutex::new(vec![]),
        }
    }

    fn tool_call(id: &str, name: &str, arguments: &str) -> ModelTurnOutcome {
        ModelTurnOutcome {
            tool_requests: vec![ModelToolRequest {
                request_id: id.into(),
                tool_name: name.into(),
                arguments_json: arguments.into(),
            }],
        }
    }

    fn scope() -> ToolScope {
        ToolScope::tasks(
            vec![NativeToolId::TasksGetDue, NativeToolId::TasksGetOpen],
            Some((
                NaiveDate::from_ymd_opt(2026, 9, 19).unwrap(),
                NaiveDate::from_ymd_opt(2026, 10, 20).unwrap(),
            )),
            true,
            true,
            20,
        )
        .unwrap()
    }

    fn route() -> RouteDecision {
        RouteDecision {
            decision_id: "decision".into(),
            routing_mode: RoutingMode::Automatic,
            backend_id: "openai".into(),
            model_id: "gpt-5-mini".into(),
            execution_location: ExecutionLocation::Cloud,
            reason_code: RouteReasonCode::AutomaticCloudNoLocalRuntime,
            presentation_reason: "Cloud".into(),
            capability_snapshot: ModelCapabilities {
                text_generation: true,
                streaming: true,
                tool_calling: ToolCallingSupport::Parallel,
                structured_output: StructuredOutputSupport::JsonObject,
                vision: false,
                context_window_tokens: Some(64_000),
                maximum_output_tokens: Some(4_096),
                reasoning_tier: ReasoningTier::Standard,
                execution_location: ExecutionLocation::Cloud,
                capability_evidence: CapabilityEvidence::AetherVerified,
            },
            privacy_decision: PrivacyDecision::StandingCloudConsent,
            fallback_state: FallbackState::LocalUnavailableCloudSelected,
        }
    }

    fn context() -> ToolExecutionContext {
        ToolExecutionContext {
            now: DateTime::parse_from_rfc3339("2026-09-26T08:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            local_date: NaiveDate::from_ymd_opt(2026, 9, 26).unwrap(),
        }
    }

    async fn run(
        backend: &Backend,
        executor: &Executor,
        gate: &Gate,
        cancellation: CancellationToken,
    ) -> Result<ToolTurnOutcome, CoordinatorError> {
        run_tool_turn(
            backend,
            executor,
            gate,
            ToolCoordinatorRequest {
                logical_request_id: "request",
                route: &route(),
                scope: &scope(),
                messages: vec![ModelMessage::User("question".into())],
                temperature: None,
                max_tokens: None,
                top_p: None,
                thinking_enabled: false,
                context: context(),
            },
            cancellation,
            ToolTurnObservers {
                on_delta: &|_| Ok(()),
                on_phase: &|_, _| Ok(()),
                on_provenance: &|_| Ok(()),
            },
        )
        .await
    }

    #[tokio::test]
    async fn zero_tool_calls_complete_without_disclosure() {
        let backend = backend(vec![ModelTurnOutcome::default()]);
        let executor = Executor {
            bytes: 0,
            calls: Mutex::new(vec![]),
        };
        let gate = Gate {
            approved: true,
            calls: Mutex::new(vec![]),
        };
        let outcome = run(&backend, &executor, &gate, CancellationToken::new())
            .await
            .unwrap();
        assert_eq!(outcome.content, "text");
        assert_eq!(outcome.provenance.tool_call_count, 0);
        assert!(gate.calls.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn authorized_tool_continues_on_same_route_after_one_time_approval() {
        let backend = backend(vec![
            tool_call("call", "tasks.get_open", "{}"),
            ModelTurnOutcome::default(),
        ]);
        let executor = Executor {
            bytes: 0,
            calls: Mutex::new(vec![]),
        };
        let gate = Gate {
            approved: true,
            calls: Mutex::new(vec![]),
        };
        let outcome = run(&backend, &executor, &gate, CancellationToken::new())
            .await
            .unwrap();
        assert_eq!(outcome.provenance.tool_round_count, 1);
        assert!(outcome.provenance.disclosure_approval_used);
        let requests = backend.requests.lock().unwrap();
        assert_eq!(requests.len(), 2);
        assert!(requests.iter().all(|request| request.model == "gpt-5-mini"));
        assert!(requests
            .iter()
            .all(|request| request.tools.iter().all(|tool| matches!(
                tool.tool_id,
                NativeToolId::TasksGetDue | NativeToolId::TasksGetOpen
            ))));
        assert!(matches!(
            requests[1].messages.last(),
            Some(ModelMessage::ToolResult(_))
        ));
    }

    #[tokio::test]
    async fn invalid_and_unknown_requests_never_execute_and_can_be_corrected_once() {
        let backend = backend(vec![
            ModelTurnOutcome {
                tool_requests: vec![
                    ModelToolRequest {
                        request_id: "a".into(),
                        tool_name: "tasks.get_open".into(),
                        arguments_json: "{\"connection_id\":\"x\"}".into(),
                    },
                    ModelToolRequest {
                        request_id: "b".into(),
                        tool_name: "shell.exec".into(),
                        arguments_json: "{}".into(),
                    },
                ],
            },
            ModelTurnOutcome::default(),
        ]);
        let executor = Executor {
            bytes: 0,
            calls: Mutex::new(vec![]),
        };
        let gate = Gate {
            approved: true,
            calls: Mutex::new(vec![]),
        };
        run(&backend, &executor, &gate, CancellationToken::new())
            .await
            .unwrap();
        assert!(executor.calls.lock().unwrap().is_empty());
        assert!(gate.calls.lock().unwrap().is_empty());
        let requests = backend.requests.lock().unwrap();
        let results = requests[1]
            .messages
            .iter()
            .filter_map(|message| {
                if let ModelMessage::ToolResult(result) = message {
                    Some(result)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        assert_eq!(results.len(), 2);
        assert!(results
            .iter()
            .all(|result| result.status == ModelToolResultStatus::Error));
    }

    #[tokio::test]
    async fn call_round_and_aggregate_byte_budgets_fail_closed() {
        let calls = (0..9)
            .map(|index| ModelToolRequest {
                request_id: format!("call-{index}"),
                tool_name: "tasks.get_open".into(),
                arguments_json: "{}".into(),
            })
            .collect();
        let too_many = backend(vec![ModelTurnOutcome {
            tool_requests: calls,
        }]);
        let executor = Executor {
            bytes: 0,
            calls: Mutex::new(vec![]),
        };
        let gate = Gate {
            approved: true,
            calls: Mutex::new(vec![]),
        };
        assert_eq!(
            run(&too_many, &executor, &gate, CancellationToken::new())
                .await
                .unwrap_err()
                .code,
            "tool_budget_exceeded"
        );

        let oversized = backend(vec![ModelTurnOutcome {
            tool_requests: vec![
                ModelToolRequest {
                    request_id: "a".into(),
                    tool_name: "tasks.get_open".into(),
                    arguments_json: "{}".into(),
                },
                ModelToolRequest {
                    request_id: "b".into(),
                    tool_name: "tasks.get_open".into(),
                    arguments_json: "{}".into(),
                },
            ],
        }]);
        let large_executor = Executor {
            bytes: 40_000,
            calls: Mutex::new(vec![]),
        };
        assert_eq!(
            run(&oversized, &large_executor, &gate, CancellationToken::new())
                .await
                .unwrap_err()
                .code,
            "tool_budget_exceeded"
        );

        let rounds = backend(
            (0..5)
                .map(|index| tool_call(&format!("call-{index}"), "tasks.get_open", "{}"))
                .collect(),
        );
        assert_eq!(
            run(&rounds, &executor, &gate, CancellationToken::new())
                .await
                .unwrap_err()
                .code,
            "tool_budget_exceeded"
        );
    }

    #[tokio::test]
    async fn cancellation_and_rejected_disclosure_prevent_continuation() {
        let rejected_backend = backend(vec![tool_call("call", "tasks.get_open", "{}")]);
        let executor = Executor {
            bytes: 0,
            calls: Mutex::new(vec![]),
        };
        let rejecting_gate = Gate {
            approved: false,
            calls: Mutex::new(vec![]),
        };
        assert_eq!(
            run(
                &rejected_backend,
                &executor,
                &rejecting_gate,
                CancellationToken::new()
            )
            .await
            .unwrap_err()
            .code,
            "cloud_disclosure_cancelled"
        );
        assert_eq!(rejected_backend.requests.lock().unwrap().len(), 1);

        let cancelled = CancellationToken::new();
        cancelled.cancel();
        let backend = backend(vec![ModelTurnOutcome::default()]);
        assert_eq!(
            run(&backend, &executor, &rejecting_gate, cancelled)
                .await
                .unwrap_err()
                .code,
            "cancelled"
        );
        assert!(backend.requests.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn provider_failure_or_cancellation_after_tools_never_reroutes_or_reexecutes() {
        for code in ["provider_unavailable", "cancelled"] {
            let backend = backend_results(vec![
                Ok(tool_call("call", "tasks.get_open", "{}")),
                Err(ProviderError::public(code, "Provider stopped.")),
            ]);
            let executor = Executor {
                bytes: 0,
                calls: Mutex::new(vec![]),
            };
            let gate = Gate {
                approved: true,
                calls: Mutex::new(vec![]),
            };
            let error = run(&backend, &executor, &gate, CancellationToken::new())
                .await
                .unwrap_err();
            assert_eq!(error.code, code);
            assert_eq!(backend.requests.lock().unwrap().len(), 2);
            assert_eq!(
                executor.calls.lock().unwrap().as_slice(),
                &[NativeToolId::TasksGetOpen]
            );
            assert_eq!(gate.calls.lock().unwrap().len(), 1);
        }
    }
}
