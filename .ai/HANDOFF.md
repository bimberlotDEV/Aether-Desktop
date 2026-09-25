# Codex Task Contract

## Contract metadata

| Field | Value |
| --- | --- |
| Schema version | 3 |
| Task ID | `INT-SYNC-002` |
| Status | `complete` |
| Owner | Codex |
| Last updated | 2026-09-25 |
| Related milestone | Aether 26 — Integration Sync Runtime hardening |
| Classification | `planned_codex` |
| Branch / worktree | `agent/integration-sync-002` |

## Objective

Guarantee fair same-provider synchronization and truthful terminal state for every started Integration sync without weakening generation safety or cached-data atomicity.

## Context

- MyTimetable and Brightspace share the native provider-neutral Integration Sync runtime.
- ADR-030 intentionally allows one active connection per provider, but the current runtime discards a distinct same-provider request as `Coalesced`.
- A successful fetch/parse followed by a reconciliation or final Integration commit error currently leaves the connection in `syncing`.
- Aether 25 introduced durable configuration generations and transaction-time stale-work rejection that this task must preserve.

## Success criteria

- [ ] Same-connection requests coalesce while distinct same-provider connections enter one bounded FIFO and progress without another scheduler pass.
- [ ] FIFO order is stable under repeated requests, stale/ineligible entries are skipped, and different providers remain isolated under the global semaphore.
- [ ] Public request outcomes distinguish accepted/start-reserved, queued, same-connection coalesced, retry-deferred, and rejected work.
- [ ] Queued entries capture generation and cannot become valid after replacement; replacement follow-up schedules the current generation exactly once.
- [ ] Disconnect/disable removes queued work, running cancellation cannot commit late, and shutdown clears queues and terminalizes started work.
- [ ] Every run whose running state was persisted emits exactly one terminal event and follows one truthful success/failure/stale/ineligible terminal path.
- [ ] Commit/reconciliation failure rolls back the transaction, preserves cached events and validators, records sanitized `local_commit` failure metadata, and leaves the connection retryable.
- [ ] Startup, periodic, resume, manual, replacement follow-up, retry, MyTimetable, Brightspace, and generation regressions pass.
- [ ] Required focused and repository validation passes.

## In scope

- Integration Sync scheduling state, provider FIFO dispatch, lifecycle cancellation/shutdown, terminalization, request results, and runtime tests.
- Minimal Integration repository completion guard changes required for disabled/deleted terminal truth.
- Minimal frontend request-result schema extension.
- ADR-030 clarification and current task records.

## Allowed paths

- `src-tauri/src/integration_sync.rs`
- `src-tauri/src/db/repositories/integrations.rs`
- `src-tauri/src/my_timetable.rs` only if a shared-lifecycle regression requires correction
- `src-tauri/src/brightspace.rs` only if a shared-lifecycle regression requires correction
- `src/lib/db/types.ts`
- `src/lib/db/tauri.test.ts`
- `docs/decisions/030-integration-sync-runtime.md`
- `.ai/HANDOFF.md`
- `.ai/TODO.md`
- `.ai/PROJECT_STATE.md`
- `.ai/SESSION_NOTES.md`
- `.ai/CHANGELOG.md`

## Out of scope

- School Space source scoping and `SCHOOL-SCOPE-002`.
- Pulse, AI Router, AI calendar tools, Calendar Core semantic changes, provider-specific ingestion redesign, OAuth, or new integrations.
- Broad scheduler frameworks, priority scheduling, background services, or queue-management UI.

## Architecture constraints

- Retain one active connection per provider and the existing global semaphore.
- Use a deterministic in-memory FIFO per provider with at most one queued/running entry per connection.
- Capture configuration generation at enqueue and revalidate current state before dispatch and commit.
- Keep credentials, validators, provider payloads, and low-level database errors behind Rust.
- Reconcile Calendar data and Integration success metadata in one transaction; rollback must preserve the prior snapshot.
- Queue progress must be driven by terminal cleanup, not app focus or a later periodic scan.

## Dependencies

- Merged `CAL-SUB-ROTATE-001` / Aether 25 generation safeguards.
- Existing Integration repository, CAL-ICS engine, shared subscribed-calendar providers, and ADR-027 through ADR-030.
- No new dependency or migration.

## Risks and safeguards

- **Queue starvation:** strict provider FIFO; duplicate active/queued requests never append or reorder an entry.
- **Stale queued work:** store request-time generation and reject mismatches before marking running.
- **Commit rollback followed by stuck state:** route commit errors to a fixed `local_commit` failure through the normal repository failure path.
- **Cancellation races:** cancel running tokens, remove queued entries synchronously, and retain generation checks as the authoritative commit guard.
- **Shutdown zombies:** clear ephemeral queues, cancellation-select semaphore waiters, drain active tasks, and persist terminal cancellation/interruption state.
- **Sensitive errors:** expose only fixed request reasons and bounded failure codes/messages.

## Rollback considerations

The implementation and IPC enum extension are reversible on the task branch. Queue state is ephemeral and never migrated. Existing persisted connections, cached events, validators, and generations remain compatible. Failed authoritative transactions roll back before terminal failure metadata is written separately.

## Required validation

