# ADR-013 — Native Windows lifecycle

- **Status:** Accepted
- **Date:** 2026-08-10

## Decision

- Closing the main window hides Aether to the system tray; the tray exposes Open Aether and Quit Aether.
- `Ctrl+Shift+Space` is the fixed alpha global shortcut for showing and focusing Aether. Registration failure is non-fatal and visible in Settings.
- Native notifications are emitted only through a narrow Rust test command for now; product reminders will use the same trusted service later.
- Window size and position are restored by the maintained Tauri window-state plugin.
- Installer metadata and icons remain Tauri-owned. Ordinary builds keep updates disabled; owner-gated signed Stable delivery is governed by ADR-023 and cannot activate without real external trust inputs.

## Consequences

Aether behaves like a persistent desktop workspace without silently failing startup when a shortcut is occupied. A real tray Quit action remains available. Update delivery cannot be enabled accidentally without generated release configuration and external signing credentials.
