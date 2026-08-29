# Codex Task Contract

| Field             | Value                                    |
| ----------------- | ---------------------------------------- |
| Schema version    | 2                                        |
| Task ID           | `RELEASE-TRUST-001`                      |
| Status            | `self_review`                            |
| Owner             | Codex                                    |
| Last updated      | 2026-08-29                               |
| Related milestone | Milestone I — Trusted Releases & Updates |
| Classification    | `planned_codex`                          |

## Objective

Give Aether a production-grade, owner-gated Windows release path and an explicit in-app update experience without weakening its local-first defaults or placing signing material, release authority, update URLs, or installer execution under frontend control.

## Context

Milestone H is merged on `master` at `76947db`; its exact-head PR CI passed and post-merge CI is expected to run. Aether 0.4.0 currently builds MSI/NSIS installers but deliberately emits no updater artifacts, contains no updater plugin/configuration, and cannot be Authenticode-signed without an owner-controlled identity. Tauri requires updater signatures and a public verification key; Windows public distribution separately requires a trusted code-signing identity. Those trust roots must remain external to Git.

## Ordered checkpoints

1. **Trust architecture:** document the two independent trust layers, fixed Stable GitHub Release feed, manual protected release environment, generated untracked release configuration, bootstrap limitation, recovery/rotation rules, and explicit owner activation gate.
2. **Release tooling:** add strict preflight/config-generation/signing scripts and a manually dispatched Windows workflow that validates the exact version/ref, runs all gates, requires owner secrets/variables, builds signed MSI/NSIS updater artifacts, verifies signatures and Authenticode, and creates only a draft GitHub Release.
3. **Native updater boundary:** integrate the maintained Tauri updater in Rust, keep development builds disabled/network-silent, expose narrow check/install/cancel commands, retain update objects in native state, serialize operations, stream bounded progress, verify before install, and restart only after successful installation.
4. **Product experience:** add typed IPC schemas/wrappers plus an accessible Settings surface for Stable-channel status, manual check, release notes, explicit install confirmation, progress, cancellation/error handling, and honest unconfigured/bootstrap messaging.
5. **Release candidate:** reconcile version metadata for 0.5.0, validate ephemeral updater signing and rejection without committing keys, run full frontend/native/security/package/UI gates, self-review, document the external activation steps, and publish a draft task PR with exact-head CI.

## Acceptance criteria

- [x] Development and ordinary CI builds remain updater-disabled and make no release-network request; production updater configuration is generated into an ignored file only after strict owner-input validation.
- [x] The Stable feed is fixed to the repository's HTTPS GitHub `latest.json`; no arbitrary endpoint, insecure transport, downgrade, prerelease channel, frontend URL, or unsigned metadata path is accepted.
- [x] Tauri's updater private key and Windows code-signing credentials remain protected environment secrets; only the updater public key may enter generated bundle configuration, and neither private material nor generated release config is tracked or logged.
- [x] A manually dispatched, protected `public-release` workflow checks out an exact `master` commit, proves package/Tauri/Cargo/UI/tag versions agree, runs the full quality/security suite, requires every trust input, signs Windows binaries/installers, creates v2 updater artifacts, verifies updater signatures plus Authenticode, and uploads a draft GitHub Release.
- [x] Release publication is never triggered by an ordinary push/tag/PR, uses least-privilege GitHub permissions and concurrency, does not silently overwrite a published release, and leaves final publication as an owner action.
- [x] Rust owns update checking, pending-update state, download/install, progress, errors, and restart. React receives metadata and an opaque pending token only; it cannot supply endpoints, artifact URLs, signatures, installer paths, or relaunch commands.
- [x] Update operations are serialized, pending tokens are one-time/expiring, a changed or missing pending update is rejected, cancellation never installs, signature/download/install failures are actionable, and no update installs without a separate explicit user confirmation.
- [x] Settings clearly shows configured/unconfigured state, current and available versions, Stable channel, release notes/date, download progress, restart consequence, and the 0.4.0-to-0.5.0 manual bootstrap limitation; keyboard/focus/live-region behavior works at supported window sizes and themes.
- [x] 0.5.0 identity is consistent in package, Tauri, Cargo/lock, Settings, README, native and release documentation; it remains an Alpha release candidate until owner-controlled trust inputs are provisioned and a signed draft is inspected.
- [x] Ephemeral local test keys prove updater artifact/signature generation and rejection of a modified artifact. No test key, signed bundle, database, backup, generated config, credential, or unrelated line-ending-only change is committed.
- [ ] Frozen install, frontend typecheck/lint/tests/build/audit, Rust format/strict all-target/all-feature Clippy/tests/release build, Tauri MSI/NSIS/updater packaging, responsive light/dark UI smoke, packaged startup, diff/secret/capability/workflow review, and exact-head GitHub CI pass.

## Allowed paths

- `package.json`
- `pnpm-lock.yaml`
- `src-tauri/Cargo.toml`
- `src-tauri/Cargo.lock`
- `src-tauri/tauri.conf.json`
- `src-tauri/capabilities/default.json`
- `src-tauri/src/lib.rs`
- `src-tauri/src/commands.rs`
- `src-tauri/src/native.rs`
- `src-tauri/src/updater.rs`
- `src/lib/db/types.ts`
- `src/lib/db/tauri.ts`
- `src/lib/db/tauri.test.ts`
- `src/components/NativeSettings.tsx`
- `src/components/NativeSettings.test.tsx`
- `src/components/Sidebar.tsx`
- `src/routes/Settings.tsx`
- `.github/workflows/public-release.yml`
- `.gitignore`
- `scripts/prepare-release-config.ps1`
- `scripts/sign-windows-artifact.ps1`
- `scripts/verify-updater-signature.ps1`
- `scripts/verify-release-inputs.ps1`
- `README.md`
- `CHANGELOG.md`
- `docs/architecture.md`
- `docs/native-desktop.md`
- `docs/release-checklist.md`
- `docs/release-runbook.md`
- `docs/decisions/013-native-windows-lifecycle.md`
- `docs/decisions/023-trusted-release-delivery.md`
- `.ai/*`

