# Aether

> A calm, local-first personal workspace for Windows.

Aether combines Spaces, Markdown Notes, Tasks, Pulse, Vault, explicit Memory, authorized local Sources, and opt-in AI assistance in a Tauri desktop application.

**Current release:** Alpha 0.3.2. The core roadmap through Phase 10 is implemented and release-gated. Installers are currently unsigned and intended for alpha testing.

## What is included

- Hierarchical Spaces with configurable modules
- Markdown Notes with autosave, search, pinning, archive, move, and duplication
- Global and Space Tasks with subtasks, filters, due attention, and Pulse integration
- Linked or managed Vault items with ownership-aware deletion
- Explicit local folder Sources with bounded, metadata-only indexing and revocable access
- `Ctrl+K` Universal Search across commands and permitted local workspace domains with deterministic ranking
- Deterministic per-Space continuity and a curated local Activity timeline for resuming real work
- Pulse 2.0 with explainable Today, Continue, New, Recent, and user-controlled Ask Aether entry points
- Safe Actions for previewed, explicitly approved Task/Note creation and bounded single-file operations inside authorized Sources
- Explicit global or Space Memory that the user may attach to AI
- DeepSeek chat with cancellable streaming, persisted conversations, response modes, visible context, and confirmed Task proposals
- Windows tray lifecycle, `Ctrl+Shift+Space`, notifications, and restored window state
- Sanitized workspace database export from Settings

## Privacy and data

Workspace data is stored in SQLite at `%APPDATA%/Aether/aether.db`. There is no account or telemetry. AI is opt-in: only a prompt and context items explicitly attached to a conversation are sent to DeepSeek. The API key is encrypted for the current Windows user with DPAPI.

Database exports contain workspace records and Vault metadata, but omit credentials and all managed or linked Vault file contents. Automated restore is not part of this alpha.

See [SECURITY.md](SECURITY.md) for the security model and reporting guidance.

## Technology

- Tauri 2 and Rust (Windows WebView2 shell)
- React 19, TypeScript, Vite, and Tailwind CSS v4
- Zustand and React Router
- Bundled SQLite through `rusqlite`
- pnpm

## Requirements

- Windows 10 or 11, x64
- Node.js 22 and pnpm 11.18 for development
- Stable Rust MSVC toolchain with Visual Studio C++ build tools

## Development

```powershell
pnpm install --frozen-lockfile
pnpm dev
pnpm tauri:dev
```

Run the complete local gates:

```powershell
pnpm check
pnpm build
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
```

## Windows packaging

```powershell
pnpm tauri:build
```

This produces x64 MSI and NSIS installers below `src-tauri/target/release/bundle/`. Follow [docs/release-checklist.md](docs/release-checklist.md) before publishing a candidate. Code signing and automatic updates remain disabled until owner-controlled keys and a trusted endpoint are configured.

## Architecture

```text
React UI -> hooks/stores -> typed invoke wrappers -> Tauri commands
         -> Rust repositories/services -> SQLite/native OS/provider
```

- `src/` contains routes, components, hooks, stores, schemas, and invoke wrappers.
- `src-tauri/` contains the trusted Rust boundary, migrations, repositories, native services, and packaging.
- Universal Search keeps SQL and ranking in Rust, uses existing Notes FTS5, and returns only bounded typed results with local provenance.
- Continuity is composed locally in Rust from Space-scoped structured records; it never invokes AI or fabricates a summary.
- Safe Actions store reviewed requests in Rust behind short-lived one-time tokens; execution cannot accept replacement arguments, shell commands, deletion, or overwrite.
- `docs/decisions/` contains architecture decisions.
- `.ai/` contains the canonical engineering state and task contracts.

See [docs/architecture.md](docs/architecture.md), [docs/database.md](docs/database.md), and [WORKFLOW.md](WORKFLOW.md).

## License

[MIT](LICENSE) - Copyright 2026 bimberlotDEV.
