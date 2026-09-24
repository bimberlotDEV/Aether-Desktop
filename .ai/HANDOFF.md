# Codex Task Contract

> Active implementation contract for the current planned Codex task.
> Completed task history belongs in `.ai/CHANGELOG.md`.

## Contract metadata

| Field | Value |
| --- | --- |
| Schema version | 3 |
| Task ID | `CAL-SUB-ROTATE-001` |
| Status | `complete` |
| Owner | Codex |
| Last updated | 2026-09-25 |
| Related milestone | Aether 25 — generation-safe subscribed-calendar replacement |
| Classification | `planned_codex` |
| Branch / worktree | `agent/calendar-rotate` |

## Objective

Make subscribed-calendar replacement generation-safe so work prepared for an older feed can never mutate current events, validators, success/failure metadata, or retry state.

## Context

- Aether 24 added bounded ICS parsing and origin-bound validators in migration 016.
- MyTimetable and Brightspace share `subscribed_calendar_provider` and the native Integration Sync runtime.
- Feed URLs are DPAPI-encrypted values in the SQLite `secrets` table referenced by Integration records.
- Cached ExternalEvents must remain available until a complete current-generation snapshot succeeds.

## Success criteria

- [x] Each subscribed-calendar Integration has one durable, native-owned, monotonically increasing configuration generation.
- [x] Candidate validation failure changes no secret, generation, validator, retry state, cache, or usable connection state.
- [x] Successful replacement atomically updates the encrypted secret, increments generation exactly once, clears conditional/retry/current-success state, and retains cached events.
- [x] Prepared work captures connection ID, generation, credential/config snapshot, and only validators belonging to that snapshot.
- [x] Transaction-time generation checks prevent stale success, failure, validator, retry, and authoritative reconciliation writes, including after disconnect.
- [x] The first request for a new generation is unconditional and a failed first sync leaves cached events intact with truthful current-generation failure state.
- [x] Connection-scoped cancellation is used when practical, with one current-generation follow-up requested after old work drains.
- [x] Rapid replacements and restart preserve monotonic generation and allow only the current generation to establish state.
- [x] MyTimetable and Brightspace continue using the same provider-neutral lifecycle.
- [x] Required focused and repository validation passes.

## In scope

- Shared subscribed-calendar replacement lifecycle and native credential transaction helpers.
- Integration generation persistence, repository guards, and migration 017.
- Integration Sync work preparation, completion guards, and bounded replacement follow-up scheduling.
- Shared/provider regression tests and relevant migration/documentation updates.
- Current task state, verification, self-review, and publication records.

## Allowed paths

- `src-tauri/src/ai/credentials.rs`
- `src-tauri/src/db/migrations.rs`
- `src-tauri/src/db/repositories/integrations.rs`
- `src-tauri/src/db/repositories/subscribed_calendars.rs`
- `src-tauri/src/integration_sync.rs`
- `src-tauri/src/subscribed_calendar_provider.rs`
- `src-tauri/src/my_timetable.rs`
- `src-tauri/src/brightspace.rs`
- `src-tauri/src/commands.rs` only if required to close the legacy configuration bypass
- `src-tauri/src/diagnostics.rs` only for the latest-schema expectation
- `docs/database.md`
- `docs/decisions/029-ics-subscription-ingestion.md`
- `.ai/HANDOFF.md`
- `.ai/TODO.md`
- `.ai/PROJECT_STATE.md`
- `.ai/SESSION_NOTES.md`
- `.ai/CHANGELOG.md`

## Out of scope

- `INT-SYNC-002` provider queue redesign.
- `SCHOOL-SCOPE-002` source-association hardening.
- Pulse, AI calendar tools, OAuth, richer LMS entities, generalized providers, or frontend redesign.
- New Brightspace semantic models or provider-specific race handling.
- Aether 26 or Aether 27 work.

## Architecture constraints

- Store generation canonically on the Integration row; do not expose it through public Integration IPC.
- Keep credentials and replacement mutation behind the Rust trust boundary.
- Keep migrations append-only and cached ExternalEvents intact.
- Reconcile events and finish Integration success in one transaction only after a current-generation check.
- Treat cancellation as an optimization; generation checks remain authoritative.
- Keep runtime dispatch closed to approved provider/auth pairs.

## Dependencies

- Merged CAL-ICS security hardening on `origin/master`.
- Existing DPAPI credential storage, migrations 013/016, Calendar Core, and Integration Sync runtime.

No new dependency is required.

## Risks and safeguards

- **Secret/metadata split state:** pre-encrypt, then update the SQLite secret and Integration generation/state in one transaction; rollback preserves the prior configuration.
- **Replacement races:** compare the generation captured before validation and reject a stale candidate rather than overwrite newer configuration.
- **Late old work:** check generation before any reconciliation or completion write in the same transaction.
- **Cancelled work overwriting state:** failure completion is generation-guarded and stale cancellation outcomes are discarded.
- **Replacement sync coalescing:** record one connection-scoped follow-up while old work drains; this is not a general queue redesign.
- **Sensitive diagnostics:** return fixed stale/configuration outcomes without URLs, credentials, validators, or payloads.

## Rollback considerations

