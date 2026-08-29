# Aether

> A calm, local-first personal workspace for Windows.

Aether combines Spaces, Markdown Notes, Tasks, Pulse, Vault, explicit Memory, authorized local Sources, and opt-in AI assistance in a Tauri desktop application.

**Current release candidate:** Alpha 0.5.0. The core roadmap through Phase 10 and canonical product evolution Milestones A–H is implemented. Milestone I repository readiness adds privacy-safe beta diagnostics, feedback intake and an external test protocol; it is not a completed Public Beta until signed distribution and meaningful real tester evidence exist. The candidate contains an owner-gated signed-release pipeline and explicit Stable updater, but is not a public signed release until protected trust inputs are provisioned and the generated draft passes the release runbook.

## What is included

- Hierarchical Spaces with configurable modules
- Local-first first-run onboarding with editable Student, Developer, Professional, Personal, and Blank starting points
- Markdown Notes with autosave, search, pinning, archive, move, and duplication
- Global and Space Tasks with subtasks, filters, due attention, and Pulse integration
- Linked or managed Vault items with ownership-aware deletion
- Explicit local folder Sources with bounded, metadata-only indexing and revocable access
- `Ctrl+K` Universal Search across commands and permitted local workspace domains with deterministic ranking
- Deterministic per-Space continuity and a curated local Activity timeline for resuming real work
- Pulse 2.0 with explainable Today, Continue, New, Recent, and user-controlled Ask Aether entry points
- Safe Actions for previewed, explicitly approved Task/Note creation and bounded single-file operations inside authorized Sources
- Explicit global or Space Memory that the user may attach to AI
- DeepSeek and OpenAI chat with cancellable streaming, persisted conversations, explicit model choice, transparent Auto routing, visible context, and separately approved Task/Note proposals
- Windows tray lifecycle, `Ctrl+Shift+Space`, notifications, and restored window state
- Sanitized workspace database export from Settings
- User-reviewed, content-free beta diagnostics with no automatic submission

## Privacy and data

Workspace data is stored in SQLite at `%APPDATA%/com.aether.desktop/aether.db`. There is no account or telemetry. AI is opt-in: only a prompt and context items explicitly attached to a conversation are sent to the provider shown on each response. DeepSeek and OpenAI API keys are separately encrypted for the current Windows user with DPAPI. Auto selects only a configured provider, explains its route, and never silently retries through another provider.

Complete `.aether-backup` archives contain the sanitized workspace database plus every file managed by Aether Vault. API credentials are never exported, and linked files remain external because Aether does not own them. Settings verifies an archive before showing its contents; restore requires a separate confirmation, creates a complete local safety backup, and restarts into a rollback-safe workspace replacement.

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

This produces unsigned development x64 MSI and NSIS installers below `src-tauri/target/release/bundle/`; those builds make no updater request. Public candidates are created only by the protected manual workflow in `.github/workflows/public-release.yml`, which requires independent Windows and updater signing trust, emits a draft GitHub Release, and must pass [the release runbook](docs/release-runbook.md) before an owner publishes it. Existing 0.4.0 installations need the signed 0.5.0 installer once to bootstrap in-app Stable updates.

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
- AI Task/Note drafts are parsed from the persisted assistant message in Rust, inherit that conversation's Space, and enter the same Safe Actions preview; the model cannot create or approve anything directly.
- `docs/decisions/` contains architecture decisions.
- `.ai/` contains the canonical engineering state and task contracts.

See [docs/architecture.md](docs/architecture.md), [docs/database.md](docs/database.md), and [WORKFLOW.md](WORKFLOW.md).

External testing is governed by [the beta handbook](docs/beta-program.md), [test matrix](docs/beta-test-matrix.md), and [privacy-minimal evidence template](docs/beta-evidence-template.md).

## License

[MIT](LICENSE) - Copyright 2026 bimberlotDEV.
