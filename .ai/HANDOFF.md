# Codex Task Contract

## Contract metadata

| Field | Value |
| --- | --- |
| Schema version | 3 |
| Task ID | `AI-ROUTER-001` |
| Status | `self_review` |
| Owner | Codex |
| Last updated | 2026-09-25 |
| Related milestone | Aether 28 — AI Router Phase 1 |
| Classification | `planned_codex` |
| Branch / worktree | `agent/ai-router` |

## Objective

Replace the cloud-specific AI route selector with a Rust-owned provider-neutral backend contract, deterministic Local only / Cloud only / Automatic routing, native privacy/disclosure policy, typed routing settings, and richer content-free provenance while preserving DeepSeek/OpenAI streaming, cancellation, credentials, proposal parsing, and Safe Actions separation.

## Success criteria

- [ ] DeepSeek and OpenAI implement one closed `ModelBackend` contract and retain fixed official endpoints, classified errors, streaming, and cancellation.
- [ ] `LocalOnly`, `CloudOnly`, and `Automatic` are deterministic; Local only truthfully fails because no local backend exists; no route changes after dispatch.
- [ ] Typed capabilities, task profile, context budget, environment, tool-scope placeholder, privacy snapshot, route request, immutable route decision, closed reasons, and typed failures are native-owned.
- [ ] Prompt/history and explicitly attached Aether context are classified natively; prohibited data cannot be cloud-serialized; sensitive disclosure and one-time approval foundations are enforced.
- [ ] Typed AI routing settings round-trip through dedicated commands and never expose or store credentials in `app_settings`.
- [ ] Migration 019 adds bounded route/disclosure provenance without rewriting historical content; legacy route semantics map deterministically.
- [ ] AI Settings exposes truthful routing/privacy controls and no-local-runtime state; responses show locality-aware provenance without raw decision JSON or hashes.
- [ ] Existing proposals and Safe Actions remain separate; no tools, local runtime, vision, embeddings, or provider fallback after dispatch are added.
- [ ] Required focused and full validation passes; no School/Calendar/Integration Sync path changes.

## In scope / allowed paths

- `src-tauri/src/ai/**`
- AI-specific portions of `src-tauri/src/commands.rs` and command registration in `src-tauri/src/lib.rs`
- `src-tauri/src/db/migrations.rs`
- `src-tauri/src/db/repositories/conversations.rs` and `settings.rs`
- `src-tauri/src/diagnostics.rs` only for the latest-schema expectation
- AI-specific schemas/wrappers/hooks/components/routes/tests under `src/`
- `docs/decisions/021-ai-provider-routing.md`, new ADR-032, `docs/database.md`
- `.ai/ARCHITECTURE.md`, `.ai/HANDOFF.md`, `.ai/TODO.md`, `.ai/PROJECT_STATE.md`, `.ai/SESSION_NOTES.md`, `.ai/CHANGELOG.md`
- publication scripts only through their existing interfaces

## Out of scope

- Ollama, llama.cpp, packaged runtimes, remote-private runtime implementations, discovery outside the closed registry, arbitrary endpoints, vision behavior, embeddings, context compression, native AI tools, calendar/task tools, and model tool execution.
- School, Calendar, MyTimetable, Brightspace, Integration Sync, Pulse, or unrelated UI redesign/refactoring.
- Reusing Safe Actions tokens for disclosure approval or allowing the model/frontend to lower native classification or authorize execution.

## Architecture constraints

- ADR-032 is the binding successor/extension to ADR-021 for routing, locality, disclosure, and provenance; ADR-021 remains binding for fixed endpoints and Safe Actions separation.
- Registry order is stable and closed. Filtering order is registered, enabled, routing locality, health/availability, capabilities, context capacity, privacy, approval, preference, then registry order.
- The route decision is created before request serialization and is not mutated after dispatch. A provider failure never chooses another backend/locality.
- `ExecutionLocation` is security-relevant. DeepSeek/OpenAI are Cloud; Phase 1 registers no OnDevice or RemotePrivate backend.
- Missing capability metadata is unsupported and unknown context capacity is conservative.
- Native classification floors are prompt/history = Personal; explicit Note/Task/Memory/Vault context = Sensitive; credentials and unbounded/raw sources = Prohibited.
- `CloudOnly` is standing consent only for ordinary prompt/history sent to its selected cloud backend. Sensitive Aether context follows disclosure policy and approval requirements.
- `ToolScope` is an empty/future-compatible authorization contract only; it grants no execution ability.
- Migration 019 is append-only and stores only IDs, enums, reason/capability summaries, disclosure categories, and approval metadata—never prompt/context/tool bodies, credentials, or raw provider errors.

## Dependencies

- Existing ADR-011 context isolation, ADR-021 provider routing/proposals, DPAPI credential store, cancellable streaming runtime, conversation repository, app settings repository, and Safe Actions.
- No new dependency.

## Risks and safeguards

- **Data egress:** native classification, cloud policy, and approval are checked before backend request construction; prohibited data is rejected.
- **Hidden recipient change:** one immutable route is persisted and dispatched; no post-selection fallback exists.
- **Legacy behavior regression:** legacy conversation provider/model fields are mapped deterministically and adapters reuse current request/SSE logic.
- **Nondeterminism:** router accepts an ordered candidate slice and never depends on map iteration, races, scores, or model output.
- **Sensitive provenance:** persisted JSON is generated from typed snapshots and tested not to contain request bodies.
- **Settings corruption:** dedicated commands validate closed enums/identifiers; credentials remain only in the secure credential store.

## Rollback considerations

Code/UI changes are reversible on the task branch. Migration 019 is append-only and nullable; older binaries ignore the added columns/settings. New routing settings have safe defaults when absent. Historical messages are not rewritten.

## Required validation