## Non-goals

- Purchasing/provisioning a certificate, Azure account, domain, CDN, GitHub Environment approver, or production updater key; publishing a live GitHub Release; rotating a real trust root; or spending owner funds.
- Background/automatic installation, forced updates, hidden startup network checks, beta/nightly channels, downgrade support, delta updates, portable binaries, Microsoft Store submission, macOS/Linux distribution, telemetry, accounts, or cloud delivery.
- Updating an installed 0.4.0 automatically: it has no updater client and must be manually upgraded once to the 0.5.0 bootstrap installer.
- Changing workspace data, migrations, backup semantics, AI behavior, Windows installation scope, or unrelated product features.

## Risks and safeguards

- **Supply-chain compromise:** release runs only by manual dispatch in a protected environment, with exact ref/version checks, least privilege, pinned actions, draft output, signature verification, and owner publication.
- **Private-key exposure:** scripts accept secrets only from environment variables, never echo values, generate config outside tracked state, and fail if sensitive/generated paths are staged.
- **Endpoint/artifact substitution:** endpoint is a fixed HTTPS repository URL; Rust retains the signed update object and accepts only an opaque one-time token for installation.
- **Arbitrary installer execution:** only the updater plugin's signature-verified artifact can install; no frontend path/URL/signature reaches native installation.
- **Concurrent or stale approval:** a mutex-owned pending update plus expiring one-time token serializes check/install and rejects reuse.
- **Interrupted upgrade:** Tauri's passive Windows installer owns replacement; Aether preserves workspace data in the app-data directory, while release notes/runbook require backup and rollback installers before publication.
- **Trust loss/rotation:** updater private key loss blocks updates to installed clients. Runbook requires protected redundant retention; public-key rotation must ship in a normally signed intermediate release before old-key retirement.
- **False public claim:** UI/docs retain Alpha candidate language until a real code-signed draft passes owner inspection; unconfigured builds say updates are unavailable.

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
PowerShell release-input negative/positive tests
ephemeral Tauri updater key generation, updater artifact build, valid signature verification, and tamper rejection
pnpm tauri:build
responsive light/dark Settings smoke and keyboard review
packaged startup smoke
git diff --check
secret/generated-file/capability/workflow/security diff review
GitHub PR exact-head CI
```

## Blocking decisions

None for implementation and test completion. Production activation remains an explicit external owner gate: a real Tauri updater key, modern Windows code-signing identity, protected `public-release` GitHub Environment, and repository secrets/variables must be provisioned before a signed draft release can exist.

## Readiness review

- **Status:** Ready. Trust boundaries, version/bootstrap behavior, workflow authority, updater UX, security invariants, test strategy, non-goals, and external activation gate are explicit.
- **Architecture gate:** ADR-023 is accepted before production implementation.
- **Data gate:** no database or workspace-data change is required.
- **Security gate:** Rust owns update state and installation; release secrets stay outside Git; ordinary builds remain updater-disabled.
- **Worktree gate:** listed frontend modifications are pre-existing Windows line-ending/index noise with no content diff; only explicit task paths will be staged.

## Implementation evidence and self-review

- **Ordinary-build boundary:** base `tauri.conf.json` contains no updater plugin configuration, ordinary `pnpm tauri:build` emits MSI/NSIS only and zero `.sig` files, and packaged startup no longer initializes the updater plugin when configuration is absent.
- **Trust boundary:** ADR-023, generated ignored release configuration, a fixed HTTPS Stable feed, a manual protected draft-only workflow, step-scoped secrets, pinned actions, Authenticode verification, and cryptographic updater/tamper verification are implemented.
- **Native boundary:** Rust retains the signed update object behind a mutex and exposes only check/cancel/install commands with one-time expiring approval; React supplies no endpoint, URL, path, signature, or relaunch instruction.
- **Product evidence:** 0.5.0 renders in the repository release window; responsive dark/light browser smokes and 83 frontend tests cover unconfigured, available, confirmation, progress, cancellation, error, and accessibility states. Computer Use could read the native 0.5.0 accessibility tree but its WebView2 click/screenshot geometry failed, so route interaction is covered by component and browser tests.
- **Quality evidence:** frozen install, typecheck, lint, 83/83 frontend tests, production build, clean pnpm audit, Rust format, strict all-target/all-feature Clippy, 103/103 Rust tests, Cargo check, release build, MSI/NSIS packaging, release-input positive/negative cases, YAML parsing, secret/capability/diff review, and packaged startup pass.
- **Supply-chain evidence:** `cargo audit` reports no vulnerability failures and 18 allowed pre-existing warnings (16 unmaintained, one unsound transitive GTK crate, one yanked transitive crate); ordinary installer hashes are recorded in session notes.
- **Review result:** all local acceptance criteria pass. Exact-head GitHub CI remains pending on the published draft PR; production signing remains an explicit owner-controlled activation gate and is not represented as completed.
