# ADR-033 — Closed native read-only AI tool boundary

- **Status:** Accepted
- **Date:** 2026-09-26
- **Task:** `AI-NATIVE-TOOLS-001`
- **Extends:** ADR-032, ADR-031, ADR-028, ADR-009

## Context

ADR-032 separated AI routing, privacy, and tool authorization but intentionally
left `ToolScope` empty and non-executing. Aether now needs a first tool foundation
that can later serve cloud or local models without giving either model direct
repository, provider, filesystem, or database authority.

Calendar data may reveal sensitive School activity. Its current safe authority
is the persisted parent-School-Space to connection/group binding from ADR-031.
Tasks are local sensitive data with deterministic date/status semantics from
ADR-009. Aether has no normalized academic Deadline domain; calendar occurrences
must not be reinterpreted as deadlines.

## Decision

Rust owns a closed `NativeToolId` enum and immutable descriptors for exactly:

- `calendar.get_events`
- `calendar.get_next_event`
- `tasks.get_due`
- `tasks.get_open`

Descriptors contain stable strict JSON schemas, a versioned output contract,
Sensitive privacy classification, a read-only/local execution type, required
domain scope, and hard item/window/serialized-size limits. Unknown IDs fail
closed. Arguments deserialize into `deny_unknown_fields` structs and receive
semantic validation; callers cannot add provider, connection, group, SQL, table,
URL, path, or ranking inputs.

`ToolScope` is a native authorization grant, not a model argument. It contains
closed allowed tool IDs and optional bounded Calendar/Task read grants. Calendar
scope carries only the trusted current parent School Space identity plus exact
bounded UTC-instant, local-date, and result ceilings. Repository reads derive connections and selected groups
from persisted bindings. Task scope grants local Task reads with native time and
result ceilings. A tool request must satisfy both its descriptor requirement and
the supplied scope.

Execution accepts a local SQLite connection, closed tool ID, untrusted JSON
arguments, native scope, and a bounded execution context. It performs no network,
provider synchronization, frontend invoke, model recursion, or mutation. SQL
queries select only dedicated minimized DTO fields and apply their limits before
materialization. Results carry native-owned Sensitive classification and output
version internally. Serialized result data has a 64 KiB ceiling; exceeding it is
a typed error rather than lossy post-hoc truncation.

Calendar reads reuse the School binding/group authorization semantics, expose no
description/source URL/provider payload/configuration/credential data, and exclude
cancelled or removed occurrences. Task reads exclude body/tags/persistence
metadata, completed/archived Tasks, and use deterministic due/open ordering.
Caller limits above 50 and Calendar windows above 31 days are rejected.

Errors use a closed sanitized taxonomy. Repository and SQLite details are never
returned through the tool result. The existing router continues with an empty
scope and does not execute tools in this task.

## Consequences

- Cloud and future local models can share one native authorization contract.
- Adding a tool requires a Rust enum/descriptor/executor change and review; a
  model cannot dynamically register or redirect native functions.
- Current Calendar access remains limited to authorized School/MyTimetable
  projections. A general personal-calendar authority requires a later decision.
- No `school.get_deadlines` tool exists. It may be reconsidered only after a true
  normalized Deadline domain is designed and approved; ICS/event inference is forbidden.
- Write tools remain separate and must use Safe Actions where applicable.
- Tool-enabled routing/coordinator behavior is the exact next task, not part of this decision.
