# Codex Task Contract

> Canonical execution contract for the single active `planned_codex` task. The filename is retained for repository compatibility; it is not an inter-agent handoff.

| Field | Value |
| --- | --- |
| Schema version | 2 |
| Task ID | `PHASE6-001` |
| Status | `self_review` |
| Owner | Codex |
| Prepared by | Codex |
| Last updated | 2026-08-10 |
| Related milestone | Phase 6 — Vault foundation |

## Responsibility of this file

- Hold exactly one active, bounded task contract for complex or risky work.
- Define objective, scope, allowed paths, acceptance criteria, validation, and risks before implementation.
- Record implementation evidence, deviations, self-review findings, and final outcome.
- Remain `idle` for `direct_codex` work.
- Never serve as the general backlog or architecture diary.

## Status values

- `idle`: no planned task is active.
- `draft`: Codex is analysing and writing the contract.
- `ready`: the readiness gate passes; implementation may begin.
- `in_progress`: implementation is underway.
- `self_review`: implementation is complete and undergoing an independent Codex review pass.
- `changes_required`: self-review found corrections that must be implemented.
- `blocked`: a human decision or external dependency is required.
- `complete`: acceptance criteria, verification, self-review, and publication are complete.
- `superseded`: replaced by another task with a recorded reason.

## Classification

```text
Classification: planned_codex
Reason: Vault adds a persistent cross-layer domain, native filesystem access, security-sensitive ownership rules, and append-only migration.
```

## Current task

### Objective

Deliver the verified Vault persistence, storage-safety, native-dialog, open, and reveal foundation required for the Phase 6 UI.

### Context

- Vault is currently a static empty state with no persistence or native file operations.
- ADR-010 defines linked versus managed ownership, relative managed paths, safe removal, and trusted Rust-side open/reveal commands.
- The official Tauri dialog plugin returns native paths; the official opener plugin supports default-app opening and Explorer reveal.
- UI delivery is intentionally separated into `PHASE6-002` so filesystem safety can be independently tested and published.

### Implementation plan

1. Add append-only migration `006_vault` with ownership constraints, metadata fields, search indexes, and nullable Space association.
2. Implement the validated Vault repository and filters.
3. Implement a filesystem service for canonical source inspection, atomic managed copies, containment checks, and quarantined safe deletion.
4. Add transactional Tauri commands for import, list, metadata update, remove, open, and reveal with Activity events.
5. Configure the official dialog and opener plugins with least-privilege frontend capabilities.
6. Add strict TypeScript contracts and invoke wrappers.
7. Document the schema and architecture, test storage/repository/IPC behavior, run all quality gates, and self-review.

### Allowed files

- `package.json`, `pnpm-lock.yaml`
- `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`
- `src-tauri/capabilities/default.json`
- `src-tauri/src/lib.rs`, `src-tauri/src/commands.rs`
- `src-tauri/src/vault.rs`
- `src-tauri/src/db/migrations.rs`
- `src-tauri/src/db/repositories/mod.rs`
- `src-tauri/src/db/repositories/vault.rs`
- `src/lib/db/types.ts`, `src/lib/db/tauri.ts`, `src/lib/db/tauri.test.ts`
- `docs/database.md`, `docs/decisions/010-vault-file-ownership.md`
- `.ai/ARCHITECTURE.md`, `.ai/HANDOFF.md`, `.ai/PROJECT_STATE.md`, `.ai/TODO.md`, `.ai/CHANGELOG.md`, `.ai/SESSION_NOTES.md`

### Out of scope

- Vault list/grid UI and import dialogs (`PHASE6-002`).
- Content parsing, full-text content indexing, thumbnails, previews, OCR, or DOCX support.
- Relinking moved external files, checksums, versioning, or duplicate-content detection.
- Cloud files, sync, or sharing.

### Acceptance criteria

