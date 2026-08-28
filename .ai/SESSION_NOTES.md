# Session Notes

| Field          | Value                |
| -------------- | -------------------- |
| Schema version | 2                    |
| Session date   | 2026-08-28           |
| Active task    | `BACKUP-RESTORE-001` |
| Agent          | Codex                |
| State          | `complete`           |

## Current work

- PR #44 merged Alpha 0.4.0 into `master` at `d7299ce`; post-merge Windows CI run `33186214700` passes.
- Local `master` and `origin/master` are synchronized; task branch is `codex/backup-restore`.
- Classified Milestone H as `planned_codex` because archive parsing, restore, credentials, live SQLite replacement, and Vault ownership can cause data loss or security failures.
- Audited ADR-014, SQLite/WAL startup, backup command/UI, migrations, managed/linked Vault paths, containment helpers, Tauri restart support, and current dependency surface.
- Accepted ADR-022 and completed the ready `BACKUP-RESTORE-001` contract before production code changes.
- Implemented strict portable archives containing sanitized SQLite plus exact managed Vault bytes; linked files and credentials remain excluded.
- Added untrusted-archive validation, expiring one-time preview tokens, separate approval, staged migration, current credential preservation, complete recovery archives, and restart-bound replacement.
- Added idempotent recovery for a process interruption between database and Vault activation and revalidated exact managed-file trees before every swap.
- Replaced Settings backup UX with accessible complete backup/restore controls and explicit replace-not-merge consequences.
- Passed frozen install, 80 frontend tests, production build/audit, Rust format/strict Clippy, 99 Rust tests, release build, MSI/NSIS packaging, responsive light/dark UI smoke, security diff review, and exact-head CI run `33194814850`.
- Published implementation commit `1cb3b90` in draft PR #45.

## Exact resume point

`BACKUP-RESTORE-001` is complete. Leave draft PR #45 unmerged until explicit owner authorization. The next public-release milestone is owner-controlled signing and updater activation; no trust keys or endpoints should be invented.
