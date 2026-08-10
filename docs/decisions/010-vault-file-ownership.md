# ADR-010: Vault file ownership and safe deletion

- **Status:** Accepted
- **Date:** 2026-08-10
- **Decision owner:** Codex within the approved Aether roadmap
- **Related work:** `VAULT-EPIC`, `PHASE6-001`, `PHASE6-002`

## Context

Vault must support both references to existing local files and copies owned by Aether. Those modes have materially different deletion semantics. A linked file remains owned by the user at its original location; a managed file lives below Aether's application-data directory. The UI cannot be trusted to enforce that boundary because all filesystem mutations execute in Rust.

## Decision

1. Vault records use one `vault_items` table with an explicit `storage_mode` of `linked` or `managed`.
2. Linked items store a canonical absolute path. Removing a linked item deletes only its database record and never touches the source file.
3. Managed items store a relative path below `%APPDATA%/Aether/vault/items/<item-id>/`. Import copies run on a blocking worker, use a partial file followed by an atomic rename, and clean up on failure.
4. Every managed open, reveal, or delete operation resolves the stored relative path against the canonical Vault root and rejects traversal or ownership-boundary violations.
5. Managed deletion first moves the owned item directory into a Vault-local quarantine, then deletes the database record transactionally. A database failure restores the directory; successful records may leave only a safe orphan in quarantine if final cleanup fails.
6. Native file selection uses the official Tauri dialog plugin. Opening and revealing use the official opener plugin from a trusted Rust command after resolving an item by database ID; arbitrary frontend paths are never accepted for those operations.
7. Stored paths remain internal to Rust and are not serialized across IPC. The webview receives an item ID and display metadata only.
8. Metadata search covers display title, original filename, media type, and tags. File content parsing and indexing are deferred.
9. Space ownership is optional and uses `ON DELETE SET NULL` so removing a Space never removes a file or Vault record.

## Consequences

- The ownership badge is authoritative and safe deletion behavior is testable below the UI.
- External files may become unavailable when moved or deleted outside Aether; the record remains until the user removes or relinks it.
- Managed copies consume Aether app-data storage and are deliberately independent of their original source after import.
- Content extraction, checksums, duplicate detection by content, previews, and relinking are future extensions.

## Rejected alternatives

- Treat every file as linked: fails offline stability and managed-storage requirements.
- Treat every file as managed: duplicates user data without an explicit choice.
- Let the frontend delete paths directly: violates the ownership boundary and makes original-file deletion possible.
- Store absolute managed paths: makes data-directory moves brittle and weakens containment checks.
