# Codex Task Contract

> Canonical execution contract for the single active `planned_codex` task. The filename is retained for repository compatibility; it is not an inter-agent handoff.

| Field | Value |
| --- | --- |
| Schema version | 2 |
| Task ID | `PHASE6-002` |
| Status | `self_review` |
| Owner | Codex |
| Prepared by | Codex |
| Last updated | 2026-08-10 |
| Related milestone | Phase 6 — Vault UI and Space integration |

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
Reason: The Vault UI coordinates native file selection, ownership choices, destructive confirmation, metadata, global filters, and embedded Space context.
```

## Current task

### Objective

Deliver a complete, accessible Vault experience for global and Space-scoped file workflows on top of the verified PHASE6-001 foundation.

### Context

- PHASE6-001 and ADR-010 are merged through PR #9 and provide all persistence and safe native operations.
- The global Vault route is still a static empty state and Space `files` modules still render a placeholder.
- The interface must explain linked versus managed ownership before import and must make removal consequences explicit.
- Browser-mode fallbacks may support UI tests but must not manufacture realistic user files.

### Implementation plan

1. Add a synchronized `useVault` domain hook with predictable browser-test behavior and actionable errors.
2. Build reusable Vault view/editor/import UI for global and Space-scoped use.
3. Add search, storage-mode and Space filters, metadata editing, open, reveal, and ownership-specific removal confirmation.
4. Replace the Space `files` placeholder with the Space-scoped Vault view.
5. Add interaction tests, update project documentation, run all gates, and self-review.

### Allowed files

- `src/hooks/useVault.ts`, `src/hooks/useVault.test.ts`
- `src/components/vault/*`
- `src/routes/Vault.tsx`, `src/routes/Vault.test.tsx`, `src/routes/SpaceDetail.tsx`
- `.ai/ARCHITECTURE.md`, `.ai/HANDOFF.md`, `.ai/PROJECT_STATE.md`, `.ai/TODO.md`, `.ai/CHANGELOG.md`, `.ai/SESSION_NOTES.md`

### Out of scope

- Content parsing, full-text content indexing, thumbnails, previews, OCR, or DOCX rendering.
- Relinking moved external files, checksums, versioning, or duplicate-content detection.
- Drag-and-drop, multi-file import, cloud files, sync, or sharing.

### Acceptance criteria

- [x] Global Vault lists items with clear ownership, filename, size, Space, tags, and updated metadata.
- [x] Native import requires a linked/managed choice and supports optional title, tags, and Space assignment.
- [x] Search, ownership filter, and Space filter use backend filter contracts.
- [x] Metadata editing validates title/tags and supports moving between unassigned and active Spaces.
- [x] Open and reveal actions surface native failures without losing UI state.
- [x] Removal confirmation states that linked originals remain or that the managed Vault copy is deleted.
- [x] Empty, loading, error, and no-results states are distinct and accessible.
- [x] Space `files` modules render the same Vault experience prefiltered and imported into that Space.
- [x] Interaction tests cover import, metadata update, filters, open/reveal, and both removal messages.
- [x] Frontend and Rust quality gates remain green.

### Required verification

- `pnpm check` — pass.
- `pnpm build` — pass.
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` — pass.
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` — pass.
- `cargo test --manifest-path src-tauri/Cargo.toml` — all tests pass, including temporary-directory ownership tests.
- `cargo build --manifest-path src-tauri/Cargo.toml` — pass.
- `git diff --check` — pass.

### Risks and rollback

| Risk | Mitigation or rollback |
| --- | --- |
| Ownership consequences are unclear. | Explain modes in the import flow and repeat the exact consequence in removal confirmation. |
| Multiple mounted views become stale. | Central change notification reloads every active Vault hook. |
| Native selection is cancelled or fails. | Treat cancel as a no-op and expose actionable command failures inline. |
| Space context is lost during import/edit. | Lock the embedded view filter to its Space while keeping metadata behavior explicit. |

## Implementation result

### Summary

Implemented a reusable global and Space-scoped Vault experience with synchronized domain state, native file selection, explicit ownership, metadata workflows, backend filters, open/reveal actions, accessible states, and safe removal messaging.

### Files changed

- Added `src/hooks/useVault.ts`, `src/components/vault/VaultEditor.tsx`, `src/components/vault/VaultView.tsx`, and `src/routes/Vault.test.tsx`.
- Replaced the global empty state in `src/routes/Vault.tsx` and the Space files placeholder in `src/routes/SpaceDetail.tsx`.
- Updated the scoped control documents.

### Verification result

- `pnpm check` - pass; 30/30 tests across ten files.
- `pnpm build` - pass.
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` - pass.
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` - pass.
- `cargo test --manifest-path src-tauri/Cargo.toml` - pass; 48/48 tests.
- `git diff --check` - pass.

### Deviations

- Browser mode remains intentionally read-only and shows an explicit desktop-app requirement instead of inventing mock user files.
- Content previews and drag-and-drop remain deferred as scoped.

## Codex self-review

| Field | Value |
| --- | --- |
| Decision | `approved_for_publication` |
| Reviewed at | 2026-08-10 |
| Acceptance evidence | Ten criteria pass; five Vault interaction tests, 30 total frontend tests, and 48 Rust tests pass. |
| Findings | Initial action labels were ambiguous to assistive technology; ownership was not sufficiently visible in list rows. |
| Corrections | Added explicit `Open <title>` labels, distinct `Add to Vault` confirmation, and linked/managed labels with curated icons. |
| Residual risks | Native Windows picker/open/reveal behavior is covered at the command boundary but still requires final packaged-app smoke testing in Phase 9/10. |
