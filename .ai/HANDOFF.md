# Codex Task Contract

## Contract metadata

| Field | Value |
| --- | --- |
| Schema version | 3 |
| Task ID | `PULSE-003` |
| Status | `complete` |
| Owner | Codex |
| Last updated | 2026-09-26 |
| Related milestone | Aether 31 — connected Pulse 2.0 |
| Classification | `planned_codex` |
| Branch / worktree | `agent/pulse-003` |

## Objective

Extend Pulse into one coherent, bounded, local-first native snapshot covering current and next schedule, today and near-term events, open due Tasks, conflicts, Continuity, and truthful source trust without provider fetching, frontend aggregation, inferred academic deadlines, or broadened School authorization.

## Context

- Pulse already uses one Rust-owned `get_pulse` command for deterministic local Tasks, Spaces, Source metadata, Activity, and a suggested next step.
- Calendar Core persists normalized timed/all-day occurrences. MyTimetable is the sole active School calendar provider.
- School authorization is connection-bound through `school_space_sources` plus selected groups; provider/group labels are not authority.
- Brightspace is intentionally retired. No course, assignment, or academic Deadline domain exists.
- The approved request requires minimized presentation projections, one captured logical `now`, bounded native queries, section-level degradation, and a focused desktop-first UI.

## Success criteria

- [x] One read-only `get_pulse` invoke returns the complete bounded snapshot; React does not assemble Calendar, School, Tasks, Integrations, or Continuity independently.
- [x] Native `now`, `next`, `today`, and seven-day `upcoming` event projections obey half-open interval, cancellation, all-day, deterministic ordering, and hard-cap rules.
- [x] Native Tasks projection contains only open overdue/due-today/due-soon Tasks, deterministically ordered and capped at 20; no academic deadline inference exists.
- [x] Timed conflicts exclude cancelled/all-day/self/touching items, de-duplicate pairs, order deterministically, and cap at 10.
- [x] Continuity is a safe bounded projection capped at 5 and does not prevent the rest of Pulse from loading when empty or unavailable.
- [x] Trust is computed in Rust from relevant authorized Integration state, uses one centralized two-hour freshness policy, exposes only sanitized status/freshness fields, and labels cached disabled/disconnected/degraded data truthfully.
- [x] MyTimetable rows enter Pulse only through persisted School Space connection/group authorization; isolation tests prove no cross-Space/connection/group leakage.
- [x] Calendar and Integration projections omit descriptions, provider payloads/config, URLs, credentials, hashes, retry internals, and raw errors.
- [x] Pulse performs no provider network work; Brightspace, AI calendar tools, and new Deadline/Assignment persistence remain absent.
- [x] Focused and full required validation passes, final-diff review is complete, records are updated, and a draft PR is open.

## In scope

- Rust Pulse snapshot/service repository and focused reusable minimized event/trust projection modules where justified.
- Existing `get_pulse` command/registration only as needed for the evolved response.
- Strict frontend Pulse schemas, invoke parsing, hook, route/components, semantic styles, and tests.
- Focused Calendar/School/Integration/Tasks/Continuity/Pulse regression tests.
- Pulse architecture/current-state/task documentation and one ADR update/new ADR if the durable native projection boundary warrants it.

## Allowed paths

- `src-tauri/src/db/repositories/pulse.rs`
- New focused modules under `src-tauri/src/db/repositories/` for minimized Pulse projections if needed
- `src-tauri/src/db/repositories/mod.rs`
- `src-tauri/src/commands.rs`, `src-tauri/src/lib.rs` only if the command contract requires registration changes
- `src/lib/db/types.ts`, `src/lib/db/tauri.ts` and focused tests
- `src/hooks/usePulse.ts` and focused tests if added
- `src/routes/Pulse.tsx`, `src/routes/Pulse.test.tsx`
- `src/styles/index.css` using existing semantic tokens/patterns
- `docs/decisions/019-deterministic-pulse.md` or one successor ADR
- `.ai/HANDOFF.md`, `.ai/TODO.md`, `.ai/PROJECT_STATE.md`, `.ai/SESSION_NOTES.md`, `.ai/CHANGELOG.md`

