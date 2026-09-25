# Session Notes

| Field | Value |
| --- | --- |
| Schema version | 2 |
| Session date | 2026-09-25 |
| Active task | `SCHOOL-SCOPE-002` |
| Agent | Codex |
| State | complete; draft PR open |

## Current work

Aether 27 explicit School source scoping is implemented and verified on
`agent/school-scope-002`. Migration 018 adds normalized Space→connection bindings
and per-source group rows. Rust derives all schedule authority from an active parent
School Space; same provider/group labels cannot cross connections or Spaces.

Disabled sources retain scoped cached data with truthful state. Disconnected or
deleted sources return no events, deletion cascades bindings, and same-connection
feed replacement preserves them. Brightspace can be associated but remains
calendar-only and is not projected into timetable views.

Focused suites pass: School repository 13, source migration 2, MyTimetable 22,
Brightspace 4, and School/frontend IPC 34. Full validation passes: 202 Rust tests,
132 frontend tests across 37 files, production frontend build, Rust formatting,
strict Clippy, TypeScript, lint, and diff check. No dependency was added; ADR-031
records the durable boundary. Implementation commit `508866f` is pushed to
`origin/agent/school-scope-002`; draft PR #63 is open.

## Exact resume point

The task is complete. Await review and exact-head CI on draft PR #63. Do not begin
Pulse, AI, or richer LMS semantics under this contract.
