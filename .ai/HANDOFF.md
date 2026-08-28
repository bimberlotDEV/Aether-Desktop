# Codex Task Contract

| Field             | Value                                   |
| ----------------- | --------------------------------------- |
| Schema version    | 2                                       |
| Task ID           | `BACKUP-RESTORE-001`                    |
| Status            | `self_review`                           |
| Owner             | Codex                                   |
| Last updated      | 2026-08-28                              |
| Related milestone | Milestone H — Complete Backup & Restore |
| Classification    | `planned_codex`                         |

## Objective

Make Aether's local workspace genuinely recoverable and portable by exporting a verified archive containing the sanitized SQLite workspace and all Aether-managed Vault bytes, then restoring that archive only through an explicit, rollback-safe, restart-bound workflow.

## Context

Alpha 0.4.0 is merged at `d7299ce` and post-merge Windows CI run `33186214700` passes. ADR-014's `.aether-backup.db` export is consistent and omits credentials, but it excludes managed Vault bytes and has no restore path. Managed Vault records store constrained paths below `vault/items/<item-id>/`; linked records point outside Aether ownership. SQLite is open in WAL mode for the process lifetime, so safe replacement must occur before startup opens the database.

## Ordered checkpoints

1. **Archive foundation:** add a versioned manifest and streaming archive writer/reader with strict entry allowlists, limits, SHA-256 verification, sanitized SQLite backup, managed Vault ownership checks, and no linked-file or credential bytes.
2. **Restore boundary:** add canonical archive preview, compatibility/integrity validation, short-lived one-time approval tokens, exact-file revalidation, staged migration, current-credential preservation, and a complete pre-restore recovery archive.
3. **Restart recovery:** apply the generated pending restore before database open using contained filesystem swaps with rollback, then restart into the replacement workspace without accepting frontend paths for live destinations.
4. **Product integration:** expose typed wrappers and an accessible Settings flow that clearly distinguishes replacement from merge, summarizes content/exclusions, requires explicit confirmation, reports errors, and disables unsupported browser behavior.
5. **Closure:** verify malicious archives, missing/corrupt bytes, old/current/newer schemas, failure rollback, UI behavior, full frontend/native gates, package/startup smoke, self-review, documentation, and exact-head GitHub CI.

## Acceptance criteria

- [x] A `.aether-backup` archive contains a format-versioned manifest, integrity-checked SQLite snapshot, and exactly the managed Vault files referenced by that snapshot; every payload records size and SHA-256.
- [x] `secrets` and API-key material never enter the archive, linked file bytes are never read/copied, absolute source paths are absent from the manifest, and the UI discloses linked-file portability limits.
- [x] Export uses a native save choice, runs blocking work outside the WebView thread, never targets live Aether storage, and replaces an existing destination with rollback-safe partial/previous files.
- [x] Preview treats archives as untrusted: it canonicalizes the selected file, rejects traversal, absolute paths, symlinks, duplicates, unknown entries, excessive count/size, digest mismatches, malformed manifests, missing/extra managed files, corrupt databases, secrets tables, and unsupported/newer schemas.
- [x] Restore is workspace replacement, never row merging. It requires a visible preview plus a separate explicit approval backed by a short-lived one-time Rust token bound to the exact archive fingerprint.
- [x] Approval revalidates the archive, migrates and integrity-checks the staged database, preserves current device-local credential rows without exporting them, and refuses to restart until a complete verified recovery archive of the current database and managed Vault bytes exists.
- [x] Startup applies only a generated pending restore inside the app-data boundary before SQLite opens; database/WAL files and managed Vault items swap together, failures restore previous live paths, and arbitrary frontend destination paths cannot reach the swap implementation.
- [x] Restore preserves Space, Note, Task, Memory, AI, Source, Activity, Action, and Vault metadata isolation. Managed files resolve under their matching item ownership after restore; linked records remain external and may honestly report unavailable files.
- [x] The Settings workflow is keyboard accessible, communicates replacement/restart/recovery consequences, prevents duplicate work, handles cancellation and errors, and never claims success before restore has been safely staged.
- [x] Legacy `.aether-backup.db` export remains available at the native boundary for compatibility, but the primary UI and current documentation use the complete archive.
- [ ] Frontend checks/build/audit, Rust format/strict all-target Clippy/tests/release build, Tauri packaging, malicious-archive tests, packaged restore startup smoke, diff/security review, and exact-head GitHub CI pass.
- [ ] No personal database, Vault byte, archive, recovery directory, credential, generated bundle, secret, or unrelated line-ending-only worktree change is committed.

## Allowed paths