- Focused router, privacy/approval, backend adapter, provenance/migration, settings, streaming/cancellation, provider, proposal/Safe Actions, and frontend AI tests.
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`
- `pnpm typecheck`, `pnpm lint`, `pnpm test`, `pnpm build`
- `git diff --check`
- Focused desktop smoke steps recorded; automated native evidence is authoritative if interactive provider credentials are unavailable.

## Independent review requirement

| Field | Value |
| --- | --- |
| Required | No |
| Reason | The user did not request a separate agent/reviewer; workflow requires a distinct evidence-based self-review. |
| Reviewer scope | Routing determinism, privacy/approval boundaries, provenance redaction, migration compatibility, backend regression, and forbidden-path review. |

## Human decisions / blockers

None. The supplied task explicitly defines the modes, defaults, consent boundary, exclusions, validation, branch, and publication outcome.

## Worktree / ownership gate

| Check | State |
| --- | --- |
| Correct branch/worktree | Pass — clean `agent/ai-router` at merged Aether 27 head |
| User-owned changes | None |
| Parallel overlap | None identified |
| Serialization points | Migration 019, AI registry/IPC/settings, conversation provenance, ADR-032 |

## Readiness review

Passed. The objective, security authorities, consent behavior, compatibility mapping, schema strategy, allowed/forbidden paths, rollback, validation, publication, and stop condition are explicit. Production implementation may begin.

## Implementation log

- 2026-09-25: Bootstrapped repository controls, confirmed clean `agent/ai-router`, inspected the existing AI provider/router/context/conversation/settings boundaries and ADR-021, accepted ADR-032, and passed the readiness gate.
- Added the closed provider-neutral backend/capability contracts, deterministic three-mode router, native privacy/disclosure settings, approval-token foundation, migration 019 provenance, and focused AI settings/provenance UI.
- Preserved fixed DeepSeek/OpenAI adapters, DPAPI credentials, request shaping, streaming, cancellation, proposal parsing, Safe Actions separation, and explicit Space context resolution.
- Completed full validation and a distinct final-diff/security/scope self-review. Publication remains the only pending stop-condition item.

## Verification evidence

- Focused native AI backend/router/privacy/settings/context/provider/proposal/runtime tests are included in the final native suite.
- Focused AI Settings/View/hooks/typed IPC tests: 34 passed across 4 files after the no-local-runtime regression case.
- `cargo test --manifest-path src-tauri/Cargo.toml`: 216 passed.
- `pnpm test`: 136 passed across 37 files.
- `pnpm typecheck`: passed.
- `pnpm lint`: passed.
- `pnpm build`: passed.
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`: passed.
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`: passed.
- `git diff --check`: passed.

## Acceptance evidence

- Backend/compatibility: DeepSeek and OpenAI use one `ModelBackend`; current fixed endpoints, provider-specific shaping, classified errors, streaming, cancellation, credentials, and connection tests remain behind their adapters.
- Modes/determinism: ordered native candidates implement Local only, Cloud only, and Automatic local-first policy; no local backend is fabricated; offline, fallback, capability, context, privacy, approval, provider, and model failures are typed.
- Privacy/approval: native floors classify prompt/history as Personal and explicit/current-or-historical Aether context as Sensitive; Prohibited cannot select cloud; opaque five-minute one-time approvals bind request, backend, model, data inventory, and future tool-result inventory.
- Settings/provenance: dedicated typed commands own validated settings; migration 019 adds content-free route/disclosure fields and deterministic legacy mapping; raw JSON/hashes remain off IPC.
- UI: Settings exposes modes, fallback/disclosure policy, preferred cloud model, and truthful no-local-runtime copy; response provenance renders policy → locality → provider/model.
- Scope: no School, Calendar, MyTimetable, Brightspace, Integration Sync, tool execution, local runtime, vision, embedding, dependency, or arbitrary endpoint change exists.

## Self-review

Passed. The final changed-path list is limited to AI routing/backend/privacy/settings, AI-specific IPC/persistence/UI/tests, migration diagnostics, ADR/database/architecture documentation, and task records. The router consumes a stable ordered registry and never mutates or reselects a decision after backend dispatch. Local only does not read cloud credentials and the UI no longer uses cloud-key status as local eligibility. Explicit legacy provider conversations map to Cloud only under the default Automatic setting; an explicit Local-only setting still overrides them. Request/context bodies, credentials, approval hashes, raw provider errors, and raw route/disclosure JSON do not cross provenance IPC or appear in staged content. No new dependency was added. Interactive cloud-provider smoke was not run because no owner API keys were supplied; deterministic adapter tests and the existing connection-test path are the evidence, with manual steps recorded for completion.

## Publication state

Pending implementation commit, push, and draft PR.

## Manual desktop smoke steps

1. Open Settings → AI and verify Automatic, Cloud only, and Local only copy plus the no-local-runtime message.
2. With no local runtime, select Local only and verify AI send remains unavailable/truthful without depending on cloud-key state.
3. Select Cloud only, configure/test one existing provider key, send a prompt, cancel one stream, and verify completed/cancelled messages show Cloud provenance.
4. Select Automatic, send an ordinary prompt, and verify `Automatic → Cloud` provenance because Phase 1 has no local candidate.
5. Attach a Note under the default `Ask for Aether data` policy and verify dispatch is blocked with an approval-required message; select `Allow explicit attachments`, resend, and verify only displayed context is included.
6. Restart Aether and verify routing settings and message provenance persist. Real-provider execution requires owner-supplied keys and was not run in this session.

## Stop condition

Stop after the acceptance criteria are evidenced, required checks pass, self-review confirms no forbidden paths/secrets/unrelated changes, records are updated, the implementation is committed and pushed on `agent/ai-router`, and an Aether 28 draft PR is open.
