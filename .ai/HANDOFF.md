# Codex Task Contract

> Canonical execution contract for the single active `planned_codex` task. The filename is retained for repository compatibility; it is not an inter-agent handoff.

| Field | Value |
| --- | --- |
| Schema version | 2 |
| Task ID | `None` |
| Status | `idle` |
| Owner | Codex |
| Prepared by | Codex |
| Last updated | 2026-08-10 |
| Related milestone | `None` |

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
Classification: None
Reason: No planned task is active.
```

## Current task

### Objective

None.

### Context

None.

### Implementation plan

None.

### Allowed files

None.

### Out of scope

None.

### Acceptance criteria

- None.

### Required verification

- None.

### Risks and rollback

None.

## Implementation result

### Summary

None.

### Files changed

- None.

### Verification result

- None.

### Deviations

None.

## Codex self-review

| Field | Value |
| --- | --- |
| Decision | `not_started` |
| Reviewed at | `Not reviewed` |
| Acceptance evidence | None |
| Findings | None |
| Corrections | None |
| Residual risks | None |

## Planned-task template

Copy the structure below into the corresponding sections before setting status to `ready`:

```markdown
Classification: planned_codex
Reason: <concrete complexity or risk>

### Objective
<one observable outcome>

### Context
<verified facts, constraints, and decision links>

### Implementation plan
1. <bounded step>

### Allowed files
- `<path>`

### Out of scope
- <non-goal>

### Acceptance criteria
- [ ] <observable behavior>

### Required verification
- `<command>` — <expected result>

### Risks and rollback
| Risk | Mitigation or rollback |
| --- | --- |
| <risk> | <response> |
```
