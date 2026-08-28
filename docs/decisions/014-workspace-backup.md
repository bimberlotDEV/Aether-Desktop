# ADR-014 — Sanitized workspace database backup

- **Status:** Accepted
- **Date:** 2026-08-10
- **Extended by:** ADR-022 (portable managed-file archives and safe restore)

## Decision

The alpha exports a transactionally consistent SQLite snapshot chosen through a native save dialog. The exported database excludes the `secrets` table, keeps local workspace records intact, and is integrity-checked before rollback-safe replacement of the chosen destination.

Managed Vault file bytes and linked external files are not copied in this first foundation. Their metadata remains in the snapshot and the UI states this limitation before export. Restore/import is deliberately not automated until conflict, version, and managed-file semantics have a separate reviewed design.

ADR-022 completes that later design. This database-only command remains available for compatibility, while the primary Settings workflow now creates and restores verified `.aether-backup` archives.

## Security

- Destination must be absolute, use the `.aether-backup.db` suffix, and differ from the live database.
- UUID-named sibling partial and previous files prevent a failed final rename from destroying an existing backup.
- API credentials are removed from the snapshot even though production values are DPAPI-protected.
- The command returns only byte count and timestamp, not arbitrary filesystem contents.
