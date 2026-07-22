# ADR 001: SQLite Integration via rusqlite

**Date:** 2026-07-22
**Status:** Accepted

## Context

Aether needs a local-first persistence layer. Options considered:

1. **tauri-plugin-sql** — Official Tauri SQL plugin
2. **rusqlite** — Rust SQLite library with bundled SQLite
3. **sqlx** — Async SQL toolkit with compile-time query checking

## Decision

**rusqlite with `bundled` feature.**

## Rationale

- **rusqlite** gives full control over connection lifecycle, migrations, and pragmas
- **bundled** feature ships SQLite with the app — no system dependency
- Direct database access in Rust with strongly-typed repository layer
- Parameterized queries prevent SQL injection
- No frontend SQL access — all queries go through Tauri commands
- Simple, well-maintained, widely used in the Rust ecosystem

### Why not tauri-plugin-sql?

- Exposes SQL execution to the frontend by default (security risk)
- Less control over connection management
- Doesn't encourage typed repository pattern

### Why not sqlx?

- Adds complexity (async runtime, connection pools)
- Compile-time query checking is useful but adds build overhead
- Overkill for a single-user desktop app with simple queries

## Architecture

```
React components
    ↓ (invoke)
Tauri commands (commands.rs)
    ↓
Repository functions (settings.rs, profile.rs, etc.)
    ↓
rusqlite Connection
    ↓
SQLite database (aether.db)
```

## Consequences

- All database logic lives in Rust — TypeScript has typed interfaces only
- Migrations are embedded Rust strings, versioned in source control
- Tests use in-memory databases — isolated, no real file needed
