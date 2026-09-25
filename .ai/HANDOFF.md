# Codex Task Contract

## Contract metadata

| Field | Value |
| --- | --- |
| Schema version | 3 |
| Task ID | `SCHOOL-SCOPE-002` |
| Status | `complete` |
| Owner | Codex |
| Last updated | 2026-09-25 |
| Related milestone | Aether 27 — explicit School Space source scoping |
| Classification | `planned_codex` |
| Branch / worktree | `agent/school-scope-002` |

## Objective

Make each active parent School Space an explicit native-owned authorization boundary for subscribed calendar connections and per-source MyTimetable group selection, eliminating provider-wide and group-name-based cross-connection reads.

## Context

- Aether 24 through 26 are merged on `origin/master` and provide hardened subscribed-calendar ingestion, generation-safe replacement, and fair/truthful synchronization.
- The existing School read model validates a parent School Space but then lists groups, sources, and events across every `my_timetable` connection.
- Existing School Spaces persist one legacy `schoolGroup` string in `spaces.settings_json`; provider ID and group text are not ownership boundaries.
- Brightspace is a calendar-only normalized source and must not be interpreted as timetable groups, courses, assignments, or deadlines.

## Success criteria

- [x] Parent School Spaces persist explicit connection bindings with referential cleanup and independent per-source group selections.
- [x] Every School schedule read begins from the requested active parent School Space and can return events only from its associated connection IDs.
- [x] No association or no valid group selection returns zero timetable events with a truthful setup/reselection state; there is no provider-wide or all-group fallback.
- [x] MyTimetable group discovery and validation are scoped to one associated connection, including when two connections expose the same group string.
- [x] Multiple parent School Spaces can independently bind different or shared sources and update source/group configuration without mutating one another.
- [x] Disabled associations remain visible with truthful cached/stale state, deletion removes bindings through foreign keys, and same-ID feed replacement preserves bindings.
- [x] Brightspace can be explicitly associated and reports truthful source state but contributes no groups and no events to the timetable views.
- [x] Existing legacy selections migrate only when exactly one MyTimetable connection exists; multiple candidates remain unassociated and require explicit choice.
- [x] Subject child and non-School Spaces cannot own or query School source scope.
- [x] The public School IPC remains minimal and exposes no URL, validator, credential, raw payload, configuration generation, or unrelated group list.
- [x] Required focused and full validation passes.

## In scope

- Additive normalized School source/group persistence and migration 018.
- Rust School source configuration, scoped read model, lifecycle behavior, and repository/migration tests.
- Typed Tauri commands and minimal TypeScript models/wrappers.
- Focused School setup UI for source association and per-MyTimetable-source group selection.
- School frontend tests and durable architecture/database/task records.

## Allowed paths

- `src-tauri/src/db/migrations.rs`
- `src-tauri/src/db/repositories/school_schedule.rs`
- `src-tauri/src/commands.rs`
- `src-tauri/src/lib.rs`
- `src-tauri/src/diagnostics.rs` only for the latest-schema expectation
- `src/lib/db/types.ts`
- `src/lib/db/tauri.ts`
- `src/lib/db/tauri.test.ts`
- `src/hooks/useSchoolSchedule.ts`
- `src/components/school/SchoolSchedule.tsx`
- `src/components/school/SchoolSchedule.test.tsx`
- `src/lib/school/schedule.ts` and tests only if source-state presentation requires a focused adjustment
- `src-tauri/src/my_timetable.rs` and `src-tauri/src/brightspace.rs` only for lifecycle regression tests if repository coverage is insufficient
- `docs/database.md`
- `docs/decisions/031-school-source-scoping.md`
- `.ai/ARCHITECTURE.md`
- `.ai/HANDOFF.md`
- `.ai/TODO.md`
- `.ai/PROJECT_STATE.md`
- `.ai/SESSION_NOTES.md`
- `.ai/CHANGELOG.md`

## Out of scope

