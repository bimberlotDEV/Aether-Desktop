# Codex Task Contract

| Field             | Value                             |
| ----------------- | --------------------------------- |
| Schema version    | 2                                 |
| Task ID           | `RELEASE-032`                     |
| Status            | `self_review`                     |
| Owner             | Codex                             |
| Last updated      | 2026-08-25                        |
| Related milestone | Alpha 0.3.2 release consolidation |
| Classification    | `planned_codex`                   |

## Objective

Publish a reproducible Aether 0.3.2 Alpha release candidate from merged `master`, install it as a verified upgrade on this Windows PC without losing user data, and publish exact artifacts, hashes, release notes, and validation evidence for review.

## Context

PR #34 (AI response reconciliation), PR #35 (personal-beta database upgrades), and PR #36 (complete interface makeover) are merged in order on `master` at `3748947`, and every post-merge Windows CI run passed. The installed application already contains the locally integrated 0.3.1 candidate, but repository metadata still identifies it as 0.3.1 and the final merged source has not been emitted as a separately versioned release. A new patch version gives Windows a clear upgrade boundary and makes the integrated release auditable.

## Acceptance criteria

- [x] `package.json`, Rust package metadata, Cargo lock metadata, Tauri configuration, and visible in-app version labels agree on `0.3.2`; the pnpm lockfile has no root-version field and passes frozen installation unchanged.
- [x] Product changelog and release artifact documentation describe AI response reconciliation, safe personal-beta upgrades, and the interface makeover without overstating public signing or updater support.
- [x] Frontend typecheck, lint, 56 tests, production build, dependency audit, Rust formatting, strict Clippy, 63 Rust tests, release build, and Tauri packaging all pass from the release branch.
- [x] The generated x64 MSI, NSIS installer, and release executable exist and have recorded byte sizes and SHA-256 hashes.
- [x] No generated binary, database, credential, environment file, log, private key, or user backup is staged or committed.
- [x] A complete pre-upgrade copy of `%APPDATA%/com.aether.desktop` is created outside the repository after stopping the verified process and before replacing the installed app.
- [x] The NSIS installer exits successfully, Windows installs version 0.3.2, and the installed executable starts and remains responsive.
- [x] The existing database remains present after upgrade with the same or greater file size; migration safety is additionally covered by the 63-test Rust suite.
- [x] The final diff is limited to release metadata, visible version labels, release/control documentation, and generated lock metadata.
- [ ] The verified work is committed, pushed, represented by a draft PR, and receives a green refreshed GitHub CI run.

## Allowed paths

- `package.json`
- `pnpm-lock.yaml`
- `src-tauri/Cargo.toml`
- `src-tauri/Cargo.lock`
- `src-tauri/tauri.conf.json`
- `src/components/Sidebar.tsx`
- `src/routes/Settings.tsx`
- `CHANGELOG.md`
- `README.md`
- `docs/native-desktop.md`
- `docs/release-artifacts-0.3.2.md`
- `.ai/CHANGELOG.md`
- `.ai/HANDOFF.md`
- `.ai/PROJECT_STATE.md`
- `.ai/SESSION_NOTES.md`
- `.ai/TODO.md`

## Non-goals

- Adding product features, schema migrations, dependencies, permissions, signing, an updater, telemetry, restore behavior, or Vault-byte backup semantics.
- Publishing a GitHub Release or marking the draft PR merged without a separate owner request.
- Sending a live DeepSeek request with the owner's credentials.
- Removing old installers, previous backups, legacy Vault recovery copies, or user content.

## Dependencies and evidence

- Merged baseline: `3748947` on `master`.
- Successful post-merge Windows CI for PRs #34, #35, and #36.
- Existing release checklist in `docs/release-checklist.md`.
- Existing 0.3.1 metadata and artifact-record pattern.
- Existing verified backup root under `%LOCALAPPDATA%/AetherInstallBackups`.

## Risks and safeguards

- **Installer replacement risk:** resolve exact source, backup, installer, and installed executable paths before mutation; stop only the verified `aether.exe` process.
- **User-data risk:** make a complete timestamped backup before installation and never delete the source or backup.
- **Version drift:** search all first-party 0.3.1 references, update only machine/version-display references, and verify locks through native package tooling.
- **Artifact ambiguity:** hash exact release executable, MSI, and NSIS outputs after the final build and document their full filenames.
- **Unsigned distribution:** preserve the existing explicit unsigned/no-updater limitation.
- **Rollback:** the pre-upgrade backup and previous 0.3.1 installer remain available; reverting the release commit restores repository metadata.

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
SHA-256 and byte-size capture for release artifacts
pre-install backup verification
silent NSIS upgrade and installed process/version smoke
GitHub PR CI
```

## Blocking decisions

None. The owner explicitly authorized the next release step. Public signing, automatic updates, and GitHub Release publication remain excluded because they require separate owner-controlled trust material or publication direction.

## Self-review record

- **Status:** Local self-review passed; publication and refreshed GitHub CI remain pending.
- **Version and scope:** All first-party machine/display sources resolve to 0.3.2. Remaining 0.3.1 references are historical changelog, completed-task evidence, dependency versions, and the previous artifact record. The changed paths match the contract and add no dependency, permission, migration, or runtime behavior.
- **Automated evidence:** Frozen install, typecheck, lint, 56/56 frontend tests, production build, clean high-severity audit, Rust formatting, strict Clippy, 63/63 Rust tests, optimized release build, Tauri MSI/NSIS packaging, and `git diff --check` pass.
- **Artifact evidence:** Loose executable: 16,096,768 bytes / `1AFF3FFA67E741B2B13A5CB59C13A171DA39250F184449F7CE4A762CACA450E7`; MSI: 7,086,080 bytes / `C5A02C8FEFA9A99D55BF2130E421EFFA310BF0B677A16EF56DDE6974BDB3FEFB`; NSIS: 4,228,661 bytes / `D0402AB4ADAC9E7A392957CD0A84404CF9C53ABDB691BFD9B603C37E91E7BE2B`. All are explicitly unsigned.
- **Data and install evidence:** Backup `pre-release-032-20260825-220627` matches the source database at 258,048 bytes and SHA-256 `F2808F56E2D421A54745118D880B566A3809B0632018806C4C9C1DD8A6E3BD2F`. The same offline database size/hash remained after the NSIS exit-code-0 upgrade; installed product version is 0.3.2 and its process is responsive.
- **Findings:** Corrected the initial lockfile criterion after confirming pnpm's importer does not encode the root package version. No secret, generated binary, database, backup, unsafe operation, or unrelated change is present in the diff.
