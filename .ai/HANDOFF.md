# Codex Task Contract

> Canonical execution contract for the single active `planned_codex` task. The filename is retained for repository compatibility; it is not an inter-agent handoff.

| Field | Value |
| --- | --- |
| Schema version | 2 |
| Task ID | `PHASE34-CLOSEOUT` |
| Status | `complete` |
| Owner | Codex |
| Prepared by | Codex |
| Last updated | 2026-08-10 |
| Related milestone | Phase 3 Spaces and Phase 4 Notes |

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
Reason: This closeout changes cross-component state synchronization and autosave lifecycle behavior, where regressions could lose user data.
```

## Current task

### Objective

Close the verified Phase 3 Spaces and Phase 4 Notes gaps so both phases meet their MVP acceptance requirements with regression coverage.

### Context

- Space update, module update, restore, and reorder operations exist in the backend but are incomplete or stale in the UI.
- Archived Space detail currently calls archive instead of restore.
- Independent Space hooks do not share invalidation, leaving archived and detail views stale after mutations.
- A pending Note autosave is cancelled on unmount, which can discard the user's latest edits.
- Archived Notes have no restore/delete UI.
- Frontend coverage currently does not exercise Spaces, Notes, or Tauri boundaries (`DEBT-001`).

### Implementation plan

1. Add shared Space invalidation and complete update, module edit, restore, and reorder interactions.
2. Add archived Notes discovery, restore, and permanent-delete interactions.
3. Extract a testable Note autosave coordinator that serializes saves and flushes pending edits on teardown.
4. Add risk-based frontend tests for Tauri wrappers, Space refresh behavior, and Note autosave lifecycle.
5. Run all frontend and Rust quality gates, then perform a separate self-review.

### Allowed files

- `src/components/CreateSpaceModal.tsx`
- `src/components/EditSpaceModal.tsx`
- `src/components/Sidebar.tsx`
- `src/components/spaceOptions.ts`
- `src/hooks/useSpaces.ts`
- `src/hooks/useNotes.ts`
- `src/hooks/*.test.ts`
- `src/lib/autosave.ts`
- `src/lib/autosave.test.ts`
- `src/lib/db/tauri.test.ts`
- `src/routes/Spaces.tsx`
- `src/routes/SpaceDetail.tsx`
- `src/routes/ArchivedSpaces.tsx`
- `src/routes/Notes.tsx`
- `src/routes/Settings.tsx`
- `.ai/HANDOFF.md`
- `.ai/PROJECT_STATE.md`
- `.ai/TODO.md`
- `.ai/CHANGELOG.md`
- `.ai/SESSION_NOTES.md`

### Out of scope

- Phase 5 Tasks implementation.
- Changes to the SQLite schema or Rust repository behavior.
- Rich-text editing, tags, or cross-Space global Notes search.
- New visual design language or architecture redesign.

### Acceptance criteria

- [x] A Space can be edited, including name, description, icon, accent, and enabled modules.
- [x] Top-level active Spaces can be reordered with accessible controls and the order persists.
- [x] Space archive, restore, favourite, duplicate, delete, and edit mutations refresh every affected view.
- [x] Archived Space detail performs restore and returns to an active view.
- [x] Pending Note edits are serialized and flushed when switching Notes or leaving the editor.
- [x] Archived Notes can be listed, restored, and permanently deleted with confirmation.
- [x] Frontend regression tests cover the highest-risk Space, Note autosave, and Tauri-boundary behavior.
- [x] No existing Rust or frontend quality gate regresses.

### Required verification

- `pnpm check` — typecheck, lint, and all frontend tests pass.
- `pnpm build` — production frontend build passes.
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` — pass.
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` — pass.
- `cargo test --manifest-path src-tauri/Cargo.toml` — all Rust tests pass.
- `cargo build --manifest-path src-tauri/Cargo.toml` — Windows backend build passes.
- `git diff --check` — pass.

### Risks and rollback

| Risk | Mitigation or rollback |
| --- | --- |
| Autosave races overwrite newer text. | Serialize writes and retain only the latest pending draft; cover with deferred-promise tests. |
| Shared invalidation causes render loops. | Subscribe once per hook instance and trigger only after completed mutations. |
| Reorder loses archived or child ordering. | Reorder only the full ordered list of active top-level IDs; leave child and archived rows untouched. |
| Edit partially saves metadata but not modules. | Surface errors, refresh only after both operations succeed, and retain modal state for retry. |

## Implementation result

### Summary

Closed the Phase 3/4 acceptance gaps with Space editing and reordering, synchronized Space views, recoverable Note archives, full-content current-Space search, and a serialized flush-on-navigation autosave coordinator. Corrected stale UI version labels discovered during visual review.

### Files changed

- `src/components/CreateSpaceModal.tsx`
- `src/components/EditSpaceModal.tsx`
- `src/components/Sidebar.tsx`
- `src/components/spaceOptions.ts`
- `src/hooks/useNotes.ts`
- `src/hooks/useSpaces.ts`
- `src/hooks/useSpaces.test.ts`
- `src/lib/autosave.ts`
- `src/lib/autosave.test.ts`
- `src/lib/db/tauri.ts`
- `src/lib/db/tauri.test.ts`
- `src/routes/Notes.tsx`
- `src/routes/Settings.tsx`
- `src/routes/SpaceDetail.tsx`
- `src/routes/Spaces.tsx`
- `.ai/HANDOFF.md`
- `.ai/PROJECT_STATE.md`
- `.ai/TODO.md`
- `.ai/CHANGELOG.md`
- `.ai/SESSION_NOTES.md`

### Verification result

- `pnpm check` — pass; 14/14 tests across five files.
- `pnpm build` — pass.
- Rust format, strict Clippy, 33/33 tests, and build — pass.
- Browser smoke test — create, edit, archive, and restore pass with no console errors.
- `git diff --check` — pass.

### Deviations

The existing Tauri update command cannot distinguish SQL `NULL` from an omitted optional string. Explicitly cleared visual fields are persisted as empty strings, which render identically and remain schema-valid; a future command-payload redesign may normalize them to SQL `NULL`.

## Codex self-review

| Field | Value |
| --- | --- |
| Decision | `pass` |
| Reviewed at | 2026-08-10 |
| Acceptance evidence | All eight criteria verified by automated checks, code-path inspection, and local browser smoke testing. |
| Findings | Autosave teardown needed retrying of a retained failed draft; stale-revision recovery needed to refresh only the revision; UI version labels were stale. |
| Corrections | Added teardown retry, safe stale-revision refresh, keyed Note editors, current-Space backend search, and corrected version labels before final verification. |
| Residual risks | Empty-string normalization for cleared optional Space fields remains compatible but should become an explicit nullable patch payload in a future API cleanup. |
