# Changelog

All notable product changes are recorded here.

## 0.3.2-alpha - 2026-08-25

- Reconciled completed DeepSeek responses with persisted conversations so answers appear without requiring Ctrl+R, including when WebView stream lifecycle events are missed.
- Added safe transactional upgrades for personal-beta Tasks, Memory, and Vault schemas while retaining legacy managed Vault files as recovery copies.
- Reimagined the complete Aether interface with a cohesive application shell, richer Pulse, redesigned product surfaces, responsive layouts, accessible focus behavior, and refined dark/light themes.
- Consolidated the merged AI, migration, and interface work into a separately versioned Windows upgrade candidate.

## 0.3.1-alpha - 2026-08-11

- Fixed submitted AI prompts appearing only after a refresh.
- Fixed a WebView2 scroll effect crash during streamed message updates.
- Incremented the Windows package version so the hotfix installs as an explicit upgrade from 0.3.0.

## 0.3.0-alpha - 2026-08-10

- Added local-first Spaces, Markdown Notes, Tasks, Pulse, Vault metadata/file workflows, and explicit Memory.
- Added opt-in DeepSeek conversations with DPAPI-protected credentials, cancellable streaming, explicit scoped context, modes, and confirmed Task proposals.
- Added Windows tray behavior, `Ctrl+Shift+Space`, native notifications, and persisted window state.
- Added sanitized workspace database export, excluding credentials and Vault file contents.
- Added Windows CI, strict frontend/Rust gates, MSI and NSIS packaging, release documentation, and dependency monitoring.

This is an unsigned alpha. Automatic updates and automatic backup restore are intentionally unavailable.
