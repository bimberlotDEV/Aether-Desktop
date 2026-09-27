# ADR-034 — Bounded tool-enabled AI Router coordinator

- **Status:** Accepted
- **Date:** 2026-09-26
- **Task:** `AI-TOOL-ROUTER-001`
- **Extends:** ADR-032, ADR-033

## Context

ADR-032 fixes routing and cloud-disclosure authority in Rust. ADR-033 exposes
four bounded Sensitive read tools through a closed native registry and typed
ToolScope, but deliberately provides no model execution loop. Connecting them
must not let a model select providers, expand local authority, disclose local
results, recurse without bounds, or turn provider wire structures into product
contracts.

## Decision

A Rust coordinator owns each logical tool-enabled turn. It receives one immutable
RouteDecision, one native ToolScope, a closed advertised descriptor set, one
cancellation token, and one budget ledger. The selected backend/model/location
cannot change after dispatch. Provider adapters translate streaming Chat
Completions tool calls and continuations to a normalized internal request/result
contract; provider response IDs and wire JSON do not cross that boundary.

The coordinator allows at most four tool rounds, eight total calls, and 64 KiB of
serialized result payload. It validates a complete requested batch before local
execution and serializes independent read calls in stable provider request order.
Invalid arguments and other safe tool errors may be returned once as normalized
tool results and consume the same budgets. Unknown or unauthorized tools fail
closed. ToolScope is constructed before model dispatch from trusted product
context and never from prompt/model arguments.

For cloud routes, native Sensitive tool results create a distinct disclosure
event. When policy requires approval, the coordinator emits a minimized pending
summary and waits without issuing a continuation request. An explicit UI action
consumes a short-lived one-time native approval bound to the logical request,
backend, model, ordered result inventory, ToolScope summary, and expiry. Cancel
or token cancellation ends the wait and prevents later transmission.

Only friendly typed lifecycle events cross IPC. Existing provenance JSON stores
bounded tool IDs, counts, rounds, result classes/sizes, whether approval was used,
and terminal phase. It stores no result body, provider wire object, or approval
secret, so no migration is required.

## Consequences

- Tool-capable and ordinary turns share one router without provider-directed routing.
- OpenAI and DeepSeek differences remain adapter-local.
- Provider parallel requests are accepted but executed deterministically in order.
- Cloud answers may pause for explicit local-data disclosure approval.
- Write tools remain a separate Safe Actions design.
- Calendar access remains parent-School-Space scoped; Tasks remain bounded local reads.
- No academic Deadline tool or event-text inference is introduced.

## Capability evidence

The closed catalog marks the registered OpenAI and DeepSeek cloud models as
tool-capable only because their current official Chat Completions contracts
document function/tool descriptors, streamed `tool_calls`, and tool-result
continuations. Evidence checked on 2026-09-26:

- <https://platform.openai.com/docs/api-reference/chat>
- <https://api-docs.deepseek.com/guides/tool_calls/>
- <https://api-docs.deepseek.com/api/create-chat-completion/>