- [x] Migration `006_vault` is transactional, idempotent, constrained, and indexed.
- [x] Records distinguish `linked` and `managed`, support optional Space, title, source filename, media type, size, tags, paths, and timestamps.
- [x] Source imports canonicalize a real regular file and reject directories or paths already owned by the Vault.
- [x] Managed copies are written below the canonical Vault root without blocking React and are cleaned up if persistence fails.
- [x] Linked removal never deletes or modifies the external source file.
- [x] Managed deletion cannot escape the owned item directory and uses quarantine/rollback around database deletion.
- [x] Metadata CRUD, Space filtering, ownership filtering, and search are covered by Rust tests.
- [x] Open and reveal accept only a Vault item ID, resolve its authoritative stored path in Rust, and fail clearly when unavailable.
- [x] Official plugins are initialized; frontend receives only native dialog-open permission, not arbitrary opener-path permission.
- [x] TypeScript contracts and invoke wrappers exactly match the Rust command boundary.
- [x] Existing frontend and Rust quality gates remain green.

### Required verification

- `pnpm check` — pass.
- `pnpm build` — pass.
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` — pass.
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` — pass.
- `cargo test --manifest-path src-tauri/Cargo.toml` — all tests pass, including temporary-directory ownership tests.
- `cargo build --manifest-path src-tauri/Cargo.toml` — pass.
- `git diff --check` — pass.

### Risks and rollback

| Risk | Mitigation or rollback |
| --- | --- |
| An external original is accidentally deleted. | Linked removal contains no filesystem delete path; tests assert the source survives. |
| A crafted relative path escapes managed storage. | Normalize and canonicalize against `<vault>/items/<id>` before every operation; reject containment failures. |
| A large copy freezes the interface. | Execute source inspection and copying through `tauri::async_runtime::spawn_blocking`. |
| Database failure leaves ownership inconsistent. | Clean failed imports; quarantine managed deletion and restore it when the transaction fails. |
| Broad plugin permissions expose arbitrary local paths. | Grant only dialog open to the frontend; opener calls occur in Rust after ID lookup. |

## Implementation result

### Summary

Implemented a complete Vault foundation spanning migration, validated repository, safe linked/managed storage, transactional commands, native dialog/open/reveal integration, and typed frontend IPC. Authoritative storage paths remain private to Rust.

### Files changed

- Added `src-tauri/src/vault.rs`, `src-tauri/src/db/repositories/vault.rs`, and ADR-010.
- Updated the migration, Tauri commands/plugins/capability, Rust and TypeScript contracts, tests, dependency locks, database documentation, and control documents listed in the allowed scope.

### Verification result

- `pnpm check` - pass; 25/25 tests.
- `pnpm build` - pass.
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` - pass.
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` - pass.
- `cargo test --manifest-path src-tauri/Cargo.toml` - pass; 48/48 tests.
- `cargo build --manifest-path src-tauri/Cargo.toml` - pass.
- `git diff --check` - pass.

### Deviations

- The JavaScript opener package and opener capability were deliberately omitted: open/reveal remain trusted Rust commands keyed only by Vault item ID.
- Stored paths are deliberately omitted from serialized Vault items; the frontend does not need filesystem authority.

## Codex self-review

| Field | Value |
| --- | --- |
| Decision | `approved_for_publication` |
| Reviewed at | 2026-08-10 |
| Acceptance evidence | All eleven criteria pass; 48 Rust tests and 25 frontend tests pass with strict build/lint gates. |
| Findings | Initial Windows path assertion compared canonical and non-canonical temp paths; controlled child directories also needed explicit junction-escape protection; frontend opener access and serialized paths were unnecessary authority. |
| Corrections | Corrected canonical-path test, added controlled-directory canonical checks, removed JavaScript opener access, hid stored paths at IPC, and added regression tests. |
| Residual risks | Full native command orchestration is not end-to-end UI-tested until `PHASE6-002`; repository and filesystem primitives are independently covered. |
