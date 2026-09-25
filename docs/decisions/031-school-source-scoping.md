# ADR-031 — Connection-bound School Space calendar sources

- Status: Accepted
- Date: 2026-09-25

## Context

The School timetable currently starts from a valid parent School Space but discovers MyTimetable sources, groups, and events provider-wide. Provider IDs and group strings are classifications, not ownership identities: two connections can expose the same group and must remain isolated, and multiple parent School Spaces need independent configuration.

School source configuration must survive restart and same-connection feed replacement, fail closed when absent or ambiguous, clean up when a Space or connection is deleted, and remain provider-neutral without inventing Brightspace course or deadline semantics.

## Decision

Persist source authorization in `school_space_sources`, keyed by `(school_space_id, connection_id)` with cascading foreign keys to an active parent School Space's durable row and the Integration row. Persist selected group references in `school_space_source_groups`, keyed by `(school_space_id, connection_id, group_reference)` and cascading through the composite source binding. Provider identity is derived from the Integration and is never an ownership boundary or duplicated in the binding.

All schedule reads accept only the parent School Space identity and bounded time range. Rust resolves its bindings, provider/status metadata, per-connection selected groups, and ExternalEvents. MyTimetable group discovery is performed only for an associated connection. Event reads join both the Space binding and per-source selected groups; no source or group means zero events, and no provider-wide/all-group fallback exists.

The schema permits multiple sources and multiple groups per source, while the current UI keeps the common case simple with one select per associated MyTimetable connection. Brightspace may be associated and report status, but supplies no timetable groups and no events to the current timetable-only views.

Migration 018 converts the legacy `schoolGroup` setting only when exactly one subscribed MyTimetable connection exists. With zero or multiple candidates it creates no binding and requires explicit selection. It never associates multiple candidates, changes cached events, or selects by provider/group text during later reads.

## Consequences

- Connection ID becomes the explicit School authorization boundary; identical provider IDs, group labels, event titles, or times cannot broaden access.
- Space and connection deletion remove bindings through foreign keys. Disablement preserves bindings and cached source state. Same-ID feed replacement preserves bindings.
- Subject child and non-School Spaces cannot own School bindings or call the School read/configuration path successfully.
- The School IPC exposes only bounded presentation/configuration state and accepts no arbitrary connection/provider/group scope for reads.
- This decision does not change Calendar Core identity, sync scheduling, CAL-ICS, credentials, Pulse, AI, or Brightspace semantics.
