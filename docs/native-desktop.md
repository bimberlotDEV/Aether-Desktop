# Native Windows behavior

## Lifecycle

- Closing the main window hides Aether to the system tray.
- Left-clicking the tray icon or choosing **Open Aether** restores and focuses the window.
- **Quit Aether** in the tray exits the process and persists window state.
- `Ctrl+Shift+Space` restores Aether when Windows accepts the registration. A conflict is non-fatal and reported in Settings.
- Native notifications use the maintained Tauri notification plugin. Settings exposes a fixed test notification; product reminders must use narrow trusted Rust commands rather than arbitrary frontend payloads.

## Installer

Tauri owns Windows packaging and reads identity/version/icons from `src-tauri/tauri.conf.json` and `src-tauri/Cargo.toml`. Release verification must build and install the generated Windows bundles on a clean-user account, then verify launch, database creation, uninstall, and preservation/removal expectations.

Machine-readable package versions use `0.5.0`; Alpha remains the human-facing product maturity until an owner-inspected signed public candidate exists. Textual SemVer prerelease identifiers are intentionally avoided because Windows MSI permits only numeric prerelease/build fields. Every shipped candidate must increment this numeric version so Windows installers have an explicit upgrade boundary.

## Trusted update delivery

Ordinary development and CI builds deliberately keep `createUpdaterArtifacts: false`, contain no update key/endpoint, and make no hidden update request. The maintained updater plugin is available only through narrow Rust commands: a user manually checks Stable, reviews metadata, and separately approves download, signature verification, passive Windows installation, and restart. React never supplies an endpoint, URL, signature, installer path, or relaunch command.

The protected manual release workflow generates an ignored Tauri overlay with the fixed GitHub Stable feed, real updater public key, passive install mode, and fixed Windows signing wrapper. It requires external Tauri and Azure Artifact Signing credentials, verifies release identity and quality gates, creates signed updater bundles, and uploads only a draft release. Follow `docs/release-runbook.md`; never commit a private key, password, generated config, placeholder trust material, or unsigned update metadata.

Alpha 0.5.0 is the updater bootstrap. Version 0.4.0 has no updater client and therefore needs one manual signed 0.5.0 installer upgrade.
