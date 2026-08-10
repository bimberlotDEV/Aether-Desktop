# ADR-009: Task domain model and date semantics

- **Status:** Accepted
- **Date:** 2026-08-10
- **Decision owner:** Codex under the owner-approved Phase 5 roadmap

## Context

Aether needs lightweight personal Tasks that work inside Spaces and in the global Pulse view. The domain must support statuses, priorities, due dates, tags, subtasks, completion, archive, search, and filtering without becoming a project-management system. It must remain local-first and fit the existing repository → Tauri command → TypeScript wrapper architecture.

The durable choices are how Tasks and subtasks are represented, whether a Task must belong to a Space, how dates behave across time zones, and how mutable records cross the IPC boundary.

## Options considered

| Option | Advantages | Drawbacks |
| --- | --- | --- |
| Separate task and subtask tables with normalized tag tables | Strong relational normalization | More joins, commands, and migration surface than the personal MVP needs. |
| Single task table with self-reference and JSON tags | One repository and lifecycle for Tasks and subtasks; simple filtering and cascading deletion | Tags are not independently queryable entities. |
| Store due timestamps | Supports exact reminders later | Introduces time-zone ambiguity for date-oriented personal planning. |
| Store local calendar dates (`YYYY-MM-DD`) | Stable personal due-day semantics and lexicographic SQLite comparisons | Exact reminder times require a future additive field. |
| Patch-style optional IPC updates | Small payloads | Nullable fields cannot be distinguished reliably from omitted fields with the current optional command convention. |
| Full-state IPC updates | Explicit, deterministic validation and null handling | Callers must hold the current Task state. |

## Decision

1. Store Tasks and subtasks in one `tasks` table. `parent_task_id` is a nullable self-reference with `ON DELETE CASCADE`.
2. `space_id` is nullable with `ON DELETE SET NULL`, allowing a global Inbox while preserving Tasks if a Space is deleted.
3. Canonical statuses are `inbox`, `planned`, `in_progress`, and `done`.
4. Canonical priorities are `none`, `low`, `medium`, and `high`.
5. `due_date` is an optional local calendar date formatted as `YYYY-MM-DD`. Reminder timestamps and recurrence are deferred.
6. Tags are a validated JSON array of strings in `tags_json`; the Rust and TypeScript boundaries expose `tags` as arrays.
7. Task creation and update use explicit full-state inputs. Completion is represented by `status = done` plus `completed_at`; moving out of `done` clears `completed_at`.
8. Repository queries own active/archived filtering, search, and the global due-attention query used by Pulse.
9. Meaningful lifecycle events—creation, completion, and archive—are recorded in Activity. Editing keystrokes or filter changes are not.

## Consequences

- The schema remains small and supports nested Task rendering without a second entity lifecycle.
- Pulse can query overdue and upcoming Tasks without loading every Task into React.
- Subtask nesting is structurally possible, while the Phase 5 UI may constrain creation to one visible level.
- Exact reminder times, recurring Tasks, tag management, and advanced project views require additive future work.
- Full-state updates avoid the nullable-patch ambiguity already observed in the Space command boundary.

## Validation

- Migration tests verify the table, constraints, indexes, and idempotence.
- Repository tests cover CRUD, validation, filtering/search, subtasks, completion, attention queries, archive/restore, and cascade behavior.
- TypeScript boundary tests verify command names and argument shapes.
