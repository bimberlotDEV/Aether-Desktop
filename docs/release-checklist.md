# Alpha release checklist

Use this checklist for every Alpha candidate. Record command output or hashes in `.ai/PROJECT_STATE.md` and the versioned artifact document.

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

## Owner-controlled release gates

Public signing and automatic updates stay disabled until the repository owner provisions a protected Windows code-signing identity, Tauri updater private key, HTTPS release endpoint, and recovery procedure. Enabling either without those controls is a release blocker, not an optional shortcut.
