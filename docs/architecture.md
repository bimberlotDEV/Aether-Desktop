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
| AI                 | SQLite conversations/context plus closed Rust provider registry and router | AI, Space AI, Settings                    |
| Native lifecycle   | Tauri plugins and `native.rs`                           | tray, shortcut, notifications, Settings   |
| Backup             | `backup.rs` and SQLite online backup API                | Settings                                  |
| Context Sources    | SQLite metadata plus bounded `context.rs` scanner       | Sources                                   |
| Universal Search   | Rust search repository, Notes FTS5, metadata ranking    | Ctrl+K                                    |
| Continuity         | Space-scoped Rust read model plus curated Activity      | Space overview, Activity                  |
| Pulse relevance    | Bounded deterministic Rust read model                   | Pulse                                     |
| Safe Actions       | Typed Rust proposal runtime plus existing repositories  | Actions                                   |
| First-run profile  | SQLite profile repository and existing trusted domains  | Onboarding gate, Settings tour            |

## Data and concurrency

SQLite is bundled with `rusqlite`; versioned migrations are append-only. The live connection uses WAL, foreign keys, and normal synchronization. A single mutex protects the connection. Credential encryption deliberately operates outside re-entrant database locking. Note autosave is serialized and revision-aware; AI streams have explicit cancellation and terminal persistence.

Workspace export uses SQLite's online backup API while holding the connection boundary. It writes a sibling partial database, removes the `secrets` table, verifies integrity, and finalizes with rollback protection for an existing destination. See ADR-014.

Context Sources store an explicitly authorized canonical root and relative child metadata. Scans run outside the UI thread, do not follow symlinks or Windows reparse points, enforce depth/file limits, and never read contents or mutate files. Snapshot application is transactional; truncated scans cannot mark unseen files removed. See ADR-016.

Universal Search uses one typed Rust repository and command. Notes content is retrieved through the existing FTS5 index; active metadata domains use parameterized bounded queries. Rust applies deterministic text-quality, current-Space, favourite/pinned, open-Task, and recency signals before returning a globally capped result set. Commands remain frontend-owned. Source results expose only Source identity and relative indexed paths, and search never invokes AI. See ADR-017.

Continuity composes bounded active Notes, open Tasks, present Source-file metadata, the latest active conversation, and curated Activity for exactly one active Space. Suggested next steps use a fixed local priority rather than generated prose. Domain commands own a closed meaningful event vocabulary with quiet-window deduplication; the webview cannot submit arbitrary events or receive raw metadata JSON. Activity and Space detail are lazy route chunks. See ADR-018.

Pulse composes a single read-only snapshot from dated open Tasks, recently worked active Spaces, new present Source metadata, and curated Activity. Relevance and the suggested next step follow a fixed, factual priority; Pulse performs no AI call or mutation. See ADR-019.

Safe Actions uses a closed Rust request enum and keeps each validated proposal in process behind a short-lived opaque token. Approval supplies only that token, which is consumed before revalidation and execution. Database writes and their curated Activity audit are transactional; reversible file writes roll back on audit failure. Filesystem actions canonicalize one explicitly authorized Source, require indexed input files, reject traversal, symlink escape, missing parents, directories, and existing destinations, and never expose absolute roots over IPC. There is no shell, delete, arbitrary executable, cross-Source transfer, or model-owned approval. See ADR-020.

AI providers are a closed Rust registry with fixed official DeepSeek and OpenAI endpoints. Credentials are separately DPAPI-protected; Auto chooses a configured route deterministically before disclosure and never performs silent cross-provider fallback. Each assistant response stores its resolved provider, model, routing mode, and human-readable reason. AI Action JSON is re-read from the persisted assistant message, strictly parsed into Task/Note drafts, bound to the conversation Space and message origin, and previewed through Safe Actions one item at a time. See ADR-021.

First-run initialization is idempotent and Rust-owned. A missing profile beside any meaningful persisted Space, Note, Task, Vault item, Memory, conversation, or Source is treated as an upgrade and marked complete without changing domain rows. Only an empty workspace receives onboarding. The frontend then composes existing transactional Space creation, explicit Source authorization, DPAPI provider configuration, and the normal Pulse shell; interrupted setup resumes an existing top-level Space rather than duplicating it. See ADR-024.

## Security and privacy

- Tauri CSP restricts scripts to the application and capabilities expose only required native actions.
- DeepSeek and OpenAI credentials are separately protected with Windows DPAPI and never returned to the frontend.
- AI context is user-selected, bounded, and Space-isolated in Rust.
- Linked Vault files are never deleted; managed deletion is containment-checked and recoverable during the operation.
- Database exports exclude secrets and disclose that Vault file bytes are out of scope.
- Source roots are explicit and revocable; indexed child APIs expose relative metadata only and are not attached to AI.
- Continuity is deterministic and Space-bound; it exposes structured facts and presentation-safe Activity without sending data to DeepSeek.
- Pulse is local and explainable; archived scopes, removed Source files, raw metadata, and absolute Source roots are excluded.
- Safe Actions require a visible consequence review and explicit user approval; one-time execution remains inside the narrow validated Rust capability boundary.
- There is no telemetry, account backend, embedded signing secret, hidden update check, or frontend-controlled installer. Signed public builds may use the fixed Stable GitHub feed through the Rust-owned updater boundary.
- Beta diagnostics are composed in Rust from a closed metadata-only schema, displayed before copying, and never include domain data, counts, paths, logs, identifiers or secrets. Aether has no diagnostic submission endpoint.
- Onboarding collects no account, contact, demographic, analytics, or payment data; optional folders and AI providers retain their existing explicit native consent boundaries.

## Quality and delivery

Pull requests and `master` run Windows frontend typecheck/lint/tests/build plus Rust format, strict Clippy, and tests. Task branches are published with `scripts/publish-task.ps1`. Ordinary MSI/NSIS builds remain unsigned and updater-disabled. Public Windows candidates use a manually dispatched protected Environment, generated untracked configuration, independent Authenticode/updater trust roots, fixed Stable feed, signature verification, and draft-first publication under ADR-023.

Durable rationale is indexed in `.ai/ARCHITECTURE.md` and stored under `docs/decisions/`.
