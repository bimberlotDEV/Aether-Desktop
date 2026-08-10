# Codex Task Contract

> Canonical execution contract for the single active `planned_codex` task. The filename is retained for repository compatibility; it is not an inter-agent handoff.

| Field | Value |
| --- | --- |
| Schema version | 2 |
| Task ID | `PHASE5-001` |
| Status | `complete` |
| Owner | Codex |
| Prepared by | Codex |
| Last updated | 2026-08-10 |
| Related milestone | Phase 5 — Tasks foundation |

## Responsibility of this file

- Hold exactly one active, bounded task contract for complex or risky work.
- Define objective, scope, allowed paths, acceptance criteria, validation, and risks before implementation.
- Record implementation evidence, deviations, self-review findings, and final outcome.
- Remain `idle` for `direct_codex` work.
- Never serve as the general backlog or architecture diary.

## Status values

- `idle`: no planned task is active.
- `draft`: Codex is analysing and writing the contract.
- `ready`: the readiness gate passes; implementation may begin.
- `in_progress`: implementation is underway.
- `self_review`: implementation is complete and undergoing an independent Codex review pass.
- `changes_required`: self-review found corrections that must be implemented.
- `blocked`: a human decision or external dependency is required.
- `complete`: acceptance criteria, verification, self-review, and publication are complete.
- `superseded`: replaced by another task with a recorded reason.

## Classification

```text
Classification: planned_codex
Reason: This task adds an append-only SQLite migration and a new cross-layer persistent domain governed by ADR-009.
```

## Current task

### Objective

Deliver the verified Task persistence and IPC foundation required for the Phase 5 Space UI and Pulse integration.

### Context

- Phase 5 requirements are defined in `IDEA.md` and include creation, editing, statuses, due dates, priority, filtering, and Pulse integration.
- The repository currently has no Task migration, repository, command, Rust type, TypeScript schema, or invoke wrapper.
- ADR-009 defines the Task model, enum values, local-date semantics, nullable Space ownership, self-referencing subtasks, tags, and full-state updates.
- UI delivery is intentionally separated into `PHASE5-002` so schema and command behavior can be independently verified and published.

### Implementation plan

1. Add append-only migration `005_tasks` with constraints and indexes.
2. Implement the Rust Task repository with validation, CRUD, filters/search, subtasks, archive lifecycle, and Pulse attention query.
3. Add Tauri commands, activity recording, and command registration.
4. Add strict Zod schemas, TypeScript types, and invoke wrappers.
5. Update database and architecture documentation.
6. Add repository, migration, and TypeScript boundary tests; run all quality gates and self-review.

### Allowed files

- `docs/decisions/009-task-domain-model.md`
- `docs/database.md`
- `.ai/ARCHITECTURE.md`
- `.ai/HANDOFF.md`
- `.ai/PROJECT_STATE.md`
- `.ai/TODO.md`
- `.ai/CHANGELOG.md`
- `.ai/SESSION_NOTES.md`
- `src-tauri/src/db/migrations.rs`
- `src-tauri/src/db/repositories/mod.rs`
- `src-tauri/src/db/repositories/tasks.rs`
- `src-tauri/src/commands.rs`
- `src-tauri/src/lib.rs`
- `src/lib/db/types.ts`
- `src/lib/db/tauri.ts`
- `src/lib/db/tauri.test.ts`

### Out of scope

- Task UI, quick capture, and Pulse rendering (`PHASE5-002`).
- Recurrence, reminders, Kanban, dependencies, estimates, or collaboration.
- AI-generated Task proposals.
- Schema changes outside the new append-only Task migration.

### Acceptance criteria

- [x] Migration `005_tasks` applies transactionally and is idempotent.
- [x] Task records support nullable Space, optional parent, title, description, canonical status/priority, optional local due date, tags, completion, archive, and timestamps.
- [x] Invalid titles, enums, due dates, tags, parent links, and self-parenting are rejected with actionable errors.
- [x] CRUD, search/filter, subtask, completion, archive/restore/delete, and due-attention repository behavior is covered by Rust tests.
- [x] Tauri commands expose the complete repository lifecycle and register in the application.
- [x] Task creation, completion, and archive produce meaningful Activity events.
- [x] Strict TypeScript/Zod contracts and invoke wrappers match the Rust command payloads.
- [x] Existing frontend and Rust behavior remains green.

### Required verification

- `pnpm check` — pass.
- `pnpm build` — pass.
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` — pass.
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` — pass.
- `cargo test --manifest-path src-tauri/Cargo.toml` — all tests pass.
- `cargo build --manifest-path src-tauri/Cargo.toml` — pass.
- `git diff --check` — pass.

### Risks and rollback

| Risk | Mitigation or rollback |
| --- | --- |
| A bad migration blocks existing user databases. | Append only, run in the existing transaction, and test both fresh and repeated migration execution. |
| Date comparisons vary by locale or time zone. | Accept and store only validated `YYYY-MM-DD` local dates. |
| Invalid enum strings leak through IPC. | Validate in the repository, independent of UI schemas. |
| Parent cycles corrupt Task trees. | Reject self-parenting and parent-to-descendant updates; cover with tests. |
| Activity logging makes a successful Task mutation fail. | Record in the same locked command flow and treat event failure as command failure before returning. |

## Implementation result

### Summary

Added the complete Task persistence boundary: append-only migration, validated Rust repository, transactional Tauri commands with Activity events, strict TypeScript contracts, invoke wrappers, and regression coverage. The UI remains intentionally isolated in `PHASE5-002`.

### Files changed

- `docs/decisions/009-task-domain-model.md`, `docs/database.md`
- `src-tauri/src/db/migrations.rs`, `src-tauri/src/db/repositories/mod.rs`, `src-tauri/src/db/repositories/tasks.rs`
- `src-tauri/src/commands.rs`, `src-tauri/src/lib.rs`
- `src/lib/db/types.ts`, `src/lib/db/tauri.ts`, `src/lib/db/tauri.test.ts`
- `.ai/ARCHITECTURE.md`, `.ai/HANDOFF.md`, `.ai/PROJECT_STATE.md`, `.ai/TODO.md`, `.ai/CHANGELOG.md`, `.ai/SESSION_NOTES.md`

### Verification result

- Frontend: `pnpm check` passed with 16/16 tests; `pnpm build` passed with a 346.14 kB main bundle.
- Rust: format check, strict Clippy, 39/39 tests, and build all passed.
- Repository: `git diff --check` passed.

### Deviations

Runtime Zod parsing was intentionally not added to the invoke wrapper: repository validation remains authoritative and importing the schemas there increased the main bundle by approximately 60 kB. The schemas remain available to the UI boundary.

## Codex self-review

| Field | Value |
| --- | --- |
| Decision | `pass` |
| Reviewed at | `2026-08-10` |
| Acceptance evidence | 39 Rust tests, 16 frontend tests, strict static checks, both production builds, and clean diff validation. |
| Findings | Initial runtime schema parsing caused an unnecessary frontend bundle increase. |
| Corrections | Kept Rust validation authoritative and removed the runtime schema import from the IPC wrapper; the bundle returned to 346.14 kB. |
| Residual risks | Tauri command transaction behavior is covered through repository and TypeScript boundary tests rather than a full desktop integration harness; UI behavior is deferred to `PHASE5-002`. |