## Out of scope

- Provider network requests, sync scheduler/parser/fetcher changes, MyTimetable connector changes, credential handling, Calendar reconciliation, or generic Integration lifecycle changes.
- Brightspace reintroduction, OAuth, another provider, Course/Assignment/Deadline/CourseContent persistence, or academic-deadline inference from calendar text.
- AI tools, AI Router changes, Safe Actions, task mutations, calendar mutations, broad tracking, or raw Activity exposure.
- Fuzzy event merging, title/description heuristics, speculative moved-event history, or arbitrary frontend source/connection/group scope.
- Database migrations and new dependencies unless implementation evidence proves one is unavoidable and the contract is revised before proceeding.

## Architecture constraints

- Capture a single UTC instant and derived local date/window at snapshot start.
- Preserve half-open semantics: active `start <= now < end`; overlap `A.start < B.end && B.start < A.end`.
- MyTimetable authorization is resolved only by native joins through active parent School Spaces, bound connection IDs, and selected groups. No provider-wide fallback.
- Minimized event projection contains only local ID, safe source identity/label, title, timing/all-day fields, optional location, cancellation state, and proven status.
- Minimized trust projection contains only safe source identity/label, enabled/connection/sync presentation, useful sync timestamps, sanitized category, freshness, and cached-data truth.
- Native SQL/service logic enforces caps: Today 30, Upcoming 30, Tasks 20, Conflicts 10, Continuity 5, relevant trust sources only.
- Optional section failures become sanitized degraded section state where safe; no SQL/provider error reaches React.
- No persistence change is expected. Migration: none.

## Dependencies

- `PULSE-002`, `CONT-001`, `CAL-CORE-001`, `SCHOOL-MTT-001`, `SCHOOL-SPACE-001`, `SCHOOL-SCOPE-002`, `INT-CORE-001`, and `SCHOOL-BSP-REMOVE-001` are complete.
- No new dependency.

## Risks and safeguards

- **School data leakage:** authorize MyTimetable with explicit connection and group joins; test two Spaces, two connections, and identical group labels.
- **Privacy regression:** use dedicated serialized projections rather than `ExternalEvent` or `Integration`; serialization tests reject forbidden fields.
- **Time errors:** inject/capture one instant in tests and cover exact boundaries, midnight, local-date grouping, touching intervals, and DST-capable chrono conversions.
- **Unbounded overview reads:** hard limits live in SQL/service logic; conflict candidates use a bounded seven-day event set.
- **Misleading cache state:** trust remains visible for relevant bound sources and explicitly distinguishes disabled, disconnected, degraded, syncing, never-synced, stale, and fresh.
- **Broad failure:** section queries are isolated and return bounded sanitized degradation metadata while independent sections continue.

## Rollback considerations

The change is read-only and adds no migration or dependency. Rollback restores the prior Pulse response/UI with no data cleanup. Existing Calendar, School, Tasks, Integration, and Continuity persistence remains unchanged.

## Required validation

