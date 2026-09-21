# ADR-029 — Native provider-neutral iCalendar subscription ingestion

- Status: Accepted
- Date: 2026-09-21

## Context

Calendar subscriptions commonly use renewable URLs that act as bearer credentials. Aether needs one reusable native path for RFC 5545 feeds without coupling Calendar Core to MyTimetable, Brightspace, or a future sync runtime.

## Decision

Persist a public subscription record keyed by its existing `ics_feed` Integration. Store the URL only under that Integration's opaque DPAPI secret key; it is accepted only by a dedicated configuration boundary and never returned in subscription, Integration, or ExternalEvent IPC.

The native engine accepts HTTPS only, rejects embedded URL credentials, follows only a bounded HTTPS redirect chain, limits response bytes and parsed/expanded occurrences, sends caller-provided ETag/Last-Modified validators, and returns a `NotModified` result for HTTP 304. It parses components with the maintained `ical` crate and expands RFC recurrence sets with `rrule` plus `chrono-tz`. Floating timed values and unresolved/ambiguous named-zone local values are rejected rather than interpreted using the machine timezone.

Normalized records use UID plus the canonical UTC recurrence identity as Calendar Core's external/occurrence identity. Overrides retain their original recurrence identity, cancellations remain cancelled records, and a complete parsed snapshot may be passed to Calendar Core authoritative reconciliation. Fetch, parse, validation, or limit failures never produce a snapshot and therefore cannot tombstone existing events.

## Consequences

- Migration `013_subscribed_calendars` stores no URL or provider payload.
- Calendar feed policy remains provider-neutral; setup UI and provider adapters can reuse the narrow configuration/validation boundary.
- `INT-SYNC-001` owns scheduling, retries, runtime guards, cancellation, and Integration lifecycle state. CAL-ICS supplies transport results and normalized data only.
