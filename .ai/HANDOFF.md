# Codex Task Contract

## Contract metadata

| Field             | Value                                                        |
| ----------------- | ------------------------------------------------------------ |
| Schema version    | 3                                                            |
| Task ID           | `AI-TOOL-ROUTER-001`                                         |
| Status            | `complete`                                                   |
| Owner             | Codex                                                        |
| Last updated      | 2026-09-27                                                   |
| Related milestone | Aether 34 — tool-enabled AI Router integration                |
| Classification    | `planned_codex`                                              |
| Branch / worktree | `agent/ai-tool-router` / `A:\Aether Desktop`                 |

## Objective

Connect the provider-neutral AI Router to the existing closed native read-tool
registry through a bounded Rust-owned coordinator. Eligible models may propose
authorized Calendar and Task reads, while native routing, privacy, ToolScope,
budgets, cancellation, serialization, disclosure approval, and provenance
remain authoritative.

## Success criteria

- [x] Only models with registered tool capability can receive tools; non-tool turns remain unchanged.
- [x] Provider-specific tool-call wire formats remain inside the DeepSeek/OpenAI adapter boundary.
- [x] Advertised descriptors are the closed registry intersected with a natively created ToolScope.
- [x] The same immutable backend/model/location handles every round of one logical turn.
- [x] Native execution validates IDs, strict arguments, scope, classification, and per-tool limits before reads.
- [x] The loop enforces four rounds, eight calls, and 64 KiB aggregate result bytes with deterministic serialized execution.
- [x] Sensitive cloud tool results are never transmitted before an explicit matching one-time disclosure approval when policy requires it.
- [x] Streaming emits typed friendly tool/approval states and cancellation covers generation, tools, approval wait, and resumed generation.
- [x] Bounded provenance records tool IDs/counts/rounds/classes/sizes/approval/failure phase without tool bodies or provider wire JSON.
- [x] No write tool, academic deadline inference, provider fetch, arbitrary SQL/path/scope, migration, or provider switch is introduced.
- [x] Required focused/full validation, self-review, commit, push, and draft PR complete successfully.

## In scope / allowed paths

- `src-tauri/src/ai/**`, focused AI command/runtime wiring in `src-tauri/src/{commands,lib}.rs`.
- Existing AI message provenance JSON and repository tests; no schema change unless evidence invalidates this plan.
- `src/lib/db/{types,tauri}.ts`, `src/hooks/useAi.ts`, `src/components/ai/{AiView,AiView.test}.tsx`, and focused AI tests.
- ADR-034, architecture docs, and `.ai/{HANDOFF,PROJECT_STATE,TODO,SESSION_NOTES,CHANGELOG}.md`.

## Out of scope

- Write/Safe Action tools; Calendar/Task mutations; shell, filesystem, browser, email, finance, or network tools.
- Academic Deadline/Assignment/Course tools or inference from events, ICS, titles, descriptions, or keywords.
- CAL-ICS, MyTimetable synchronization, School authorization redesign, Pulse, Brightspace, local-model runtime, provider failover, or unrelated UI work.
- Full release validation, schema migration, or new dependencies unless required by verified implementation evidence.

## Architecture constraints

- Routing, privacy/disclosure, and tool authorization remain independent native authorities.
- ToolScope is created before dispatch from trusted product context and is never accepted from model/frontend output.
- Provider adapters translate between one normalized model-tool contract and provider wire JSON.
- Parallel provider calls are normalized then validated together and executed serially in stable request order.
- Approval is bound to logical request, backend, model, inventory, ToolScope summary, expiry, and one-time use; raw results stay only in the active native turn.
- Existing route/disclosure JSON stores bounded summaries; full payloads are never persisted.

## Dependencies

- Completed `AI-ROUTER-001` and `AI-NATIVE-TOOLS-001` plus ADR-032/ADR-033.
- Existing Tauri channel, cancellation runtime, SQLite connection, disclosure approval foundation, `serde_json`, `chrono`, and provider HTTP client.
- Official provider documentation verifies Chat Completions function/tool calling for the registered OpenAI and DeepSeek model families.

