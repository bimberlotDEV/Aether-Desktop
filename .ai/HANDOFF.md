# Codex Task Contract

## Contract metadata

| Field             | Value                                                    |
| ----------------- | -------------------------------------------------------- |
| Schema version    | 3                                                        |
| Task ID           | `AI-NATIVE-TOOLS-001`                                    |
| Status            | `in_progress`                                            |
| Owner             | Codex                                                    |
| Last updated      | 2026-09-26                                               |
| Related milestone | Aether 33 — native read-only AI tool foundation          |
| Classification    | `planned_codex`                                          |
| Branch / worktree | `agent/ai-native-tools` / `A:\Aether Desktop`            |

## Objective

Create a Rust-owned, closed, read-only AI tool registry for
`calendar.get_events`, `calendar.get_next_event`, `tasks.get_due`, and
`tasks.get_open`, with strict arguments, native authorization, minimized typed
outputs, deterministic sanitized errors, Sensitive privacy classification, and
hard query/result-size bounds. Do not connect tools to model execution yet.

## Context

- `origin/master` at `4da949f` contains the merged repository-health baseline,
  provider-neutral AI Router, Calendar Core, explicit School source scoping,
  Pulse, Tasks, Continuity, and Safe Actions.
- ADR-032 deliberately left `ToolScope` as a non-executing placeholder. This
  task replaces that placeholder with the first closed native authorization
  contract while preserving the existing router's no-tools behavior.
- Calendar reads must reuse the current parent-School-Space authorization and
  MyTimetable group isolation. Task reads use local normalized Tasks only.
- `school.get_deadlines` is superseded: no Deadline tool may exist until Aether
  has a reviewed normalized Deadline domain. Events, ICS text, and
  MyTimetable data are never deadline inference inputs.

## Success criteria

- [x] A closed `NativeToolId` registry contains exactly the four approved read tools and rejects unknown IDs.
- [x] Every descriptor has a stable public name, description, strict input/output schema, output version, execution type, Sensitive privacy class, required scope, and hard result limits.
- [x] A typed native `ToolScope` authorizes only closed tool IDs plus bounded Calendar/Task read grants; models cannot select connections, providers, groups, SQL, tables, URLs, or paths.
- [x] All inputs reject unknown fields, malformed values, invalid limits, inverted ranges, and windows over 31 days.
- [x] Calendar tools read only the current authorized parent School Space's persisted MyTimetable/group projection, exclude cancelled/removed events, order deterministically, and return at most 50 minimized records.
- [x] Task tools read bounded SQL projections only, exclude archived/completed Tasks, preserve deterministic due/open ordering, and return at most 50 minimized records.
- [x] Every result is typed/versioned, Sensitive by native policy, and serialized below the 64 KiB hard ceiling; failures use a closed sanitized taxonomy.
- [x] No write tool, arbitrary query/filesystem/provider path, frontend execution command, provider fetch, model tool loop, local model, academic deadline inference, or Brightspace behavior is added.
- [x] Existing AI Router behavior remains unchanged and all required focused/full validation passes.
- [ ] Task-owned work is committed, pushed, and represented by a draft PR.

## In scope

- `src-tauri/src/ai/tools/` (registry, scope, validation, projections, execution, errors, tests).
- Minimal AI module/routing changes needed to replace the empty `ToolScope` placeholder without enabling routing execution.
- Small reusable native repository projection functions for bounded School Calendar and Task reads when needed.
- ADR-033 plus current AI architecture/project records and tests.

## Allowed paths

- `src-tauri/src/ai/**`
- `src-tauri/src/db/repositories/{school_schedule,tasks}.rs`
- `src-tauri/src/db/repositories/mod.rs` only if a focused module registration is required.
- `docs/decisions/033-native-read-only-ai-tools.md`, `docs/decisions/README.md`, `docs/architecture.md`, `.ai/ARCHITECTURE.md`
- `.ai/{HANDOFF,PROJECT_STATE,TODO,SESSION_NOTES,CHANGELOG}.md`
- Test-only files directly covering the approved behavior.

## Out of scope

- Model/router tool-call execution, tool loops, local LLM/runtime work, or provider adapter changes.
- Tauri/frontend tool execution commands or a tool console.
- Write/mutation/Safe Action tools, shell/filesystem/network/provider tools, arbitrary SQL, or raw repository access.
- Academic Deadline/Assignment/Course/Material tools or inference from ICS/event content.
- Brightspace, CAL-ICS parsing/sync, MyTimetable sync, Pulse UI, Integration runtime, routing selection changes, migrations, or dependencies.

## Architecture constraints

- Native code owns tool identity, descriptors, argument parsing, authorization,
  execution, privacy metadata, output projection, and limits.
- Execution reads only the provided local SQLite connection and cannot perform
  network or recursive model work.
- Calendar authorization starts from an opaque parent School Space ID resolved
  by trusted native coordination; repository queries derive connection/group
  access from persisted bindings and never accept those identities as tool args.
