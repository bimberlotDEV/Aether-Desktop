# Session Notes

| Field | Value |
| --- | --- |
| Schema version | 2 |
| Session date | 2026-09-26 |
| Active task | `SCHOOL-BSP-REMOVE-001` |
| Agent | Codex |
| State | complete; draft PR open |

## Current work

Aether 30 removes the calendar-only Brightspace connector because it cannot satisfy
the intended rich LMS product requirement without institution-managed application
authorization. The clean task branch is `agent/remove-brightspace` at merged Aether
27. The ready contract preserves all generic ICS/calendar/sync/credential machinery
and MyTimetable behavior.

Legacy Brightspace rows are not migrated or reinterpreted. They remain readable,
are rejected by active runtime dispatch, are excluded from School sources/events,
and can be explicitly removed through a provider-neutral unsupported subscribed-
calendar cleanup path with credential deletion and existing cascade semantics.

Focused suites pass: retired-ID/legacy read 1, unsupported cleanup 2, MyTimetable
22, CAL-ICS 21, Integration Sync 18, School repository 13, Calendar Core 3, and
frontend/IPC 46. Full validation passes: 202 Rust tests, 133 frontend tests across
37 files, production build, TypeScript, lint, Rust formatting, strict Clippy, and
diff check. No migration, dependency, Pulse, AI, or replacement LMS work was added.

## Exact resume point

Implementation commit `57ca726` is pushed to `origin/agent/remove-brightspace` and
draft PR #65 is open. The post-commit helper could not fork, but the required
publication script's explicit push and PR creation succeeded. The task is complete;
stop without beginning replacement LMS, Pulse, AI, or unrelated work.