- Focused Rust Pulse/event projection/School isolation/Integration trust/Tasks/Continuity tests.
- Focused frontend Pulse, hook/IPC/schema tests.
- Existing School/MyTimetable regression tests.
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`
- `pnpm typecheck`
- `pnpm lint`
- `pnpm test`
- `pnpm build`
- `git diff --check`
- Manual smoke plan recorded for owner execution.

## Independent review requirement

| Field | Value |
| --- | --- |
| Required | No |
| Reason | The owner requested implementation and publication; repository workflow requires a distinct evidence-based self-review but not a separate agent/session. |
| Reviewer scope | Authorization, minimization, time semantics, bounds, trust truthfulness, partial degradation, Brightspace absence, and no provider fetching. |

## Human decisions required

None. The supplied task specifies the architecture, semantics, caps, exclusions, validation, and publication outcome.

## Blocking decisions

None.

## Worktree / ownership gate

| Check | State |
| --- | --- |
| Correct branch/worktree confirmed | Pass — stale missing worktree registration pruned; clean `agent/pulse-003` at `origin/master` commit `64f572e` |
| Latest master confirmed | Pass — fetched `origin/master`; worktree HEAD is identical |
| `git status` inspected | Pass — clean before contract updates |
| User-owned changes identified | None |
| Parallel task overlap checked | Pass — requested branch had only a prunable missing checkout; no active Pulse implementation work found |
| Serialization points identified | Pulse IPC/type contract, native read model, School authorization joins, semantic Pulse styles, current-state docs |

## Readiness review

Passed. Goal, observable behavior, minimized projections, School authority, time/conflict/task/trust/continuity semantics, caps, partial failure, privacy exclusions, validation, rollback, documentation, publication, and stop condition are explicit. Production implementation may begin.

## Implementation log

- 2026-09-26: Read the approved request and mandatory control documents; fetched `origin/master`, confirmed exact base `64f572e`, pruned a stale missing worktree registration, and attached the clean requested branch.
- 2026-09-26: Inspected Pulse/Continuity/Calendar/Integration/School architecture and current implementation; confirmed no migration or new dependency is required.
- 2026-09-26: Implemented the minimized one-command snapshot, focused Pulse UI, strict frontend validation, source trust, conflict detection, section degradation, and regression coverage.
- 2026-09-26: Completed focused/full validation and the authorization, minimization, bounds, provider-fetch, Brightspace, and scope self-review.

## Verification evidence

- Native Pulse tests: 11 passed.
- Focused frontend Pulse/typed IPC tests: 32 passed across 2 files.
- Existing School source isolation tests: 13 passed.
- Existing MyTimetable-filtered tests: 23 passed.
- `cargo test --manifest-path src-tauri/Cargo.toml`: 225 passed.
- `pnpm test`: 140 passed across 37 files.
- Typecheck, lint, production build, Rust formatting, strict Clippy, and `git diff --check`: passed.

## Acceptance evidence

- AC1: `get_pulse` remains the single no-argument invoke and its strict response contains every rendered section.
- AC2: one captured native clock drives local-day UTC boundaries, Now/Next, seven-day windows, and DST-aware tests.
- AC3: SQL caps and stable ordering enforce Today 30, Upcoming 30, Tasks 20, Continuity 5; conflict candidates are bounded at 128 and results at 10.
- AC4: MyTimetable SQL joins explicit active parent School bindings, selected groups, and event groups; identical unauthorized group data is excluded.
- AC5: serialized projection tests prove broad Calendar/Integration fields are absent; source errors are categorized and section failures are sanitized.
- AC6: cached disabled/disconnected/degraded/stale data remains truthful through native trust state; freshness uses one two-hour constant.
- AC7: academic deadlines remain explicitly unavailable; no migration, network path, AI tool, Brightspace behavior, or dependency was added.

## Self-review

Passed. The final diff is limited to the Pulse repository/read model, strict Pulse IPC response parsing, focused route/styles/tests, the existing Pulse ADR extension, and required task records. Event authorization is native and connection/group-bound for MyTimetable. All reads are bounded; cancellation, all-day, active, next, overlap, touching, cross-midnight, local-day, and DST behavior are covered. The response exposes no description, URL, credential, provider configuration, hash, raw error, or provider payload. `pulse.rs` contains no fetch/runtime dispatch. Brightspace appears only in the explicit retired-provider SQL exclusion and documentation assertions. No unrelated persistence, sync, AI, Safe Actions, routing, or provider behavior changed.

## Publication state

Published implementation commit `96af188` (`feat(pulse): add connected native snapshot`) to `origin/agent/pulse-003`; draft PR [#66](https://github.com/bimberlotDEV/Aether-Desktop/pull/66) is open. The post-commit hook could not fork its helper process, but `scripts/publish-task.ps1` completed the explicit push and draft-PR creation successfully.

## Stop condition

Stop when acceptance criteria are evidenced, all required checks pass, the final diff has been reviewed for authorization/minimization/bounds/network/Brightspace regressions, records are updated, commits are pushed on `agent/pulse-003`, and an Aether 31 draft PR is open. Do not begin AI-CAL, tool-enabled AI Router, academic Deadline persistence, or another integration.
