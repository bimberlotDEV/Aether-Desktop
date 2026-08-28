# Native Windows behavior

## Lifecycle

- Closing the main window hides Aether to the system tray.
- Left-clicking the tray icon or choosing **Open Aether** restores and focuses the window.
- **Quit Aether** in the tray exits the process and persists window state.
- `Ctrl+Shift+Space` restores Aether when Windows accepts the registration. A conflict is non-fatal and reported in Settings.
- Native notifications use the maintained Tauri notification plugin. Settings exposes a fixed test notification; product reminders must use narrow trusted Rust commands rather than arbitrary frontend payloads.

## Installer

Tauri owns Windows packaging and reads identity/version/icons from `src-tauri/tauri.conf.json` and `src-tauri/Cargo.toml`. Release verification must build and install the generated Windows bundles on a clean-user account, then verify launch, database creation, uninstall, and preservation/removal expectations.

Machine-readable package versions use `0.4.0`; Alpha remains the human-facing product maturity. Textual SemVer prerelease identifiers are intentionally avoided because Windows MSI permits only numeric prerelease/build fields. Every shipped candidate must increment this numeric version so Windows installers have an explicit upgrade boundary.

## Update activation gate

Updater artifacts are deliberately disabled. Enabling updates requires all of the following in one reviewed release task:

1. Generate and securely retain a Tauri signing private key outside the repository.
2. Add only its real public key to the updater configuration.
3. Configure a stable HTTPS endpoint or signed GitHub Release `latest.json` feed.
4. Set `bundle.createUpdaterArtifacts` to `true` and install the maintained updater/process plugins.
5. Add CI secrets for signing and verify signature rejection, upgrade, rollback communication, and Windows install mode.

Never commit the private key, password, placeholder trust material, or unsigned update metadata.
