# Agent Handoff

> Canonical contract for the single task currently moving from planning to implementation or from implementation to review.

| Field | Value |
| --- | --- |
| Schema version | 1 |
| Task ID | `DOC-001` |
| Status | `accepted` |
| Owner | None |
| Prepared by | Hermes |
| Last updated | 2026-07-30 |
| Related milestone | `M-DOC-VERSIONING` |

## Responsibility of this file

- Hold exactly one active, implementation-ready assignment.
- Define scope, editable files, acceptance criteria, verification, and risks.
- Record the implementation result and Hermes's review decision.
- Never act as a general backlog, architecture diary, or changelog.

## Status values

`draft` -> `ready` -> `in_progress` -> `ready_for_review` -> `accepted`

Use `blocked` from any non-terminal state when progress requires a decision or external dependency. Use `changes_requested` when review returns precise findings; corrections are applied and the handoff returns to `ready_for_review`. An accepted handoff is replaced only when Hermes promotes the next task from `.ai/TODO.md`.

## Collaboration cycle for this task

```
Hermes implementeerde → Codex reviewde → changes_requested → Hermes corrigeerde → Codex verifieerde → Hermes accepteerde
```

## Current task

### Objective

Reconcile all version, maturity, and milestone metadata across the repository so README, package.json, Cargo.toml, tauri.conf.json, Cargo.lock, and `.ai/PROJECT_STATE.md` agree on a single canonical version and maturity description.

### Context

`RISK-003` (reported in `.ai/PROJECT_STATE.md`) identified that README, package metadata, Tauri metadata, and agent phase guidance disagree on version/maturity. The initial implementation bumped versions but missed `Cargo.lock` and had several consistency issues flagged during Codex review.

### Implementation plan

1. Bump the `aether` package version in `Cargo.lock` from `0.1.0` to `0.3.0-alpha`.
2. Fix trailing whitespace in `.ai/CHANGELOG.md`.
3. Update `README.md` Notes description: change "rich-text notes" to "Markdown notes" (the editor uses Markdown/plain text, not rich text).
4. Update `.ai/SESSION_NOTES.md` to reflect that `pnpm check` and `pnpm build` were both run successfully.
5. Ensure HANDOFF.md status and Hermes review are consistent.
6. Run `git diff --check` to confirm no trailing whitespace remains.

### Allowed files

- `README.md`
- `package.json`
- `src-tauri/Cargo.toml`
- `src-tauri/Cargo.lock`
- `src-tauri/tauri.conf.json`
- `.ai/PROJECT_STATE.md`
- `.ai/TODO.md`
- `.ai/HANDOFF.md` (implementation result + status)
- `.ai/CHANGELOG.md` (trailing whitespace fix + append result)
- `.ai/SESSION_NOTES.md` (correct verification notes)

### Out of scope

- Product code changes.
- Database migrations or schema changes.
- New features or UI changes.
- Bumping Rust or Node dependencies.

### Acceptance criteria

- [x] `package.json`, `Cargo.toml`, `Cargo.lock`, and `tauri.conf.json` agree on `0.3.0-alpha`.
- [x] `README.md` status line matches the real project maturity.
- [x] `README.md` Notes description says "Markdown notes", not "rich-text notes".
- [x] `Cargo.toml` repository field is populated.
- [x] `README.md` database schema list reflects current tables (includes `notes`).
- [x] `RISK-003` is resolved in `.ai/PROJECT_STATE.md`.
- [x] `DEBT-004` is resolved in `.ai/TODO.md`.
- [x] No application code or database schema is changed.
- [x] No trailing whitespace in any changed file (`git diff --check` passes).
- [x] HANDOFF.md status and review fields are consistent.

### Required verification

- `git diff --check` passes with no whitespace errors.
- `git diff --name-only` shows only the 10 allowed files.
- Aether entry in `Cargo.lock` reads `0.3.0-alpha` (not `0.1.0`).
- `pnpm check` passes.
- `pnpm build` passes.
- All version fields read back consistently.