- Pulse, AI Router, AI calendar tools, Safe Actions, OAuth, courses, assignments, deadlines, or richer LMS semantics.
- Integration Sync scheduling, CAL-ICS transport/parsing, subscribed-calendar replacement, credentials, or generic Calendar Core identity.
- Injecting Brightspace general events into timetable Today/Week/Upcoming views.
- School visual redesign beyond focused source/group setup and truthful states.
- A generalized plugin schema or provider-defined School settings.

## Architecture constraints

- Connection ID is the authoritative ownership boundary; provider ID is validation/presentation metadata only and is not duplicated in the binding table.
- Use normalized `school_space_sources` and `school_space_source_groups` tables with explicit cascading foreign keys.
- The group table is keyed by `(school_space_id, connection_id, group_reference)` so group text is meaningful only inside one bound source and the schema can support multiple selected groups later.
- Query authorization is derived entirely in Rust from `school_space_id`; the schedule request accepts no provider, connection, or group authority.
- Configuration mutations validate an active top-level School Space and an existing supported subscribed-calendar connection.
- Keep cached ExternalEvents untouched; association removal changes only School ownership/configuration.
- Retain current local-time, all-day, cancellation, overlap, stale/cache, and bounded-view behavior.

## Dependencies

- Merged Aether 24 CAL-ICS hardening, Aether 25 generation-safe replacement, and Aether 26 Integration Sync hardening.
- Existing Spaces, Integration, subscribed-calendar, ExternalEvent, School timetable, and typed IPC layers.
- Accepted ADR-031.
- No new dependency.

## Risks and safeguards

- **Cross-connection leakage:** every event/group query joins the requested Space's persisted connection binding before considering provider or group text.
- **Ambiguous legacy ownership:** migration binds only when the global MyTimetable candidate count is exactly one; zero or multiple candidates produce no binding.
- **Stale group after source change:** removing a binding cascades its selected groups; adding another source starts with no selected group.
- **Deleted or disabled source:** connection deletion cascades binding/event rows; disabled rows remain associated and visible without alternate-source fallback.
- **Brightspace semantic inflation:** it may be associated and shown as a calendar source, but is excluded from group discovery and timetable event queries.
- **Broad IPC exposure:** return only source identity/provider/display/status, association state, selected groups, scoped options, and the existing event projection; add no secret or sync-internal field.

## Rollback considerations

Code and UI changes are reversible on the task branch. Migration 018 is append-only; its binding/group rows are isolated metadata and cascade with their owning Space or Integration. Cached ExternalEvents and credentials are not migrated or deleted. Earlier code safely ignores the new tables, though source ownership configured after upgrade would not be enforced by an older binary.

## Required validation

