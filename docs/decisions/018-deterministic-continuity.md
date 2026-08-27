# ADR-018: Deterministic local continuity and curated Activity

- **Status:** Accepted
- **Date:** 2026-08-26
- **Task:** `CONT-001`

## Context

Aether already persists enough structured local state to help a user resume a Space, but it does not compose that state into a useful overview. Its Activity table is only partially populated and the Activity route is a placeholder. Generating a prose brief through DeepSeek would make a basic local workflow network-dependent, less predictable, and harder to audit. Allowing the frontend to submit arbitrary Activity names also undermines a meaningful event vocabulary.

## Decision

Build one Rust continuity repository that returns a bounded, typed, Space-scoped read model from existing SQLite tables. It selects recent active Notes, open Tasks, present Source-file metadata, the latest active AI conversation, curated Activity events, and a fixed-priority next step. The repository returns structured facts, not generated prose.

Activity event creation belongs to backend domain commands. Events use a closed vocabulary and small presentation-safe metadata. Repeated Space opens and Note autosaves are deduplicated with fixed quiet windows; Source scans record an event only when the indexed snapshot changes. Remove the public raw event-recording Tauri command while retaining a bounded typed Activity read command.

The global Activity route and Space overview consume typed presentation models. Browser mode shows an installed-app disclosure instead of fake records. No new migration or dependency is introduced.

## Consequences

- Continuity remains available offline, deterministic, fast, testable, and explainable.
- Space isolation and archived/removed exclusions can be enforced and reviewed in one repository boundary.
- Activity is intentionally a useful product timeline, not a complete compliance log.
- Event titles may be less expressive than generated summaries, but cannot hallucinate or silently disclose data to an AI provider.
- Future AI briefs, watchers, or richer file intelligence require separate consent and privacy decisions.
