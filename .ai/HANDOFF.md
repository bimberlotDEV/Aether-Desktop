# Codex Task Contract

| Field             | Value                          |
| ----------------- | ------------------------------ |
| Schema version    | 2                              |
| Task ID           | `RELEASE-040`                  |
| Status            | `in_progress`                  |
| Owner             | Codex                          |
| Last updated      | 2026-08-28                     |
| Related milestone | Integrated Alpha 0.4.0 release |
| Classification    | `planned_codex`                |

## Objective

Cut, verify, install, and publish Aether Alpha 0.4.0 from merged `master`, preserving the owner's existing local database and installation while producing traceable unsigned Windows MSI and NSIS artifacts.

## Context

Milestones A–G are merged through PR #43 at `da4ca98`; pull-request and post-merge Windows CI pass. The merged product remains labeled 0.3.2 in package, Tauri, Rust, and UI metadata, so the new functionality has no honest Windows upgrade boundary or artifact record. The current public trust infrastructure still lacks owner-controlled code-signing and updater keys, which remain explicitly out of scope.

## Ordered checkpoints

1. **Release identity:** reconcile project state after PR #43 and update every machine/UI release identifier to numeric SemVer 0.4.0.
2. **Release gates:** run frozen dependency install, frontend/Rust quality, security audit, production/release builds, package creation, diff/secret review, and artifact hashing.
3. **Protected upgrade:** stop Aether cleanly, copy the existing install/data to a timestamped local backup outside the repository, verify the backup, silently install 0.4.0, and confirm database preservation plus responsive startup.
4. **Closure:** record exact artifacts and evidence, self-review every criterion, publish a draft PR, and require exact-head Windows CI.

## Acceptance criteria

- [x] `package.json`, Tauri configuration, Rust package/lock metadata, Settings, Sidebar, README, and native release documentation consistently identify Alpha 0.4.0; the pnpm lock has no root-version field and remains frozen-valid.
- [x] The release remains an unsigned Alpha with updater artifacts disabled; no signing secret, placeholder key, endpoint, or false public-release claim is introduced.
- [x] Frozen install, frontend typecheck/lint/tests/build, high-severity dependency audit, Rust format/strict Clippy/tests/release build, Tauri packaging, and diff/security checks pass from the merged tree.
- [x] MSI and NSIS 0.4.0 x64 artifacts exist and their sizes and SHA-256 hashes are recorded in a dedicated release artifact document.
- [x] Before installation, the active Aether process is stopped and the existing installation plus the runtime-resolved `%APPDATA%/com.aether.desktop` data are copied to a timestamped directory outside the repository with verifiable database size/hash evidence.
- [x] Installing 0.4.0 succeeds without deleting or corrupting existing workspace data; the post-install database passes SQLite integrity validation and its exact preservation outcome is recorded.
- [x] The installed executable reports product version 0.4.0, starts from the installed location, remains responsive, and preserves the existing workspace row counts.
- [x] No database, credential, backup, generated bundle, private key, environment file, or unrelated worktree change is committed.
- [ ] Release documentation, `.ai` state, changelog, backlog, branch/head evidence, and exact-head GitHub Windows CI agree.

## Allowed paths

- `package.json`
- `pnpm-lock.yaml`
- `src-tauri/Cargo.toml`
- `src-tauri/Cargo.lock`
- `src-tauri/tauri.conf.json`
- `src/components/Sidebar.tsx`
- `src/routes/Settings.tsx`
- `README.md`
- `AGENTS.md`
- `docs/database.md`
- `docs/decisions/010-vault-file-ownership.md`
- `docs/native-desktop.md`
- `docs/release-checklist.md`
- `docs/release-artifacts-0.4.0.md`
- `.ai/*`

## Non-goals

- Code signing, automatic updates, GitHub Release publication, public store distribution, telemetry, accounts, cloud sync, new product features, provider credentials, or live paid AI calls.
- Changing schemas, migrations, application behavior, visual design, dependency versions, installer identity, data locations, backup format, or Vault ownership semantics.
- Deleting the previous installation backup or removing user data during uninstall/reinstall.

## Risks and safeguards

- **Personal data loss:** close the process, resolve exact runtime data/install paths instead of trusting stale documentation, copy rather than move, hash the database before/after, run integrity checks, and retain the backup.
- **Windows upgrade mismatch:** use 0.4.0 consistently in package/Tauri/Rust metadata and verify installed file metadata after the silent installer.
- **False trust:** continue to label artifacts unsigned and keep updater creation disabled.
- **Generated-file leakage:** bundles, databases, backups, and credentials stay ignored/outside Git and are checked before publication.
- **Rollback:** retain the complete pre-install copy; the previous 0.3.2 installer remains available locally. Restore requires an explicit owner instruction if needed.

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
git diff --check
artifact SHA-256 and version metadata checks
offline pre-install backup and SQLite integrity verification
silent NSIS upgrade and installed startup smoke
GitHub PR exact-head CI
```

## Blocking decisions

None. The owner explicitly authorized the next milestone after accepting the recommended 0.4.0 release sequence. Unsigned Alpha status and lack of updater trust remain visible constraints, not hidden release claims.

## Readiness review

- **Status:** Ready. Version scope, artifacts, data protection, validation, publication, rollback, and excluded trust infrastructure are explicit.
- **Architecture gate:** No new ADR is required; this applies existing ADR-013/014 release and backup constraints without changing architecture.
- **Data gate:** Installation may proceed only after a verified, non-destructive backup outside the repository.
- **Publication gate:** A draft PR and exact-head CI are required; no GitHub Release or merge is implied.

## Implementation evidence and self-review

- Version identifiers and user-visible labels are consistently 0.4.0; historical 0.3.x artifact documents remain intentionally unchanged.
- The release patch changes metadata and documentation only. Application behavior, database schema, permissions, CSP, dependencies, installer identity, data paths, and updater state are unchanged.
- Frozen frontend gates pass with 77/77 tests and no known audited vulnerabilities; native gates pass with strict Clippy and 95/95 tests.
- MSI/NSIS artifacts exist with recorded size/hash evidence and remain unsigned; `createUpdaterArtifacts` is still `false`.
- A verified external pre-install backup retained the full runtime app-data and installed directories. Source/backup hashes, SQLite integrity, and domain row counts match.
- Silent installation returned zero, preserved the database byte-for-byte before first launch, registered 0.4.0, and started a responsive installed 0.4.0 process.
- Review corrected a pre-existing documentation mismatch: Tauri resolves the configured identifier to `%APPDATA%/com.aether.desktop`, which is now used consistently by release and database documentation.
- No unresolved local acceptance finding remains; exact-head GitHub CI is the final publication gate.