- Focused School repository source-scoping, same-group/different-connection, multiple-Space, lifecycle, child/non-School, and persistence tests.
- Focused migration fresh/upgrade tests for unique and ambiguous legacy MyTimetable candidates.
- Focused MyTimetable and Brightspace tests.
- Focused School frontend and typed IPC tests.
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`
- `pnpm typecheck`
- `pnpm lint`
- `pnpm test`
- `pnpm build`
- `git diff --check`
- Focused desktop smoke steps recorded; automated native behavior remains the authoritative evidence if an interactive desktop session is unavailable.

## Independent review requirement

| Field | Value |
| --- | --- |
| Required | No |
| Reason | The repository workflow requires a distinct evidence-based self-review; no separate reviewer was requested. |
| Reviewer scope | Connection-bound authorization, migration ambiguity, lifecycle cleanup, Brightspace semantics, IPC minimization, and multi-Space isolation. |

## Human decisions required

None. The request explicitly chooses connection identity as the ownership boundary and permits deterministic one-candidate migration.

## Blocking decisions

None.

## Worktree / ownership gate

| Check | State |
| --- | --- |
| Correct branch/worktree confirmed | Pass — `agent/school-scope-002` fast-forwarded to merged Aether 26 `origin/master` |
| `git status` inspected | Pass — clean before contract/ADR updates |
| User-owned changes identified | None |
| Parallel task overlap checked | Pass — Aether 24–26 are merged; this branch owns migration 018 and School IPC/read model |
| Serialization points identified | Migration ordering, School IPC contract, School source configuration, ADR-031 |

## Readiness review

Passed. The ownership model, schema, migration ambiguity rule, Brightspace boundary, IPC authority, UI scope, lifecycle semantics, rollback, validation, and stop condition are explicit. Implementation is in progress.

## Implementation log

- 2026-09-25: Fetched and fast-forwarded the clean task branch to merged Aether 26.
- 2026-09-25: Inspected the School repository/UI, Spaces hierarchy/settings, Integration/subscribed-calendar lifecycle, ExternalEvents, migrations 012–017, provider connectors, and ADRs 028–030.
- 2026-09-25: Accepted ADR-031 and completed the readiness gate.
- Added migration 018, connection-bound repository reads/mutations, typed IPC, and focused source/group setup UI.
- Added deterministic isolation, lifecycle, migration, restart, provider, and frontend regression coverage.
- Completed full validation and a distinct final-diff/security/scope self-review.
- Published implementation commit `508866f` to `agent/school-scope-002` and opened draft PR #63.

## Verification evidence

- Focused School repository tests: 13 passed.
- Focused School source migration tests: 2 passed.
- Focused MyTimetable tests: 22 passed.
- Focused Brightspace tests: 4 passed.
- Focused School/frontend IPC tests: 34 passed across 3 files.
- `cargo test --manifest-path src-tauri/Cargo.toml`: 202 passed.
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`: passed.
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`: passed.
- `pnpm typecheck`: passed.
- `pnpm lint`: passed.
- `pnpm test`: 132 passed across 37 files.
- `pnpm build`: passed.
- `git diff --check`: passed.

## Acceptance evidence

- AC1/AC2/AC3: migration tables plus the binding-first event query; no-association and invalid-selection tests return zero events.
- AC4/AC5: same-group/different-connection, multiple-MTT-source, two-Space, source-change, and group-change tests prove identity and configuration isolation.
- AC6: disabled, disconnected, deleted, replacement-generation, and restart tests prove lifecycle semantics and persistence.
- AC7: combined Brightspace/MyTimetable test proves Brightspace has no groups and contributes no timetable event.
- AC8: unique-candidate and ambiguous-candidate upgrade tests prove deterministic fail-closed migration.
- AC9: non-School and subject-child tests reject read and configuration ownership.
- AC10: the School source schema exposes bounded identity/presentation/status/group fields only; UI tests verify no feed URL text.
- AC11: all focused and repository gates above pass.

## Self-review

Passed. The changed-path list is task-owned and contains migration 018, the School repository/IPC/UI, necessary diagnostics and documentation, and task records only. The schedule read accepts only `school_space_id` and a bounded range; connection and group authority are derived in Rust. Every group lookup is connection-scoped and every event joins the Space binding plus that connection's selected group. Brightspace is excluded from timetable events. Disabled cached behavior is preserved; disconnected/deleted sources cannot return events; foreign keys clean up ownership; replacement retains the same connection ID. No credential, feed URL, validator, raw payload, configuration generation, or new dependency crosses the School IPC. Pulse, AI, richer LMS entities, sync scheduling, CAL-ICS, and credential storage remain untouched.

## Publication state

| Field | Value |
| --- | --- |
| Commit | `508866f` (`fix(school): scope schedules to explicit sources`) |
| Remote branch | `origin/agent/school-scope-002` |
| Draft PR | [#63](https://github.com/bimberlotDEV/Aether-Desktop/pull/63) |
| Exact-head CI | Pending after publication; local required validation passed. |

## Stop condition

Stop after all acceptance criteria are evidenced, required checks pass, required records are updated, the implementation is committed and pushed on `agent/school-scope-002`, and an Aether 27 draft PR is open. Do not begin Pulse, AI Router, AI calendar, or richer School/LMS work.
