# ADR-019 — Deterministic local Pulse relevance

- **Status:** Accepted
- **Date:** 2026-08-27
- **Task:** `PULSE-002`

## Context

Pulse currently combines a few frontend lists but does not answer what matters today across the workspace. Relevance must remain explainable, local, quiet, and privacy-safe before later intelligence or actions are introduced.

## Decision

Rust exposes one bounded read model over existing SQLite records. Today groups open Tasks by local due date. Continue ranks active Spaces by their latest real work timestamp. New contains only present metadata from explicitly authorized Sources. Recent reuses curated Activity items. A fixed priority—overdue Task, due-today Task, upcoming Task, recently worked Space, new file, then empty—selects one next step and includes the factual reason.

Pulse never invokes DeepSeek, extracts file contents, infers urgency beyond stored dates/status, mutates data, or expands Source permissions. React renders the typed facts and retains user-controlled navigation to AI.

### 2026-09-26 extension — connected schedule snapshot (`PULSE-003`)

The same single-command boundary now also composes minimized Calendar, School,
Integration, Task, and Continuity projections. Rust captures one logical instant,
applies local-day and half-open interval semantics, detects bounded timed conflicts,
and computes source freshness from persisted Integration state using one two-hour
policy. React receives no broad `ExternalEvent` or `Integration` record and does not
assemble domain state independently.

MyTimetable occurrences are authorized only through active parent School Space
bindings and selected groups. Cached events may remain visible after a source is
disabled, disconnected, degraded, or stale, but the snapshot labels that state
truthfully. Academic deadlines remain unavailable because Aether has no normalized
Deadline domain; Pulse does not infer them from calendar text. The extension adds no
migration, dependency, provider fetch, AI tool, or mutation path.

## Consequences

- Relevance is predictable, testable, offline, and fast.
- A single bounded command avoids cross-hook loading races and inconsistent snapshots.
- The first version intentionally favors explainability over semantic ranking.
- Rollback removes the repository/command/UI integration without migration or data cleanup.
- The connected snapshot remains intentionally small: Today 30, Upcoming 30, Tasks
  20, Conflicts 10, Continuity 5, and only relevant source-trust rows.
