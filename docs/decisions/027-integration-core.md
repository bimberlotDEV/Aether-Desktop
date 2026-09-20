# ADR-027 — Provider-neutral Integration Core

- Status: Accepted
- Date: 2026-09-21

## Context

Aether needs a durable local foundation before it can connect schedules, learning systems, source-control services, calendars, or automation providers. Those providers differ in authentication and synchronization mechanisms, but they all need a local connection record and must not expose credentials or provider-specific transport payloads to React.

Existing ADR-006 already establishes DPAPI-protected native secret storage. Existing SQLite repositories and typed Tauri commands are the product's trust boundary.

## Decision

Persist one provider-neutral `integrations` record per configured connection. It contains a stable ID, provider identifier, enabled state, declared capabilities, authentication type, opaque native credential reference and presence flag, generic synchronization configuration, connection/sync status, bounded last error, relevant sync timestamps, and audit timestamps.

The record stores no credential material. Rust alone maps its opaque credential reference to the existing encrypted secrets store; typed IPC returns no secret key or value. Provider connectors, credential entry flows, polling engines, webhooks, and normalized provider data are separate follow-up tasks. Future providers must translate provider-specific data at their connector boundary before generic domains or UI receive it.

The core recognizes realistic upcoming authentication modes (`none`, `api_key`, `api_token`, `oauth`, `ics_feed`) and synchronization triggers (`manual`, `periodic`, `app_start`, `app_resume`, `webhook`) as declarative metadata only. It does not execute them.

## Consequences

- Future providers share connection and sync lifecycle semantics without coupling generic UI or local domains to their remote schemas.
- A new append-only SQLite table is required and safely ignored by older product paths.
- This task deliberately does not establish connection, disconnect, synchronization, scheduling, webhook, or deletion behavior; later tasks must define those lifecycles and secret cleanup explicitly.
- Backup behavior continues to exclude the native `secrets` table; opaque references alone do not make credentials portable.
