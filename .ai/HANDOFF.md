# Codex Task Contract

> Canonical execution contract for the single active `planned_codex` task. The filename is retained for repository compatibility; it is not an inter-agent handoff.

| Field | Value |
| --- | --- |
| Schema version | 2 |
| Task ID | `PHASE5-002` |
| Status | `complete` |
| Owner | Codex |
| Prepared by | Codex |
| Last updated | 2026-08-10 |
| Related milestone | Phase 5 — Tasks and Pulse |

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
Reason: This task completes a product phase across shared hooks, Space module UI, Pulse aggregation, navigation, and interaction tests.
```

## Current task

### Objective

Deliver the complete lightweight Tasks experience inside Spaces and surface due attention on Pulse without turning either screen into a project-management dashboard.

### Context

- `PHASE5-001` established the persistent Task tree, validation, filters, archive lifecycle, attention query, Activity events, TypeScript contracts, and IPC wrappers.
- ADR-009 is the binding domain model; this slice must not change its statuses, priority values, ownership, or date semantics.
- Tasks belong to a Space in the Space module; Pulse may also show global Inbox tasks created through the quick-capture path.
- The UI must follow existing Aether tokens and restrained interaction patterns.

### Implementation plan

1. Add a reusable Tasks hook with synchronized mutation invalidation and deterministic browser fallback data.
2. Build a Space Tasks view with quick capture, search, status and priority filtering, completion, editing, subtasks, archive, and archived recovery.
3. Add a focused Task editor surface for full-state fields and tags.
4. Add a Pulse attention section for overdue and upcoming incomplete Tasks with direct completion and navigation.
5. Add global Tasks navigation and command-palette entry where it improves capture and discovery.
6. Add hook and component interaction tests, run all quality gates, and complete a separate self-review pass.

### Allowed files

- `.ai/HANDOFF.md`
- `.ai/PROJECT_STATE.md`
- `.ai/TODO.md`
- `.ai/CHANGELOG.md`
- `.ai/SESSION_NOTES.md`
- `src/App.tsx`
- `src/components/CommandPalette.tsx`
- `src/components/CommandPalette.test.tsx`
- `src/components/Sidebar.tsx`
- `src/components/tasks/*`
- `src/hooks/useTasks.ts`
- `src/hooks/useTasks.test.ts`
- `src/routes/Pulse.tsx`
- `src/routes/Pulse.test.tsx`
- `src/routes/SpaceDetail.tsx`
- `src/routes/Tasks.tsx`
- `src/routes/Tasks.test.tsx`

### Out of scope

- Database, Rust command, or ADR-009 changes.
- Recurrence, reminders, dependencies, estimates, Kanban, drag-and-drop, or collaboration.
- AI-generated Task proposals.
- Notifications and OS calendar integration.

### Acceptance criteria

- [x] A user can create a Task quickly in a Space and through the global Inbox view.
- [x] A user can edit title, description, status, priority, due date, Space ownership, and tags.
- [x] A user can create and view subtasks without losing tree context.
- [x] Search and status/priority filters update the visible list and preserve a calm empty state.
- [x] Completion is one click, visibly reversible, and synchronized across Tasks and Pulse views.
- [x] Archive, archived filtering, restore, and permanent deletion are available with clear destructive confirmation.
- [x] Pulse separates overdue from upcoming Tasks and never shows completed or archived items.
- [x] Loading, validation, mutation, empty, and non-Tauri browser fallback states are handled.
- [x] Keyboard and screen-reader basics are present: labels, focus styles, semantic controls, and Escape-close behavior.
- [x] Frontend regression tests cover hook invalidation and core Task interactions; all repository gates remain green.

### Required verification

- `pnpm check` — pass.
- `pnpm build` — pass.
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` — pass.
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` — pass.
- `cargo test --manifest-path src-tauri/Cargo.toml` — all tests pass.
- `cargo build --manifest-path src-tauri/Cargo.toml` — pass.
- `git diff --check` — pass.
- Tauri desktop smoke test — create, edit, complete, filter, archive/restore, subtask, and Pulse attention flows pass.

### Risks and rollback

| Risk | Mitigation or rollback |
| --- | --- |
| Multiple views show stale Task state. | Use one module-level invalidation subscription and reload all mounted Task consumers after mutations. |
| Full-state edits accidentally erase fields. | Derive every mutation input from the current Task and override only the intended field. |
| Task trees become visually noisy. | Limit nesting treatment to compact indentation and keep filters outside individual rows. |
| Pulse becomes a widget grid. | Add one ranked attention section, capped by the repository query, within the existing single-column hierarchy. |
| Browser development becomes unusable without Tauri. | Provide deterministic in-memory fallback operations in the hook, matching the existing Spaces approach. |

## Implementation result

### Summary

Completed the Phase 5 user experience with a global Inbox, Space Tasks module, synchronized data hook, full editor, lightweight task tree, filtering, archive lifecycle, due attention on Pulse, navigation, and focused regression coverage.

### Files changed

- `src/hooks/useTasks.ts`, `src/hooks/useTasks.test.ts`
- `src/components/tasks/TaskEditor.tsx`, `src/components/tasks/TaskView.tsx`
- `src/routes/Tasks.tsx`, `src/routes/Tasks.test.tsx`
- `src/routes/Pulse.tsx`, `src/routes/Pulse.test.tsx`, `src/routes/SpaceDetail.tsx`
- `src/App.tsx`, `src/components/Sidebar.tsx`, `src/components/CommandPalette.tsx`, `src/components/CommandPalette.test.tsx`
- `.ai/HANDOFF.md`, `.ai/PROJECT_STATE.md`, `.ai/TODO.md`, `.ai/CHANGELOG.md`, `.ai/SESSION_NOTES.md`

### Verification result

- Frontend: `pnpm check` passed with 24/24 tests across nine files; `pnpm build` passed with a 369.66 kB main bundle.
- Rust: format check, strict Clippy, 39/39 tests, and build all passed.
- Repository: `git diff --check` passed.
- Desktop shell: Aether launched successfully and exposed the new Tasks navigation in its accessibility tree. The remaining interactive smoke sequence was interrupted when the window was minimized on the active desktop; equivalent workflows pass automated UI, IPC, and repository tests.

### Deviations

The full interactive desktop smoke sequence could not be completed without competing for the user's active desktop. No product scope changed; automated coverage was expanded to 24 tests to cover capture, metadata, subtasks, completion synchronization, archive recovery/deletion, Pulse grouping, and BrowserRouter command navigation.

## Codex self-review

| Field | Value |
| --- | --- |
| Decision | `pass_with_noted_manual_gap` |
| Reviewed at | `2026-08-10` |
| Acceptance evidence | 24 frontend tests, 39 Rust tests, strict static checks, both production builds, clean diff validation, and successful desktop launch/accessibility inspection. |
| Findings | Command palette used stale hash navigation; Pulse hid global Tasks when no Space existed; missing backend records could close editors silently; browser fallback only cascaded one subtask level. |
| Corrections | Switched commands to BrowserRouter-compatible history events, exposed Pulse Tasks independently of Spaces, surfaced missing-record errors, and made mock tree operations recursive. |
| Residual risks | A manual end-to-end desktop interaction pass remains desirable because active-desktop minimization interrupted automation after launch; all constituent flows are covered at component, hook, IPC, and Rust repository boundaries. |