- Focused Integration Sync scheduling, fairness, cancellation, generation, and commit-failure tests.
- Focused Integration repository tests.
- Focused subscribed-calendar, MyTimetable, Brightspace, and Aether 25 generation tests.
- Frontend contract tests because the request-result schema changes.
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`
- `pnpm typecheck`
- `pnpm lint`
- `pnpm test`
- `pnpm build`
- `git diff --check`

## Independent review requirement

| Field | Value |
| --- | --- |
| Required | No |
| Reason | The repository workflow requires a distinct evidence-based self-review; no separate reviewer was requested. |
| Reviewer scope | FIFO fairness, terminalization, rollback, cancellation, generation safety, and provider isolation. |

## Human decisions required

None. The requested FIFO behavior and existing ADR-030 serialization boundary determine the design.

## Blocking decisions

None.

## Worktree / ownership gate

| Check | State |
| --- | --- |
| Correct branch/worktree confirmed | Pass — `agent/integration-sync-002` rebased to merged Aether 25 `origin/master` |
| `git status` inspected | Pass — clean before contract update |
| User-owned changes identified | None |
| Parallel task overlap checked | Pass — Aether 25 is merged; this branch owns the shared runtime serialization point |
| Serialization points identified | Integration runtime state, terminal repository updates, request-result IPC contract, ADR-030 |

## Readiness review

Passed. The bounded objective, FIFO design, generation handling, terminal guarantee, failure mapping, scope, rollback, validation, and publication stop condition are explicit; implementation is in progress.

## Implementation log

- 2026-09-25: Rebased the clean task branch to merged Aether 25 and completed the readiness gate.
- Added one bounded FIFO per provider while preserving the global semaphore and per-connection single-flight.
- Added generation/trigger/current-eligibility revalidation at dequeue, queue removal on disconnect/disable, and current-generation replacement follow-up handling.
- Routed every persisted start through one terminal event; commit errors now record sanitized retryable `local_commit` after transaction rollback.
- Added `Queued` to the typed request outcome and made lifecycle scheduling match the exact declared trigger.
- Updated ADR-030. No migration, dependency, provider ingestion change, or broad UI change was added.

## Verification evidence

- Focused Integration Sync tests: 17 passed.
- Focused Integration repository tests: 7 passed.
- Focused MyTimetable tests: 22 passed.
- Focused Brightspace tests: 4 passed.
- Focused CAL-ICS tests: 21 passed.
- Focused subscribed-calendar tests: 2 passed.
- Focused generation tests: 9 passed.
- `cargo test --manifest-path src-tauri/Cargo.toml`: 196 passed.
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`: passed.
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`: passed.
- `pnpm typecheck`: passed.
- `pnpm lint`: passed.
- `pnpm test`: 132 passed across 37 files.
- `pnpm build`: passed.
- `git diff --check`: passed.

## Acceptance evidence

- FIFO tests prove A → B → C order, repeated A requests cannot move ahead of B/C, and explicit MyTimetable A/B and Brightspace A/B pairs queue rather than disappear.
- Cross-provider tests prove MyTimetable and Brightspace can occupy separate global slots without sharing queue state.
- Request tests prove `Accepted`, `Queued`, `Coalesced`, `Deferred`, and `Rejected` remain distinct and the public schema accepts `queued`.
- Disconnect/disable tests prove queued work is removed and late running completion cannot overwrite disabled idle state.
- Replacement tests prove stale queued generation is replaced by one current-generation request and existing running-generation safeguards remain green.
- Fetch/parse-style failure, cancellation, rate-limit, and commit-failure tests prove the next FIFO entry progresses.
- SQLite failure injection proves authoritative reconciliation, validators, and success timestamps roll back before bounded `local_commit` failure is persisted.
- Shutdown tests prove running work is cancelled, terminalized once, leaves no persisted `syncing` state, and rejects later requests.
- Trigger tests prove startup, resume, and periodic scans select only the matching declared mode.

## Self-review

Passed. The complete diff is limited to the shared runtime, narrow Integration terminal guards, one typed IPC enum extension/test, ADR-030, and required control records. Provider serialization and the global concurrency cap remain intact. Each connection appears at most once across active/queued state, queue handoff is FIFO and terminal-driven, and stale/ineligible entries are skipped without blocking later work. Generation remains native/private and is checked at enqueue/dequeue/commit boundaries. Commit errors are sanitized before persistence/IPC, authoritative data and validators roll back atomically, and no secret/provider payload is exposed. Disabled/deleted connections reject late writes. Every path after a successful `mark_running` emits one terminal event. No migration, dependency, School scoping, Pulse, AI, Calendar semantic, or provider ingestion work entered scope.

## Publication state

| Field | Value |
| --- | --- |
| Commit | `e4d0baf` |
| Remote branch | `origin/agent/integration-sync-002` |
| Draft PR | [#62](https://github.com/bimberlotDEV/Aether-Desktop/pull/62) |
| Exact-head CI | Pending |

## Stop condition

Stop after all acceptance criteria are evidenced, required checks pass, the implementation is committed and pushed on `agent/integration-sync-002`, and an Aether 26 draft PR is open. Do not begin `SCHOOL-SCOPE-002`, Aether 27, Pulse, AI Router, or AI calendar work.
