# Codex Task Contract

| Field | Value |
| --- | --- |
| Schema version | 2 |
| Task ID | `PHASE8-001` |
| Status | `complete` |
| Owner | Codex |
| Last updated | 2026-08-10 |
| Related milestone | Phase 8 — Memory |

## Classification

```text
Classification: planned_codex
Reason: Memory adds durable private data, database ownership, deletion semantics, and external AI context boundaries.
```

## Objective

Implement visible, editable, attributable, removable global and Space-scoped Memory with explicit AI attachment and no automatic conversation extraction.

## Acceptance criteria

- [x] Append-only migration and tested Rust repository implement validated CRUD and search.
- [x] Every item records title, content, reason, category, source, scope, and timestamps.
- [x] Deleting a Space deletes its scoped Memory rather than changing its meaning.
- [x] Global and embedded Space UI support search, create, edit, and confirmed delete.
- [x] Memory is an optional Space module and a global navigation destination.
- [x] AI context picker exposes Memory; Rust enforces current scope and resolves current content.
- [x] The interface clearly states that Memory is never created from chat automatically.
- [x] Activity events record create, update, and delete without copying private content.
- [x] Frontend/Rust tests and all strict quality gates pass.

## Allowed files

- `docs/decisions/012-explicit-memory.md`, `docs/database.md`
- `src-tauri/src/db/migrations.rs`, `src-tauri/src/db/repositories/memory.rs`, repository registration
- `src-tauri/src/commands.rs`, `src-tauri/src/lib.rs`, `src-tauri/src/ai/context.rs`
- Memory types, wrappers, hooks, UI, routes, navigation, Space module files, and tests
- `.ai/*`

## Risks and controls

| Risk | Control |
| --- | --- |
| Invisible or unwanted memory | User-only creation; no chat extraction path. |
| Cross-Space disclosure | Rust resolves entity ownership against conversation scope at send time. |
| Private content copied into audit logs | Activity events contain IDs and category only. |
| Scope changes after Space deletion | Space-owned rows use `ON DELETE CASCADE`. |

## Verification and self-review

- `pnpm check` — pass; 39/39 tests across 14 files.
- `pnpm build` — pass.
- Rust format and strict Clippy — pass.
- `cargo test` — pass; 58/58 tests.
- `git diff --check` — pass.
- **Decision:** approved for publication.
- **Correction:** self-review added a database trigger and regression test to remove polymorphic AI attachments whenever Memory is deleted directly or through Space cascade.
