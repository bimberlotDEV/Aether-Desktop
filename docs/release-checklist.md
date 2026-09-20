# Aether release-candidate checklist

Use this checklist for every internal, beta or later candidate. Record command output or hashes in `.ai/PROJECT_STATE.md` and the versioned artifact document.

## Automated gates

- [ ] `pnpm install --frozen-lockfile`
- [ ] `pnpm check`
- [ ] `pnpm build`
- [ ] `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`
- [ ] `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`
- [ ] `cargo test --manifest-path src-tauri/Cargo.toml`
- [ ] `cargo build --manifest-path src-tauri/Cargo.toml --release`
- [ ] `pnpm audit --audit-level high`
- [ ] `pnpm tauri:build`
- [ ] `git diff --check`

## Product and safety review

- [ ] Keyboard focus remains visible and primary controls have accessible names/states.
- [ ] Loading, empty, error, cancellation, retry, and destructive confirmation paths are present.
- [ ] No secrets, `.env` files, databases, logs, private keys, or generated bundles are tracked.
- [ ] Tauri capabilities and CSP match the operations actually used.
- [ ] AI privacy boundaries and backup exclusions are visible before the action.
- [ ] A complete archive round-trip preserves managed Vault bytes and workspace counts while omitting credentials and linked-file bytes.
- [ ] Restore preview, one-time approval, safety archive, restart swap, interrupted-swap recovery, traversal/duplicate/hash/schema rejection, and post-restore Vault ownership pass in isolated app data.
- [ ] Release executable starts and remains alive long enough to render.
- [ ] MSI and NSIS bundles exist and SHA-256 hashes are recorded.
- [ ] Version, license, README, security policy, changelog, and architecture agree.
- [ ] Sanitized diagnostics contain exactly the documented allowlist and no content, paths, logs, identifiers, counts, credentials or automatic submission.
- [ ] Public issue forms and beta documentation warn against sharing private workspace data and route vulnerabilities privately.

## Owner-controlled release gates

The repository contains the protected draft-release and Rust-owned updater implementation, but live activation remains blocked until the owner provisions the trust inputs below. Enabling or bypassing them is a release blocker, not an optional shortcut.

- [ ] The GitHub `public-release` Environment exists, requires an owner reviewer, restricts deployment to `master`, and contains the documented secrets/variables.
- [ ] The Tauri updater private key has two protected recovery copies; only its matching public key is supplied to release configuration.
- [ ] The Azure Artifact Signing identity is active, least-privilege credentials are current, and its publisher identity matches the expected Aether owner.
- [ ] The **Public Windows release** workflow is manually dispatched from `master` with the exact committed version and completes without a failed verification step.
- [ ] The generated GitHub Release remains a draft until every clean-install, previous-Stable upgrade, in-app update, data-preservation, signature, hash, and rollback check in `docs/release-runbook.md` passes.
- [ ] Publishing is a deliberate owner action. Ordinary pushes, tags, pull requests, and CI never publish a release.

## Public beta evidence gates

- [ ] Every applicable scenario in `docs/beta-test-matrix.md` passes on signed artifacts using clean and protected-upgrade Windows accounts.
- [ ] Testers received prerelease/privacy/backup limitations and consented to privacy-minimal evidence collection.
- [ ] Completed evidence ledgers remain owner-private and contain no names, emails, private content, paths, machine identifiers, keys, databases, backups or full logs.
- [ ] No unresolved stop-ship defect exists and high-severity issues have deterministic regression coverage.
- [ ] Multiple real external testers activated with meaningful local work and returned voluntarily; appearance-only feedback is not treated as retention.
- [ ] The owner deliberately records whether Public Beta exit evidence is sufficient. A green repository or signed draft alone is not sufficient.