## Risks and safeguards

- **Premature disclosure:** gate serialized Sensitive result bytes before provider continuation and test absence before approval.
- **Scope escalation:** advertise/execute only the immutable native scope; reject unknown IDs and all injected identity/path/query fields.
- **Unbounded recursion:** hard round/call/aggregate-byte budgets survive all continuations and errors.
- **Route drift:** construct one backend from one RouteDecision and never invoke routing again within the turn.
- **Cancellation race:** check the logical-turn token before/after reads, approval, and provider continuation.
- **Private persistence/UI:** persist inventories and byte counts only; expose friendly status summaries, never raw results.

## Rollback considerations

The change is additive/refactoring within AI contracts and UI events. No migration
or dependency is planned. Reverting the cohesive task commit restores the current
no-tools execution path while retaining the independent native registry.

## Required validation

- Focused router, native tool, provider adapter, disclosure, cancellation, provenance, Calendar/Task regression, hook, IPC, and AI view tests.
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`
- `pnpm test`, `pnpm typecheck`, `pnpm lint`, `pnpm build`
- `git diff --check`
- Final diff review for route switching, scope escalation, premature disclosure, payload persistence, hidden writes, unbounded loops, model-controlled authority, and secret leakage.

## Independent review requirement

| Field          | Value                                                                                       |
| -------------- | ------------------------------------------------------------------------------------------- |
| Required       | No                                                                                          |
| Reason         | Repository workflow requires a distinct self-review; the owner did not request delegation. |
| Reviewer scope | Coordinator, adapters, scope, disclosure, cancellation, persistence, IPC/UI, and tests.     |

## Human decisions / blockers

None. The owner supplied the authority model, budgets, UX, validation, publication,
and explicit exclusions. Repository evidence determines the bounded implementation.

## Worktree / readiness gate

- Clean isolated worktree confirmed on `agent/ai-tool-router` at the completed native-tools head.
- No user-owned or parallel changes are present.
- Conflict-prone ownership is limited to the AI backend/provider/runtime/IPC boundary and handled sequentially here.
- ADR-034 records the durable coordinator/disclosure decision before production edits.
- Readiness gate passed; implementation may proceed.

## Stop condition

Stop after all acceptance criteria and required validation pass, self-review and
project records are complete, the task branch is pushed, and a draft PR is open.
Do not begin release validation or write-tool work.

## Completion evidence

- Added one normalized model-turn contract and a Rust-owned coordinator with hard
  limits of four tool rounds, eight calls, and 64 KiB aggregate serialized results.
- DeepSeek/OpenAI adapters own wire serialization and streamed tool-call assembly;
  one selected backend/model handles the full logical turn.
- Native product context advertises bounded Task tools and, only for an active
  parent School Space, locally authorized Calendar tools. All calls are validated
  together and executed serially in stable order.
- Sensitive cloud continuations wait for a request-, route-, inventory-, and
  scope-bound one-time approval; cancel removes the pending approval and stops the
  turn. The frontend exposes only a compact provider/model/category/count summary.
- Route provenance contains tool identifiers, counts, rounds, classifications,
  byte totals, approval outcome, and failure phase without result bodies.
- Validation: focused AI/frontend suites, full `cargo test` (247/247), full
  `pnpm test` (141/141 across 37 files), typecheck, lint, production build, Rust
  format, strict Clippy, and `git diff --check` all pass.
- Self-review found no route switching, write path, arbitrary SQL/path/scope,
  deadline inference, provider fetch, payload persistence, migration, dependency,
  or unrelated source change.
- Publication: implementation commit `a99c61d` is pushed to
  `origin/agent/ai-tool-router`; draft PR
  [#69](https://github.com/bimberlotDEV/Aether-Desktop/pull/69) is open.
