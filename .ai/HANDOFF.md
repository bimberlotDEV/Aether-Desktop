# Codex Task Contract

## Contract metadata

| Field | Value |
| --- | --- |
| Schema version | 3 |
| Task ID | `SCHOOL-BSP-REMOVE-001` |
| Status | `self_review` |
| Owner | Codex |
| Last updated | 2026-09-25 |
| Related milestone | Aether 30 — remove the Brightspace integration |
| Classification | `planned_codex` |
| Branch / worktree | `agent/remove-brightspace` |

## Objective

Remove Brightspace as an advertised, connectable, synchronizable, or School-associated provider while preserving provider-neutral subscribed-calendar infrastructure and allowing legacy Brightspace rows and credentials to be safely inspected and removed.

## Context

- The shipped Brightspace connector exposes only renewable ICS calendar data, while the intended product requires rich courses, assignments, deadlines, and materials.
- Rich Brightspace APIs require institution-managed application authorization and are not presently a universal self-service integration.
- MyTimetable, CAL-ICS, subscribed-calendar replacement, DPAPI secrets, Integration Sync, ExternalEvent, Calendar Core, and normalized School source scoping are shared infrastructure and remain authoritative.
- Existing databases may contain `provider_id = 'brightspace'`, encrypted feed credentials, cached ExternalEvents, and School source bindings.

## Success criteria

- [x] Brightspace is absent from provider catalog/setup UI, typed setup wrappers, native provider commands, and active runtime dispatch.
- [x] Generic creation rejects the retired Brightspace provider ID, while MyTimetable setup and refresh remain operational.
- [x] Legacy Brightspace rows remain readable without startup failure, never dispatch sync, and can be removed through a provider-neutral unsupported-connection cleanup path that removes credentials and cascaded data.
- [x] School source discovery and association accept only MyTimetable; legacy Brightspace bindings/events never enter School timetable presentation or event queries.
- [x] Shared CAL-ICS, subscribed-calendar, DPAPI, redirect/SSRF, validator, replacement-generation, Integration Sync, ExternalEvent, Calendar Core, and `school_space_sources` infrastructure remain intact.
- [x] Current-state documentation records intentional removal and parked rich support; historical changelog and ADR records remain historical.
- [x] Required focused and full validation passes, followed by final diff review; draft PR publication is the remaining mechanical step.

## In scope

- Provider catalog, Connections setup/action presentation, frontend wrappers/types/tests.
- Native Brightspace module/command removal and closed runtime dispatch update.
- Small provider-neutral legacy cleanup command/service plus creation denylist for the retired ID.
- School source query/association restriction and regression tests.
- Current-state/task documentation and truthful retirement notes.

## Allowed paths

- `src-tauri/src/brightspace.rs` (delete)
- `src-tauri/src/lib.rs`
- `src-tauri/src/commands.rs`
- `src-tauri/src/integration_sync.rs`
- `src-tauri/src/subscribed_calendar_provider.rs`
- `src-tauri/src/db/repositories/integrations.rs`
- `src-tauri/src/db/repositories/school_schedule.rs`
- `src/hooks/useConnections.ts`
- `src/lib/db/tauri.ts`
- `src/lib/db/types.ts`
- `src/lib/integrations/presentation.ts` and focused tests
- `src/components/connections/ConnectionsSettings.tsx` and focused tests
- `src/components/school/SchoolSchedule.tsx` and focused tests
- `docs/database.md`
- `docs/decisions/031-school-source-scoping.md` only to mark the historical Brightspace clause superseded by this task
- `.ai/ARCHITECTURE.md`, `.ai/HANDOFF.md`, `.ai/TODO.md`, `.ai/PROJECT_STATE.md`, `.ai/SESSION_NOTES.md`, `.ai/CHANGELOG.md`

## Out of scope

- Changes to MyTimetable behavior, generic calendar ingestion, Calendar Core identity, sync scheduling semantics, DPAPI, redirect/SSRF policy, validators, or replacement generations.
- Database migration or automatic deletion of legacy rows, credentials, cached events, or historical records.
- A replacement LMS connector, OAuth, courses, assignments, deadlines, materials, Pulse, AI, or Safe Actions.
- Rewriting historical changelog entries, historical ADR rationale, Git history, or unrelated UI.

## Architecture constraints

- `my_timetable` remains the sole active School timetable provider.
- Runtime provider dispatch stays closed; a persisted unregistered provider is rejected before work starts.
- Legacy cleanup is provider-neutral and limited to unsupported subscribed-calendar records; it cancels runtime work, deletes the native secret, and then deletes the owning Integration so existing foreign-key lifecycle cleanup applies.
- The public Integration schema continues to accept string provider IDs so legacy/unknown rows remain readable.
- No migration: retention until explicit user cleanup is safer and smaller than mutating or deleting sensitive legacy state during upgrade.

## Dependencies

- Existing Integration Core, CAL-ICS, subscribed-calendar provider, DPAPI credential repository, Integration Sync Runtime, Calendar Core, MyTimetable, and School source scoping.
- No new dependency.

## Risks and safeguards

- **Legacy credential orphaning:** cleanup resolves the existing private credential key, removes it first, and deletes the Integration only after secret removal succeeds.
- **Accidental legacy sync:** runtime handler registration excludes the retired provider and focused startup/manual-request tests prove rejection.
- **School leakage:** source listing and association accept only MyTimetable; event SQL already requires MyTimetable and receives regression coverage with persisted legacy rows/bindings.
- **Shared-infrastructure regression:** shared modules are retained and their focused/full suites are required.
- **Recreation through generic IPC:** Integration creation explicitly rejects retired provider IDs without changing generic provider-neutral persistence.

