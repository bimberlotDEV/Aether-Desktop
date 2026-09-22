# ADR-030 — Native provider-neutral Integration sync runtime

- Status: Accepted
- Date: 2026-09-21

## Context

Integration Core persists connection state and CAL-ICS owns safe feed transport and normalized Calendar input, but desktop synchronization needs one owner for scheduling, coalescing, retry eligibility, and truthful lifecycle state.

## Decision

Use a closed, native Rust handler registry. The runtime starts only while the Aether desktop process runs, schedules startup, focus, periodic, and manual requests per connection, and uses a global semaphore with one active connection per provider. It never passes credentials or raw provider payloads through IPC.

Handlers prepare remote work outside the SQLite mutex. The runtime then opens a short transaction to reconcile normalized Calendar data and update Integration success metadata atomically. A failed or cancelled run records a bounded taxonomy code/message and a jittered exponential eligibility delay; manual refresh uses the same `next_allowed_sync_at` gate.

## Consequences

- CAL-ICS remains responsible for feed URL access, HTTPS fetches, validators, parsing, and normalized snapshots; it does not schedule itself.
- Runtime state is additive in migration `014_integration_sync_runtime`.
- React may request a sync and refetch Integration state, but may only mutate the enabled flag. State events carry only a connection ID and active/terminal state.
- No provider plugins, external writes, or background service are introduced.
