# ADR-028 — Provider-neutral external Calendar Core

- Status: Accepted
- Date: 2026-09-21

## Context

Future calendar feeds and APIs need one local representation without exposing provider payloads or allowing partial synchronization results to erase data. Recurrence, cancellation, all-day semantics, and daylight-saving transitions make title/time matching unsafe.

## Decision

Persist normalized external occurrences under `(connection_id, external_id, occurrence_id)`, with an empty stored occurrence component for non-recurring events. A connector supplies stable provider identifiers; Calendar Core never deduplicates by title or time. Upserts preserve an occurrence when its title or timing moves, cancelled occurrences remain records with `cancelled`, and missing authoritative occurrences become `removed` tombstones. A reappearing occurrence becomes active or cancelled as supplied.

Trusted native connectors call reconciliation. `ObservedOnly` upserts observations and never removes. `Authoritative` requires an explicit complete half-open UTC window and may tombstone only earlier observations within that connection/window. React receives only bounded range reads and cannot create, update, or remove provider-owned occurrences.

Timed occurrences store RFC3339 UTC instants and their source timezone context. Floating or non-UTC values are rejected before insertion. All-day occurrences use date-only start/end values, with an exclusive end date. The same half-open overlap rule applies to timed and all-day range reads.

## Consequences

- Migration `012_external_events` is additive and references the provider-neutral Integration Core connection.
- Calendar Core stores only normalized content, source provenance/version/hash, and optional HTTPS deep links; it stores no raw provider payload, feed URL, credential, token, or secret metadata.
- ICS parsing, connectors, scheduling, UI, local authored events, and cross-provider matching remain separate work.
