# Codex Task Contract

| Field             | Value                      |
| ----------------- | -------------------------- |
| Schema version    | 2                          |
| Task ID           | `AI-EVOL-001`              |
| Status            | `in_review`                |
| Owner             | Codex                      |
| Last updated      | 2026-08-27                 |
| Related milestone | Milestone G — AI Evolution |
| Classification    | `planned_codex`            |

## Objective

Evolve Aether from a DeepSeek-specific chat into a transparent multi-provider intelligence layer with deterministic Auto routing and strictly user-approved AI proposals that reuse the existing Safe Actions boundary.

## Context

Milestones A–F are merged at `51fd6d9`. AI already has cancellable streaming, persisted Space-isolated conversations, explicit context, response modes, DPAPI-protected DeepSeek credentials, and a provider trait. The implementation still hardcodes DeepSeek in conversation validation, schemas, settings, model selection, provider creation, and error text. Milestone F now supplies a closed preview → approval → one-time execution Action boundary.

Official provider contracts checked on 2026-08-27: DeepSeek and OpenAI both expose SSE Chat Completions; their provider-specific request fields and error semantics differ. Official documentation explicitly requires generated tool arguments to be treated as untrusted and validated before use.

## Ordered checkpoints

1. **Provider foundation:** capability registry, provider-aware encrypted credentials, OpenAI adapter, dynamic typed model catalog, and unchanged DeepSeek behavior.
2. **Auto routing:** deterministic Rust route decision from mode, configured providers, model capabilities, and user preference; no silent remote fallback; persisted per-response provenance and visible disclosure.
3. **Approved proposals:** a strict Rust-parsed Task/Note draft mode that can only enter the existing Safe Actions review; the model never receives an approval token or execution command.
4. **Product integration:** provider settings, Auto/manual selection, route explanations, accessible proposal card, honest browser/offline/error states, docs, release verification, publication, and merge.

## Acceptance criteria

- [x] Rust owns a closed provider registry for `deepseek` and `openai`, fixed official endpoints, curated model capabilities, provider-specific request shaping, uniform cancellation/streaming, and presentation-safe errors.
- [x] Provider API keys are separately DPAPI-encrypted, never returned/logged/exported, independently testable/removable, and the legacy DeepSeek key continues to work without destructive migration.
- [x] OpenAI can be manually selected through the same persisted conversation/streaming flow; no custom endpoint, arbitrary URL, SDK secret, or provider credential crosses IPC.
- [x] `Auto` is a real Rust route preference, not a provider/model alias. It deterministically selects only a configured provider, records the resolved provider/model/reason on each assistant response, and never silently retries through another remote provider.
- [x] Existing conversations and populated personal-beta databases upgrade append-only and remain readable; fresh creation, upgrade, idempotence, rollback, and repository behavior are tested.
- [x] The UI exposes connected-provider state, test/save/remove controls, Auto plus explicit model choices, and a calm per-response route disclosure including remote processing and visible context count.
- [x] AI Action output is parsed strictly in Rust into only `createTask` or `createNote`, is size/count bounded, validated against the current active Space, attributed to its conversation/message, and cannot contain file, shell, delete, network, or approval capabilities.
- [x] An AI draft cannot execute directly: the user must open the existing Safe Actions preview and separately choose “Approve and execute”; cancellation, expiry, replay protection, and audit behavior remain owned by Milestone F.
- [x] Existing `create_tasks` behavior remains compatible or is safely converged without introducing a weaker second mutation path.
- [x] Cross-Space context, prompt-injected tool text, malformed JSON, unknown provider/model, missing credentials, stale route state, stream races, and provider failures have automated coverage.
- [ ] Frontend/Rust gates, migration tests, build/audit, packaging, diff/security review, responsive theme/browser smoke, packaged startup, and exact-head Windows CI pass.

## Allowed paths

