# ADR-023 — Trusted Windows release delivery

- **Status:** Accepted
- **Date:** 2026-08-28
- **Decision owner:** Codex

## Context

Aether can build unsigned Windows installers, but a public desktop product needs two independent trust layers: Windows Authenticode identity for downloaded executables/installers and Tauri updater signatures for artifact authenticity. The 0.4.0 client has no updater and therefore cannot bootstrap itself. Private keys must never enter source control, frontend IPC, logs, ordinary CI, or developer configuration.

## Options considered

1. Publish unsigned installers and let users bypass SmartScreen.
2. Put updater permissions directly in React and commit a permanent updater configuration.
3. Use a protected, manually dispatched release pipeline; generate release-only configuration from owner inputs; keep update state and installation in Rust; publish drafts for owner inspection.
4. Use a custom unsigned updater or download-and-run implementation.

## Decision

Choose option 3.

- Stable updates use the fixed HTTPS GitHub Release feed `https://github.com/bimberlotDEV/Aether-Desktop/releases/latest/download/latest.json`.
- Ordinary builds retain `createUpdaterArtifacts: false`, contain no updater endpoint/public key, perform no hidden update checks, and report updates unavailable.
- A strict script generates an ignored release-only Tauri configuration after validating the real public key and fixed endpoint. Tauri updater artifacts are signed by the externally retained `TAURI_SIGNING_PRIVATE_KEY`.
- Windows executables/installers are signed through a modern owner-controlled signing service invoked by a fixed wrapper; configuration values are passed as separately validated arguments, not interpolated into shell commands.
- GitHub release automation is manual, targets an exact `master` commit/version, runs in a protected `public-release` Environment, uses least privilege, verifies signatures, and creates a draft. Publishing remains an owner action.
- The maintained Tauri updater plugin is initialized in Rust. React calls narrow commands to check and explicitly install a pending update. Rust owns endpoint configuration, the pending signed update object, expiring one-time approval, progress, and restart.
- Aether checks only when the user asks. It never silently downloads or installs. Windows uses passive install mode so progress is visible.
- 0.5.0 is the updater bootstrap. Existing 0.4.0 installations require one manual installer upgrade. Key rotation must be shipped in an update signed by the old key before that key is retired.

## Consequences

- Repository implementation and ephemeral signature tests can complete without production credentials, but public activation cannot.
- Losing the updater private key prevents future updates to clients trusting it; redundant protected retention is mandatory.
- A compromised release account alone cannot forge an accepted updater artifact without the updater private key, while Authenticode separately gives Windows publisher identity.
- GitHub Releases is a practical first Stable feed and availability dependency. Moving endpoints or adding channels requires a reviewed ADR and a signed transition release.
- Updater commands increase native dependency surface but avoid broad guest permissions and arbitrary installer execution.

## Evidence

- Task contract: `.ai/HANDOFF.md` (`RELEASE-TRUST-001`)
- Planned implementation: `src-tauri/src/updater.rs`, generated release config scripts, `.github/workflows/public-release.yml`, Settings update surface
- Official constraints: Tauri updater signatures are mandatory; update public/private keys have separate roles; production endpoints enforce TLS; Windows passive install mode is recommended; `tauri-action` creates GitHub Release updater metadata.
