# Aether architecture

## System boundaries

```text
React routes and components
  -> hooks and Zustand stores
  -> strict TypeScript types and Tauri invoke wrappers
  -> registered Rust commands
  -> repositories and domain/native services
  -> SQLite, Windows APIs, local files, or the configured AI provider
```

The webview is untrusted relative to native resources. It never executes SQL, receives credential values, or receives internal Vault storage paths. Narrow Tauri commands validate external input and delegate persistence to Rust repositories.

## Domains

| Domain             | Persistent owner                                        | UI entry points                           |
| ------------------ | ------------------------------------------------------- | ----------------------------------------- |
| Spaces and modules | SQLite repositories                                     | Pulse, Spaces, Space detail               |
| Notes              | SQLite repository and FTS                               | Space Notes                               |
| Tasks              | SQLite repository                                       | Tasks, Pulse, Space Tasks                 |
| Vault              | SQLite metadata plus Rust filesystem service            | Vault, Space Vault                        |
| Memory             | SQLite repository                                       | Memory, Space Memory, explicit AI context |
| AI                 | SQLite conversations/context plus Rust provider service | AI, Space AI, Settings                    |
| Native lifecycle   | Tauri plugins and `native.rs`                           | tray, shortcut, notifications, Settings   |
| Backup             | `backup.rs` and SQLite online backup API                | Settings                                  |
| Context Sources    | SQLite metadata plus bounded `context.rs` scanner       | Sources                                   |

## Data and concurrency

SQLite is bundled with `rusqlite`; versioned migrations are append-only. The live connection uses WAL, foreign keys, and normal synchronization. A single mutex protects the connection. Credential encryption deliberately operates outside re-entrant database locking. Note autosave is serialized and revision-aware; AI streams have explicit cancellation and terminal persistence.

Workspace export uses SQLite's online backup API while holding the connection boundary. It writes a sibling partial database, removes the `secrets` table, verifies integrity, and finalizes with rollback protection for an existing destination. See ADR-014.

Context Sources store an explicitly authorized canonical root and relative child metadata. Scans run outside the UI thread, do not follow symlinks or Windows reparse points, enforce depth/file limits, and never read contents or mutate files. Snapshot application is transactional; truncated scans cannot mark unseen files removed. See ADR-016.

## Security and privacy

- Tauri CSP restricts scripts to the application and capabilities expose only required native actions.
- DeepSeek credentials are protected with Windows DPAPI and never returned to the frontend.
- AI context is user-selected, bounded, and Space-isolated in Rust.
- Linked Vault files are never deleted; managed deletion is containment-checked and recoverable during the operation.
- Database exports exclude secrets and disclose that Vault file bytes are out of scope.
- Source roots are explicit and revocable; indexed child APIs expose relative metadata only and are not attached to AI.
- There is no telemetry, account backend, active updater, or embedded signing secret.

## Quality and delivery

Pull requests and `master` run Windows frontend typecheck/lint/tests/build plus Rust format, strict Clippy, and tests. Task branches are published with `scripts/publish-task.ps1`. MSI and NSIS packages are built locally for alpha candidates; signing and auto-update activation require owner-controlled infrastructure.

Durable rationale is indexed in `.ai/ARCHITECTURE.md` and stored under `docs/decisions/`.
