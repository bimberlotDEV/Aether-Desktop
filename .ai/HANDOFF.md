# Codex Task Contract

| Field | Value |
| --- | --- |
| Schema version | 2 |
| Task ID | `PHASE9-001` |
| Status | `complete` |
| Owner | Codex |
| Last updated | 2026-08-10 |
| Related milestone | Phase 9 — Native desktop features |

## Objective

Complete Aether's Windows desktop lifecycle with tray behavior, a reliable global show shortcut, native notifications, restored window state, verified installer assets/metadata, and an honest update-ready boundary.

## Acceptance criteria

- [x] Closing the main window hides it; tray Open restores/focuses it and tray Quit exits.
- [x] `Ctrl+Shift+Space` shows Aether when registration succeeds; conflicts do not prevent startup.
- [x] Settings shows native readiness and can send a real OS test notification.
- [x] Window position and size persist across runs.
- [x] Installer icon, identity, metadata, and Windows bundle output are verified.
- [x] Update activation requires real signing trust and is documented without fake keys.
- [x] Rust and frontend quality gates pass and native behavior has closest-layer tests where practical.

## Risks and controls

| Risk | Control |
| --- | --- |
| Users cannot truly exit | Explicit tray Quit calls application exit. |
| Shortcut is occupied | Non-fatal registration with visible availability status. |
| Notifications become an arbitrary spam API | Expose only a fixed test notification command. |
| Fake updater key creates false security | Keep updater disabled until real public key and endpoint are configured. |

## Verification and self-review

- `pnpm check` — pass; 41/41 tests across 15 files.
- `pnpm build` — pass.
- Rust format and strict Clippy — pass.
- `cargo test` — pass; 59/59 tests.
- `pnpm tauri:build` — pass; x64 MSI and NSIS bundles generated.
- Release executable startup smoke test — pass.
- Bundle metadata — product/file version 0.3.0; MSI manufacturer `bimberlotDEV`.
- **Decision:** approved for publication.
- **Correction:** Windows MSI rejected textual `0.3.0-alpha`; machine-readable versions were normalized to 0.3.0 while Alpha remains the product maturity.
