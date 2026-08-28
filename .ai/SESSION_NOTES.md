# Session Notes

| Field          | Value                |
| -------------- | -------------------- |
| Schema version | 2                    |
| Session date   | 2026-08-28           |
| Active task    | `BACKUP-RESTORE-001` |
| Agent          | Codex                |
| State          | `in_progress`        |

## Current work

- PR #44 merged Alpha 0.4.0 into `master` at `d7299ce`; post-merge Windows CI run `33186214700` passes.
- Local `master` and `origin/master` are synchronized; task branch is `codex/backup-restore`.
- Classified Milestone H as `planned_codex` because archive parsing, restore, credentials, live SQLite replacement, and Vault ownership can cause data loss or security failures.
- Audited ADR-014, SQLite/WAL startup, backup command/UI, migrations, managed/linked Vault paths, containment helpers, Tauri restart support, and current dependency surface.
- Accepted ADR-022 and completed the ready `BACKUP-RESTORE-001` contract before production code changes.

## Exact resume point

Begin checkpoint 1 in `src-tauri/src/backup.rs`: introduce the strict archive manifest/writer/reader and retain the legacy sanitized DB export for compatibility. Do not touch the installed 0.4.0 personal workspace during development.
