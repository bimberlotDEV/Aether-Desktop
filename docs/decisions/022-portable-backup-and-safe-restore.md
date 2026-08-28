# ADR-022 — Portable workspace archives and restart-bound restore

- **Status:** Accepted
- **Date:** 2026-08-28

## Context

ADR-014 introduced a consistent, sanitized SQLite export but deliberately excluded managed Vault bytes and automated restore. A complete personal-workspace backup must now move the database and Aether-owned files together without exporting credentials, following linked external files, trusting archive paths, or replacing files that are open by the running application.

## Decision

Aether adds a versioned `.aether-backup` ZIP container with a strict manifest, a sanitized SQLite snapshot, and only managed Vault file bytes. Every payload entry has an exact relative path, byte length, and SHA-256 digest. Linked Vault items remain metadata-only external references and are disclosed in preview and export results.

Restore is a closed two-phase Rust workflow:

1. Preview canonicalizes and fingerprints the selected archive, rejects unknown/duplicate/unsafe entries, enforces size and count limits, verifies every digest, checks SQLite integrity and migration compatibility, and validates managed Vault metadata against the packaged bytes.
2. Preview issues a short-lived, one-time token bound to that exact archive. Approval revalidates the fingerprint, stages and migrates the replacement database, preserves only the current device-local `secrets` rows inside the staged database, creates a complete recovery archive of the current workspace, and writes a generated pending-restore marker.
3. Aether restarts. Before opening SQLite, startup applies the staged database and managed Vault tree through contained renames. A failed swap restores the prior live paths. Only after the swap succeeds does normal database initialization continue.

The archive reader never extracts caller-controlled paths. It accepts only `manifest.json`, `workspace.db`, and manifest-declared `vault/items/<item-id>/<filename>` entries whose normalized ownership agrees with the database. Extra entries, symlinks, traversal, absolute paths, unsupported formats, newer migration sets, and payload mismatches are rejected.

## Consequences

- Backup becomes portable for all Aether-owned workspace content while linked files remain explicitly external.
- API keys never enter archives. Restoring on the same Windows profile retains current device credentials; restoring elsewhere starts without credentials.
- Restore replaces the workspace as one unit after explicit approval and restart; it is not a merge/import operation.
- A verified recovery archive is retained locally so an operator can recover even after a later startup problem.
- ZIP and SHA-256 support become direct native dependencies; archive work runs off the WebView thread.

## Rejected alternatives

- **Overwrite the live database from a command:** unsafe with SQLite WAL and open handles, and cannot make Vault bytes atomic.
- **Extract arbitrary ZIP paths:** creates traversal, symlink, overwrite, and archive-bomb risk.
- **Merge rows into the current database:** conflict semantics differ by domain and can silently break Space and Memory isolation.
- **Include credentials:** violates the existing backup trust boundary and Windows-user binding.
- **Copy linked files:** exceeds Aether ownership and user consent.

## Rollback

Remove the new archive/restore commands and UI while retaining legacy sanitized database export support. Pending restore state is ignored only after manually restoring its generated recovery directory; no schema change is required.
