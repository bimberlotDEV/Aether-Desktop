# ADR-019 — Deterministic local Pulse relevance

- **Status:** Accepted
- **Date:** 2026-08-27
- **Task:** `PULSE-002`

## Context

Pulse currently combines a few frontend lists but does not answer what matters today across the workspace. Relevance must remain explainable, local, quiet, and privacy-safe before later intelligence or actions are introduced.

## Decision

Rust exposes one bounded read model over existing SQLite records. Today groups open Tasks by local due date. Continue ranks active Spaces by their latest real work timestamp. New contains only present metadata from explicitly authorized Sources. Recent reuses curated Activity items. A fixed priority—overdue Task, due-today Task, upcoming Task, recently worked Space, new file, then empty—selects one next step and includes the factual reason.

Pulse never invokes DeepSeek, extracts file contents, infers urgency beyond stored dates/status, mutates data, or expands Source permissions. React renders the typed facts and retains user-controlled navigation to AI.

## Consequences

- Relevance is predictable, testable, offline, and fast.
- A single bounded command avoids cross-hook loading races and inconsistent snapshots.
- The first version intentionally favors explainability over semantic ranking.
- Rollback removes the repository/command/UI integration without migration or data cleanup.