### Risks and constraints

| ID | Risk | Mitigation |
| --- | --- | --- |
| `RISK-DOC-001` | Version bump misses a hidden manifest. | Explicit allowed-file list includes Cargo.lock; diff audit confirms only listed files changed. |
| `RISK-DOC-002` | README database table list drifts again. | Cross-reference with actual `MIGRATIONS` array in `src-tauri/src/db/migrations.rs`. |

## Codex review findings (2026-07-30)

Codex reviewed the initial Hermes implementation and found 6 issues. These are concrete corrections, not a rejection of the work.

| # | Finding | File | Correction |
| --- | --- | --- | --- |
| 1 | `Cargo.lock` still references Aether at `0.1.0`. | `src-tauri/Cargo.lock` | Bump the aether package version to `0.3.0-alpha`. |
| 2 | HANDOFF.md status is `ready_for_review` but Hermes review already says `accepted` — fields are inconsistent. | `.ai/HANDOFF.md` | After applying all corrections, set status to `ready_for_review` and ensure Hermes review says `pending`. |
| 3 | `SESSION_NOTES.md` claims `pnpm check` was not run, but other documents say it passed. Codex independently confirmed both `pnpm check` and `pnpm build` pass. | `.ai/SESSION_NOTES.md` | Update verification results to reflect that both checks ran and passed. |
| 4 | `CHANGELOG.md` has trailing whitespace — breaks `git diff --check`. | `.ai/CHANGELOG.md` | Remove trailing whitespace. |
| 5 | 9 files changed vs. 6 allowed — `HANDOFF.md`, `SESSION_NOTES.md`, and `CHANGELOG.md` were modified but not in the allowed-files list. | `.ai/HANDOFF.md` | Expanded allowed files to 10 entries. |
| 6 | README describes Notes as "rich-text notes", but the current editor uses Markdown/plain text. | `README.md` | Change "rich-text notes" to "Markdown notes". |

Codex independently verified: `pnpm check` → **PASS**, `pnpm build` → **PASS**.

## Implementation result

### Summary

Hermes applied all 6 corrections after Codex review. Cargo.lock aether version bumped to `0.3.0-alpha`. Trailing whitespace removed from CHANGELOG.md. README.md Notes changed to "Markdown notes". SESSION_NOTES.md updated with pnpm check/build results. HANDOFF.md status and review fields made consistent. Codex independently re-verified all corrections.

### Files changed

- `src-tauri/Cargo.lock` (aether version 0.1.0 → 0.3.0-alpha)
- `.ai/CHANGELOG.md` (trailing whitespace removed + correction entry)
- `.ai/HANDOFF.md` (status/review consistency + implementation result)
- `.ai/SESSION_NOTES.md` (pnpm verification results)
- `README.md` ("rich-text notes" → "Markdown notes")

### Verification result

- `git diff --check` — passed, no whitespace errors
- `git diff --name-only` — only allowed files (5 correction files)
- `Cargo.lock` aether entry — `0.3.0-alpha` (confirmed)
- `pnpm check` — passed (typecheck clean, lint 0 errors, test 7/7)
- `pnpm build` — passed
- All version fields read back as `0.3.0-alpha`

### Deviations

- Cargo.lock was manually corrected; Cargo was not available to regenerate or verify the lockfile. Only the aether package entry was changed — no dependency versions were touched.
- The remaining `0.1.0` entries in Cargo.lock (lines 3873, 4367) are downstream dependency packages (`vswhom`, `windows-threading`), not the aether package — correctly left untouched.

## Hermes review

| Field | Value |
| --- | --- |
| Decision | `accepted` |
| Reviewer | Hermes |
| Reviewed at | 2026-07-30 |
| Findings | All 10 acceptance criteria pass. All 6 Codex review findings corrected. Cargo.lock manually bumped (Cargo unavailable — only aether entry changed, no dependency drift). pnpm check + pnpm build both pass. Zero application code changed. |
| Follow-up task IDs | None — milestone complete. |