## Rollback considerations

Code/UI changes are reversible on the task branch. No migration or automatic data mutation occurs. Users retain legacy state until explicit cleanup, and rollback can again interpret those records through the historical provider code if required.

## Required validation

- Focused Integration repository, Integration Sync, unsupported subscribed-calendar cleanup, MyTimetable, School repository/UI, Connections/presentation/typed IPC, CAL-ICS, subscribed-calendar, and Calendar Core tests.
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`
- `pnpm typecheck`
- `pnpm lint`
- `pnpm test`
- `pnpm build`
- `git diff --check`
- Final diff review confirming shared ICS/MyTimetable infrastructure is intact.

## Independent review requirement

| Field | Value |
| --- | --- |
| Required | No |
| Reason | Repository workflow requires a distinct evidence-based self-review; no separate reviewer was requested. |
| Reviewer scope | Runtime dispatch closure, legacy cleanup/credential ordering, School isolation, MyTimetable preservation, current documentation truth, and scope discipline. |

## Human decisions required

None. The request explicitly chooses removal, retention of shared infrastructure, smallest safe legacy compatibility, no automatic reinterpretation, and draft-PR publication.

## Blocking decisions

None.

## Worktree / ownership gate

| Check | State |
| --- | --- |
| Correct branch/worktree confirmed | Pass — clean `agent/remove-brightspace` at merged `origin/master` commit `1999a51` |
| `git status` inspected | Pass — clean before contract updates |
| User-owned changes identified | None |
| Parallel task overlap checked | Pass — prior School scoping task is merged; no PR exists for this branch |
| Serialization points identified | Provider registry/commands, Integration runtime dispatch, generic connection cleanup, School source policy, current state docs |

## Readiness review

Passed. Removal boundaries, legacy-data behavior, credential cleanup ordering, no-migration decision, School isolation, shared-infrastructure preservation, validation, rollback, publication, and stop condition are explicit. Production implementation may begin.

## Implementation log

- 2026-09-25: Read the approved request and mandatory control documents; confirmed the clean task branch at merged Aether 27.
- 2026-09-25: Classified repository references into production removal, current-state update, and historical/shared retention categories.
- 2026-09-26: Removed the provider module, command/setup wrappers, runtime registration, catalog entry, capability presentation, and School association path.
- 2026-09-26: Added generic unsupported-calendar cleanup, retired-ID creation rejection, legacy startup/sync/credential/cascade/School regression coverage, and truthful current-state documentation.
- 2026-09-26: Completed focused/full validation and a distinct final-diff, security, shared-infrastructure, and scope review.

## Verification evidence

- Integration repository retired-ID/legacy-read test: 1 passed.
- Unsupported subscribed-calendar cleanup tests: 2 passed.
- MyTimetable tests: 22 passed.
- CAL-ICS tests: 21 passed.
- Integration Sync Runtime tests: 18 passed.
- School repository tests: 13 passed.
- Calendar Core repository tests: 3 passed.
- Focused Connections, presentation, School, and typed IPC tests: 46 passed across 4 files.
- `cargo test --manifest-path src-tauri/Cargo.toml`: 202 passed.
- `pnpm test`: 133 passed across 37 files.
- `pnpm typecheck`, `pnpm lint`, `pnpm build`, Rust formatting, strict Clippy, and `git diff --check`: passed.

## Acceptance evidence

- AC1: provider file and Tauri commands are deleted; frontend catalog/setup/wrappers contain no active Brightspace path.
- AC2: retired provider IDs are rejected case-insensitively by generic creation; all 22 MyTimetable tests and active setup/refresh frontend tests pass.
- AC3: legacy rows deserialize, manual/startup runtime requests reject before work, and generic cleanup removes the encrypted secret plus subscribed-calendar, ExternalEvent, and School-binding dependents.
- AC4: School discovery, association, public schema, and UI now accept only MyTimetable; a seeded legacy binding/event remains invisible.
- AC5: shared CAL-ICS, subscribed-calendar replacement, credential, Integration Sync, Calendar Core, ExternalEvent, and normalized School tables remain present and green.
- AC6: no migration was added; current docs record intentional retirement and the institution-authorization gate while historical ADR/changelog evidence remains.

## Self-review

Passed. The final diff is limited to the provider/runtime/Connections/School removal, provider-neutral legacy cleanup, regression tests, and current task/documentation records. No generic parser, fetcher, redirect/SSRF rule, validator, generation guard, sync scheduling policy, ExternalEvent identity, Calendar Core contract, DPAPI implementation, migration, or MyTimetable behavior was removed or weakened. Cleanup cancels in-flight work, refuses supported connections, removes the native secret before deleting the Integration, and relies on existing foreign-key cascades. Remaining production `brightspace` text is only the explicit retired-ID creation guard; other source references are compatibility tests. Historical docs remain intentionally historical. No Pulse, AI, replacement LMS, dependency, or unrelated change was introduced.

## Publication state

Verified implementation is ready for the required `scripts/publish-task.ps1` commit, push, and draft PR. Exact commit and PR are recorded after publication.

## Stop condition

Stop after all acceptance criteria are evidenced, required checks pass, task records are updated, the implementation is committed and pushed on `agent/remove-brightspace`, and a draft PR is open. Do not begin a replacement LMS, Pulse, AI, or unrelated integration work.
