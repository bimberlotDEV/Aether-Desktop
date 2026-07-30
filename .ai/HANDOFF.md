# Agent Handoff

> Canonical contract for the single task currently moving from planning to implementation or from implementation to review.

| Field | Value |
| --- | --- |
| Schema version | 1 |
| Task ID | `DOC-001` |
| Status | `ready` |
| Owner | Codex |
| Prepared by | Hermes |
| Last updated | 2026-07-30 |
| Related milestone | `M-DOC-VERSIONING` |

## Responsibility of this file

- Hold exactly one active, implementation-ready assignment.
- Define scope, editable files, acceptance criteria, verification, and risks.
- Record Codex's implementation result and Hermes's review decision.
- Never act as a general backlog, architecture diary, or changelog.

## Status values

`draft` -> `ready` -> `in_progress` -> `ready_for_review` -> `accepted`

Use `blocked` from any non-terminal state when progress requires a decision or external dependency. An accepted handoff is replaced only when Hermes promotes the next task from `.ai/TODO.md`.

## Current task

### Objective

Reconcile all version, maturity, and milestone metadata across the repository so README, package.json, Cargo.toml, tauri.conf.json, and `.ai/PROJECT_STATE.md` agree on a single canonical version and maturity description.

### Context

`RISK-003` (reported in `.ai/PROJECT_STATE.md`) identified that README, package metadata, Tauri metadata, and agent phase guidance disagree on version/maturity:

- `README.md` line 8 claims "Alpha 0.2.0 — Database Layer (Phase 2)".
- `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json` all say version `0.1.0`.
- `Cargo.toml` has an empty `repository` field despite the configured Git remote.
- `.ai/PROJECT_STATE.md` shows Phases 0-2 are complete, Phase 3 (Spaces) and Phase 4 (Notes) are substantially complete, Phase 7 (AI) is a prototype.
- `DEBT-004` notes the empty `repository` field.

The true state is: the application is in alpha, with Spaces and Notes materially implemented. The version should reflect this reality.

### Implementation plan

1. Read `README.md`, `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, and `.ai/PROJECT_STATE.md` to confirm current values.
2. Decide the canonical version. Since Phases 3 and 4 (Notes) are substantially complete and Phase 5 (Tasks) is not started, bump to `0.3.0-alpha` across all three version fields.
3. Update `README.md` status line to reflect true maturity: "Alpha 0.3.0 — Notes & Spaces (Phases 3-4)".
4. Set `Cargo.toml` repository field to the actual Git remote URL.
5. Update the README database schema list to include `notes` (the notes table exists from Phase 4).
6. Verify all changes and mark `RISK-003` and `DEBT-004` as resolved.

### Allowed files

- `README.md`
- `package.json`
- `src-tauri/Cargo.toml`
- `src-tauri/tauri.conf.json`
- `.ai/PROJECT_STATE.md` (update quality snapshot and resolve RISK-003)
- `.ai/TODO.md` (mark DOC-001 done and resolve RISK-003/DEBT-004)

### Out of scope

- Product code changes.
- Database migrations or schema changes.
- New features or UI changes.
- Bumping Rust or Node dependencies.

### Acceptance criteria

- [ ] `package.json`, `Cargo.toml`, and `tauri.conf.json` agree on the same version.
- [ ] `README.md` status line matches the real project maturity.
- [ ] `Cargo.toml` repository field is populated.
- [ ] `README.md` database schema list reflects current tables (includes `notes`).
- [ ] `RISK-003` is resolved in `.ai/PROJECT_STATE.md`.
- [ ] `DEBT-004` is resolved in `.ai/TODO.md`.
- [ ] No application code or database schema is changed.

### Required verification

- `git diff --check` passes with no whitespace errors.
- `git diff --name-only` shows only the allowed files.
- All version fields read back consistently.
- `pnpm check` still passes (only docs changed, but confirm).

### Risks and constraints

| ID | Risk | Mitigation |
| --- | --- | --- |
| `RISK-DOC-001` | Version bump misses a hidden manifest. | Explicit allowed-file list; diff audit confirms only listed files changed. |
| `RISK-DOC-002` | README database table list drifts again. | Cross-reference with actual `MIGRATIONS` array in `src-tauri/src/db/migrations.rs`. |

## Implementation result

### Summary

`Not started`

### Files changed

- None

### Verification result

`Not run`

### Deviations

None

## Hermes review

| Field | Value |
| --- | --- |
| Decision | `pending` |
| Reviewer | Hermes |
| Reviewed at | `Not reviewed` |
| Findings | None |
| Follow-up task IDs | None |

## Reusable task template

When Hermes prepares the next assignment, replace **Current task** through **Hermes review** with:

```markdown
## Current task

### Objective
<One measurable outcome>

### Context
<Why this task exists and relevant state>

### Implementation plan
1. <Ordered implementation step>

### Allowed files
- `<path or bounded directory>`

### Out of scope
- <Explicit exclusion>

### Acceptance criteria
- [ ] <Observable criterion>

### Required verification
- `<command>`

### Risks and constraints
| ID | Risk | Mitigation |
| --- | --- | --- |

## Implementation result
### Summary
`Not started`
### Files changed
- None
### Verification result
`Not run`
### Deviations
None

## Hermes review
| Field | Value |
| --- | --- |
| Decision | `pending` |
| Reviewer | Hermes |
| Reviewed at | `Not reviewed` |
| Findings | None |
| Follow-up task IDs | None |
```
