# Changelog

All notable product changes are recorded here.

## Unreleased — Public beta readiness

- Added a Rust-owned sanitized diagnostic report that exposes only version, schema, integrity, generic platform and native-capability state; users inspect it before copying and Aether never uploads it.
- Added privacy-safe beta issue forms, an external Windows test matrix, severity/rollback guidance, known limitations and a manual evidence template for activation and retention signals.
- Reconciled security and release documentation with the current 0.5.0 backup, AI, signing and updater architecture.
- Public Beta remains an external milestone gate: signed publication and meaningful real tester evidence are not claimed by repository changes.

## 0.5.0-alpha - 2026-08-28

- Added complete verified workspace backup and rollback-safe restore, including Aether-managed Vault bytes while excluding credentials and externally linked files.
- Added owner-gated Windows release automation with separate Authenticode and updater trust, generated untracked configuration, exact-version quality gates, and draft-first GitHub publication.
- Added an explicit Stable update experience: Aether checks only when asked, shows release details, requires separate approval, verifies the signed artifact in Rust, and hands the passive installer to Windows.
- Kept ordinary development/CI builds updater-disabled and network-silent. Existing 0.4.0 installations require one manual signed 0.5.0 bootstrap upgrade.

## 0.4.0-alpha - 2026-08-28

- Added explicit local Sources, private Universal Search, deterministic Continuity/Activity, and Pulse 2.0.
- Added typed user-approved Safe Actions with containment, rollback, and audit trails.
- Added transparent DeepSeek/OpenAI routing, provider provenance, and approved Task/Note AI drafts.

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

The 0.3.x releases were unsigned alpha builds without automatic updates or restore.