Code can be reverted while the additive generation column remains harmless. Existing rows start at generation 1. Replacement transaction failure rolls back both the secret and generation/state reset, and cached events are never removed by replacement itself.

## Required validation

- Focused subscribed-calendar replacement/generation tests.
- Focused CAL-ICS regressions.
- Focused Integration Sync and Integration repository tests.
- Focused MyTimetable and Brightspace shared-lifecycle tests.
- Migration fresh/upgrade tests.
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`
- `pnpm typecheck`
- `pnpm lint`
- `pnpm build`
- `git diff --check`

No frontend test suite is required unless frontend source or contracts change.

## Independent review requirement

| Field | Value |
| --- | --- |
| Required | No |
| Reason | The repository workflow requires a distinct evidence-based self-review; no separate reviewer was requested. |
| Reviewer scope | Generation monotonicity, credential atomicity, stale completion guards, cache preservation, and provider-neutral reuse. |

## Human decisions required

None. The requested behavior and current native storage/runtime architecture determine a bounded implementation.

## Blocking decisions

None.

## Worktree / ownership gate

| Check | State |
| --- | --- |
| Correct branch/worktree confirmed | Pass — `agent/calendar-rotate` |
| `git status` inspected | Pass — clean before contract updates |
| User-owned changes identified | None |
| Parallel task overlap checked | Pass — current worktree is based on merged Aether 24 `origin/master` |
| Serialization points identified | Migration ordering, Integration generation/state, shared runtime completion |

## Readiness review

Ready. The objective, acceptance criteria, bounded paths, migration and atomicity design, stale-work guard, validation, rollback behavior, and stop condition are explicit.

## Implementation log

- Added migration `017_subscribed_calendar_generation` with private generation 1 for existing Integration rows.
- Added atomic DPAPI-secret replacement helpers and a compare-and-swap replacement transaction that advances generation, clears validators/retry/errors/prior success, and retains cached events.
- Bound request-time records and prepared outcomes to connection ID and generation; production preparation captures the decrypted credential under the same SQLite lock used to verify generation.
- Guarded runtime start, success, failure, validator persistence, and authoritative reconciliation by generation with a typed `StaleGeneration` outcome.
- Added connection-scoped cancellation and one replacement follow-up after old work drains; queued work now retains its request-time generation.
- Rejected duplicate subscribed-calendar metadata creation so the legacy configuration seam cannot act as an unvalidated replacement path.
- Updated migration/database documentation and ADR-029. No frontend source, dependency, or new ADR was added.

## Verification evidence

- Focused Integration Sync tests: 8 passed.
- Focused Integration repository tests: 6 passed.
- Focused MyTimetable tests: 22 passed.
- Focused Brightspace tests: 4 passed.
- Focused CAL-ICS tests: 21 passed.
- Focused migration tests: 15 passed.
- Focused subscribed-calendar repository tests: 1 passed.
- `cargo test --manifest-path src-tauri/Cargo.toml`: 186 passed.
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`: passed.
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`: passed.
- `pnpm typecheck`: passed.
- `pnpm lint`: passed.
- `pnpm build`: passed.
- `git diff --check`: passed.
- Full frontend tests were not required because no frontend source or contract changed.

## Acceptance evidence

- Migration/restart tests prove generation persists, existing connections start at 1, same-feed validation does not advance it, and rapid replacements advance exactly once each.
- Candidate-validation and encryption-failure tests prove the old secret, generation, validators, retry gate, connection state, and cache survive failure.
- Runtime tests prove old fetching work is cancelled and followed once, while stale parsed work cannot reconcile events, persist validators, or overwrite success/failure state.
- Replacement-state tests prove validators/origin and retry/defer/error/prior-success state clear before the first current-generation request, so it is unconditional.
- Failed first-sync coverage proves cached events remain while the current generation reports degraded failure; successful current-generation coverage proves one authoritative reconciliation and current validator persistence.
- Concurrent replacement coverage proves a slower caller cannot overwrite a newer generation; disconnect coverage proves stale work cannot recreate a removed connection.
- MyTimetable and Brightspace focused suites prove both providers retain the shared lifecycle.

## Self-review

Passed. The final diff contains only task-owned native lifecycle, persistence, provider regression tests, migration/docs, and control records. Generation remains private to Rust and canonical on the Integration row. Credential bytes, URLs, validators, and payloads do not enter public results or errors. Replacement itself never deletes cached events. The commit guard runs before reconciliation inside the same SQLite transaction, and failure writes carry the captured generation. Same-feed validation remains an ordinary refresh and does not advance generation. `INT-SYNC-002`, `SCHOOL-SCOPE-002`, Pulse, AI, frontend redesign, and provider-specific race logic were not introduced.

## Publication state

| Field | Value |
| --- | --- |
| Commit | `173b517` |
| Remote branch | `origin/agent/calendar-rotate` |
| Draft PR | [#61](https://github.com/bimberlotDEV/Aether-Desktop/pull/61) |
| Exact-head CI | Pending |

## Stop condition

Stop after all acceptance criteria are evidenced, required checks pass, the task commit is pushed, and a draft PR is open. Do not begin `INT-SYNC-002`, `SCHOOL-SCOPE-002`, Aether 26, Aether 27, Pulse, or AI work.