- `src-tauri/src/ai/`
- `src-tauri/src/actions.rs`
- `src-tauri/src/commands.rs`
- `src-tauri/src/lib.rs`
- `src-tauri/src/db/migrations.rs`
- `src-tauri/src/db/repositories/conversations.rs`
- `src-tauri/src/db/repositories/activity.rs`
- `src/lib/db/`
- `src/hooks/useAi*.ts`
- `src/components/ai/`
- `src/routes/AI.tsx`
- `src/routes/Settings.tsx`
- `src/styles/index.css`
- `README.md`
- `docs/architecture.md`
- `docs/decisions/021-ai-provider-routing.md`
- `.ai/*`

## Non-goals

- General autonomous computer control, background agents, automation schedules, multi-step plans, browser/shell execution, arbitrary endpoints, plugin/tool execution, destructive actions, or model-owned approval.
- File Action proposals until a separate design makes Source/file metadata an explicit visible AI attachment; existing manual file Actions remain available.
- Silent cross-provider fallback, automatic Memory, telemetry, cloud sync, managed billing, subscriptions, Anthropic/Gemini support, or a bundled local-model runtime.
- Replacing the entire AI API with OpenAI Responses during this milestone; Chat Completions preserves the existing stable streaming abstraction while provider-specific adapters remain separable.

## Risks and safeguards

- **Secret exposure:** namespaced secret keys remain behind the DPAPI Rust boundary; status IPC returns booleans only.
- **Unexpected data recipient:** Auto considers configured providers but selects one before sending; the UI identifies remote provider/model and there is no automatic fallback after send failure.
- **Provider drift:** capability/model catalogs are centralized, tested, and rejected when unknown; provider-specific bodies are separate.
- **Migration damage:** append nullable provenance columns only; legacy rows need no rewrite and upgrade runs transactionally.
- **Prompt injection/confused deputy:** model output is untrusted data, strict Rust parsing permits only two non-file drafts, and execution still needs an opaque F token plus explicit approval.
- **Single approval policy:** AI proposal mutation exists only through Safe Actions; the former direct Task batch command is removed.
- **Rollback:** remove OpenAI/Auto/proposal UI and commands, retain nullable provenance, legacy secret, and readable conversations; no user content is deleted.

## Required validation

```text
pnpm check
pnpm build
pnpm audit --audit-level high
cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all-targets --all-features
pnpm tauri:build
git diff --check
browser AI/Settings smoke at 1024x640 in dark and light themes
packaged desktop startup smoke
GitHub PR exact-head CI
```

Live OpenAI/DeepSeek requests are not required for acceptance because no owner credentials may be requested, extracted, or logged. HTTP request shaping, SSE decoding, routing, credential isolation, and failure behavior require deterministic local tests; the UI must honestly report unconfigured providers.

## Blocking decisions

None. The owner authorized the next milestone. Adding one fixed-endpoint provider, transparent Auto, and Task/Note-only drafts is reversible and follows the approved roadmap without enabling general autonomy.

## Implementation evidence and self-review

- Provider request-shaping, SSE, routing, secret isolation, strict proposal parsing, migration preservation, Space isolation, one-time approval, and AI-origin audit tests pass locally.
- The direct `create_tasks_batch` mutation command and its frontend bridge were removed; AI drafts can now reach mutation only through a server-reconstructed Safe Action preview.
- `pnpm check` passes 77/77 tests across 29 files; production build and high-severity dependency audit pass.
- strict Rust format and Clippy pass; 95/95 Rust tests pass.
- MSI and NSIS package successfully; the final release executable starts and remains responsive.
- AI and Settings smoke at 1024×640 passes in light/dark themes with no browser warnings or errors.
- Diff/security review found only two fixed official provider URLs, no credential return path, no arbitrary endpoint, no silent fallback, and no model-owned token or execution path.
- Live provider calls were intentionally not run because owner credentials are neither required nor available; exact-head GitHub CI remains the final review item after publication.

## Readiness review

- **Status:** Ready. Provider scope, routing semantics, consent boundary, migration, rollback, evidence, and non-goals are explicit.
- **Architecture gate:** ADR-021 is accepted before production implementation.
- **Migration gate:** append-only nullable message provenance; no existing row or secret is rewritten.
- **Security gate:** Rust selects the recipient, resolves explicit context, parses drafts, and delegates approval/execution exclusively to Safe Actions.