- Bounds must exist in SQL/repository reads, not only after loading records.
- Existing router calls continue to use `ToolScope::none()`/default and declare
  no tool requirement.

## Dependencies

- Merged `AI-ROUTER-001`, `CAL-CORE-001`, `SCHOOL-SCOPE-002`, `PULSE-003`, and Tasks domain.
- Existing `serde`, `serde_json`, `chrono`, and `rusqlite`; no new dependency is expected.

## Risks and safeguards

- **Authorization leakage:** derive Calendar source/group access only through persisted School bindings and test identical groups across connections/Spaces.
- **Over-broad data:** select dedicated DTO columns; test serialized projections for prohibited fields.
- **Unbounded work:** enforce 31-day/50-item/64-KiB ceilings and SQL limits; reject rather than silently clamp caller values.
- **Router behavior drift:** keep execution internal and preserve default empty scope plus existing routing regression tests.
- **Error leakage:** map repository/native failures to closed codes and static sanitized messages.

## Rollback considerations

The change is additive Rust code plus documentation and a compatible typed
replacement for an unused placeholder. No migration, persisted data mutation,
frontend contract, or dependency change is planned. Reverting the task commit
restores the previous no-tools foundation.

## Required validation

- Focused registry, strict-argument, Calendar, Task, scope/privacy, projection, and size-cap tests.
- Pulse/School Calendar projection regressions and AI Router regressions.
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`
- `pnpm test`, `pnpm typecheck`, `pnpm lint`, `pnpm build`
- `git diff --check`
- Final diff review for scope escalation, arbitrary invocation, raw exposure,
  write capability, bounds, privacy drift, and deadline/Brightspace absence.

## Independent review requirement

| Field          | Value                                                                                         |
| -------------- | --------------------------------------------------------------------------------------------- |
| Required       | No                                                                                            |
| Reason         | The owner requires a distinct final self-review; no separate agent/session was requested.     |
| Reviewer scope | Closed registry, scope isolation, projections, bounds, errors, router regression, final diff. |

## Human decisions required

None at readiness. The owner supplied exact tool IDs, security/privacy policy,
bounds, exclusions, validation, publication, and stop condition.

## Blocking decisions

None.

## Worktree / ownership gate

| Check                             | State                                                                                  |
| --------------------------------- | -------------------------------------------------------------------------------------- |
| Correct branch/worktree confirmed | Pass — `agent/ai-native-tools` / `A:\Aether Desktop`                                   |
| Latest master confirmed           | Pass — fast-forwarded to merged PR #67, `origin/master` at `4da949f`                    |
| `git status` inspected            | Pass — clean before task-contract creation                                             |
| User-owned changes identified     | None                                                                                   |
| Parallel task overlap checked     | Pass — no open task owns this branch or the native AI-tool boundary                    |
| Serialization points identified   | AI routing `ToolScope`, School authorization projection, Task projection, AI ADR/docs  |

## Readiness review

Passed. The contract has a stable task ID, exact baseline, bounded objective,
observable acceptance criteria, allowed paths, explicit exclusions, dependencies,
risks, rollback, validation, publication, and stop condition. ADR-033 records the
durable native authorization/execution boundary before production implementation.

## Implementation log

- 2026-09-26: Read the owner request and mandatory control documents, verified
  PR #67 merged, fast-forwarded the clean requested branch to `origin/master`,
  classified the task as `planned_codex`, and completed the readiness gate.
- 2026-09-26: Implemented the closed registry, strict descriptors/arguments,
  typed ToolScope, bounded School Calendar and Task projections, native Sensitive
  result metadata, sanitized errors, 64 KiB enforcement, and focused tests.
- 2026-09-26: Corrected next-event behavior to exclude ongoing/all-day events
  like Pulse, bounded overdue Tasks to the native due grant, preserved explicit
  local-date semantics for all-day reads, and completed validation/self-review.

## Verification evidence

- Focused native tool foundation: 10/10 tests.
- Full native: 235/235 tests, including existing AI Router, Pulse, School,
  Calendar, MyTimetable, migrations, and Safe Actions regressions.
- Full frontend: 139/139 tests across 37 files.
- `pnpm typecheck`, `pnpm lint`, `pnpm build`, Rust formatting, strict all-target
  Clippy, and `git diff --check`: Pass.
- Migration: none. Dependency changes: none. Frontend/Tauri command changes: none.

## Self-review outcome

`complete` for implementation; publication remains. The final changed-path and
diff review found no unrelated files, write SQL/tool IDs, provider/network/filesystem
execution, frontend invoke surface, model loop, raw descriptions/source URLs/config,
unbounded query/result path, privacy downgrade, Brightspace behavior, or academic
deadline inference. Calendar authorization remains parent-School-Space plus exact
persisted connection/group joins; Task output is a dedicated body-free projection.
Every acceptance criterion above maps to focused tests or the recorded full gates.

## Stop condition

Stop after the four read tools and their native foundation meet the acceptance
criteria, validation and self-review pass, project records are updated, and a
draft PR is open. Do not begin tool-enabled AI Router integration.
