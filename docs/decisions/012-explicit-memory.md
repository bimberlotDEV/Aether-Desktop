# ADR-012 — Explicit, scoped Memory

- **Status:** Accepted
- **Date:** 2026-08-10

## Context

Aether needs durable context without turning conversations into an invisible or uncontrolled memory store. Every retained item must be visible, editable, attributable, scoped, removable, and sent to an external AI only after explicit attachment.

## Decision

- Store Memory as a first-class SQLite entity with optional Space ownership.
- Require a title, content, reason, category, and `source = user` attribution.
- Support global and Space-specific listing, search, creation, editing, and permanent deletion.
- Delete Space-owned Memory with its Space; never silently convert it into global Memory.
- Integrate Memory into the existing explicit AI context attachment model. Rust resolves the latest item at send time and enforces Space isolation.
- Do not automatically extract or persist Memory from conversations in the MVP.

## Consequences

Users always control what is remembered and what leaves the device. Automatic memory suggestions or extraction require a later, separately reviewed consent design.