- `src-tauri/Cargo.toml`
- `src-tauri/Cargo.lock`
- `src-tauri/src/backup.rs`
- `src-tauri/src/commands.rs`
- `src-tauri/src/lib.rs`
- `src-tauri/src/db/mod.rs`
- `src-tauri/src/db/migrations.rs`
- `src-tauri/src/vault.rs`
- `src/lib/db/types.ts`
- `src/lib/db/tauri.ts`
- `src/lib/db/tauri.test.ts`
- `src/components/BackupSettings.tsx`
- `src/components/BackupSettings.test.tsx`
- `src/routes/Settings.tsx`
- `README.md`
- `docs/database.md`
- `docs/release-checklist.md`
- `docs/decisions/014-workspace-backup.md`
- `docs/decisions/022-portable-backup-and-safe-restore.md`
- `.ai/*`

## Non-goals

- Cloud backup, sync, schedules, encryption/password protection, credential portability, linked-file copying, partial-domain restore, merge/import conflict resolution, archive browsing, automatic retention cleanup, or public updater/signing activation.
- Changing domain schemas, Space/Memory/AI isolation semantics, Vault ownership rules, user-selected live data locations, installer identity, or current release version.
- Applying a restore to the owner's live installed workspace during development; packaged restore smoke uses isolated temporary app data.

## Risks and safeguards

- **Data loss:** no live replacement occurs until staging, migration, integrity checks, and a complete recovery archive pass; startup swaps generated contained paths and rolls back partial moves.
- **Archive attacks:** fixed entry grammar, duplicate/extra rejection, symlink rejection, bounded streaming, size/count ceilings, exact hashes, and no generic extraction.
- **TOCTOU:** token binds canonical path, file identity metadata, and SHA-256; approval fully revalidates before staging.
- **Credential exposure/loss:** archive snapshots drop `secrets`; approval copies current rows directly into the local staged DB and never serializes them into archive/IPC/logs.
- **Ownership escape:** managed entries must equal the database's `items/<id>/<filename>` path; linked targets are never opened by backup.
- **Schema mismatch:** unknown/newer migrations are rejected; known older archives migrate while staged, never after the live swap.
- **Concurrency:** export/restore preparation runs on blocking workers and uses a single native restore runtime to prevent overlapping previews/executions.
- **Rollback:** generated recovery archive and contained swap rollback preserve the pre-restore workspace; legacy DB-only export remains callable.

## Required validation

```text
pnpm install --frozen-lockfile
pnpm check
pnpm build
pnpm audit --audit-level high
cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all-targets --all-features
cargo build --manifest-path src-tauri/Cargo.toml --release
pnpm tauri:build
isolated packaged backup/restore/restart smoke
git diff --check
archive/path/secret/security diff review
GitHub PR exact-head CI
```

## Blocking decisions

None. The owner authorized the next milestone. Replacement rather than merge follows the existing deferred restore boundary, managed-only bytes follow Vault ownership, and restart-bound application is the reversible safe option for a process-lifetime SQLite connection.

## Readiness review

- **Status:** Ready. Archive format, exclusions, compatibility, consent, staging, restart, rollback, limits, tests, UI, and non-goals are explicit.
- **Architecture gate:** ADR-022 is accepted before production implementation.
- **Migration gate:** no new schema is introduced; restored older known schemas migrate only while staged.
- **Security gate:** Rust owns archive parsing, path containment, credential handling, approval tokens, staging, and live replacement.
- **Worktree gate:** apparent frontend modifications are existing Windows line-ending/index noise with no content diff; task-owned paths will be staged explicitly.

## Implementation evidence and self-review

- Archive writer/reader tests round-trip a current SQLite workspace plus managed bytes while proving the snapshot has no `secrets` table and linked bytes are absent.
- Restore tests cover one-time tokens, exact archive revalidation, current credential preservation, managed ownership, traversal, duplicate names, corrupt bytes, unsafe destinations, newer schemas, normal restart application, and a simulated process crash between database and Vault activation.
- `pnpm install --frozen-lockfile`, `pnpm check` (80/80), production build, and high-severity audit pass.
- Rust format, strict all-target/all-feature Clippy, 99/99 tests, release build, and MSI/NSIS packaging pass. RustSec reports no vulnerabilities; 18 allowed pre-existing cross-platform/unmaintained/yanked warnings remain, none introduced by `zip` or `sha2`.
- Settings browser smoke passes at 1024×640 and 720×640 in light/dark themes with no console warnings/errors. The browser fallback is honest and disabled; component tests exercise the native preview, cancel, and approval UI states.
- The restart swap is exercised against isolated temporary app-data paths in native tests. A second packaged executable could not use temporary `%APPDATA%` because Tauri resolves Windows known folders and enforces one instance; the attempt created no isolated database and was stopped. The owner's live workspace was not restored or overwritten.
- Self-review found and fixed an interrupted-swap crash window, exact managed-tree extra-file acceptance, unsafe Windows filename components, unbounded release metadata, and non-regular destination replacement before final gates.
- Diff/security review confirms every recursive removal and live rename targets generated app-data children, approval accepts only an opaque token, credentials never enter archive/IPC/logs, linked files are never opened, and no archive/database/generated bundle is in the publishable diff.
- Remaining closure gates: task-owned staging/publication and exact-head GitHub CI.
