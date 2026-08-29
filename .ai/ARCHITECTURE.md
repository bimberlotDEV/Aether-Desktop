# Architecture and Engineering Constraints

> Concise agent-facing map of the current system and its binding patterns. Detailed product intent remains in `IDEA.md`; durable decisions may also live in `docs/decisions/`.

| Field                   | Value             |
| ----------------------- | ----------------- |
| Schema version          | 1                 |
| Last updated            | 2026-08-28        |
| Architecture owner      | Codex             |
| Implementation verifier | Codex self-review |

## Responsibility of this file

- Give both agents the same high-level system model before work begins.
- Record binding boundaries, patterns, and conventions.
- Index accepted and proposed architecture decisions.
- Avoid task plans, transient session details, and completion history.

## System overview

Aether is a local-first Windows desktop workspace. React renders the interface inside Tauri's WebView. TypeScript wrappers invoke registered Rust commands. Rust repositories own SQLite access, and versioned migrations evolve the local database.

```text
React routes/components
        |
Hooks and Zustand stores
        |
Typed TS schemas + invoke wrappers
        |
Tauri command boundary
        |
Rust repositories and domain services
        |
SQLite / native capabilities / external providers
```

## Repository map

| Path                             | Responsibility                                        | Change authority                                                     |
| -------------------------------- | ----------------------------------------------------- | -------------------------------------------------------------------- |
| `src/routes/`                    | Routed product surfaces and page composition          | Codex within task scope                                              |
| `src/components/`                | Reusable UI components                                | Codex within task scope                                              |
| `src/hooks/`                     | UI-facing orchestration and async state               | Codex within task scope                                              |
| `src/stores/`                    | Cross-component client state                          | Codex within task scope                                              |
| `src/lib/db/`                    | TypeScript schemas and Tauri invoke wrappers          | Codex within task scope                                              |
| `src/styles/index.css`           | Design tokens and shared styling foundation           | Codex; token changes require a planned task                          |
| `src-tauri/src/commands.rs`      | IPC boundary and command orchestration                | Codex within task scope                                              |
| `src-tauri/src/db/repositories/` | Persistent domain operations                          | Codex within task scope                                              |
| `src-tauri/src/db/migrations.rs` | Append-only schema evolution                          | Codex; migration design requires a planned task and ADR when durable |
| `src-tauri/src/ai/`              | Provider abstraction and credential security          | Codex; security work requires `planned_codex`                        |
| `src-tauri/src/backup.rs`        | Sanitized, consistent SQLite workspace export         | Codex; restore semantics require a separate planned task             |
| `docs/`                          | Human-facing architecture and decisions               | Codex owns verified technical intent and implementation facts        |
| `.ai/`                           | Codex planning, execution, and live engineering state | Codex, governed by `WORKFLOW.md`                                     |

## Binding architecture rules

1. UI code never executes SQL directly.
2. Persistent domain access follows Rust repository -> Tauri command -> typed TS wrapper -> hook/store -> UI.
3. New database changes use new append-only migration entries; applied migrations are never rewritten.
4. Domain logic stays outside React components.
5. TypeScript remains strict; `any` requires an explicit rationale.
6. External AI context is explicit and scoped; no implicit bulk access to user data.
7. Secrets must never be committed, logged, placed in handoff files, or stored using reversible project-derived key material.
8. Design tokens are preferred over scattered raw CSS values.
9. Architecture changes require a `planned_codex` task contract and accepted ADR before implementation.
10. AI retries reference an existing persisted user turn; generated Task proposals require strict validation and explicit transactional confirmation.
11. Workspace archives use SQLite backup semantics, omit credentials, verify every payload, include only Aether-managed Vault bytes, and restore only through preview, one-time approval, staging, restart, and rollback-safe replacement.
12. Public Windows delivery uses separate Authenticode and updater-signature trust roots. Release configuration is generated only in a protected manual workflow; Rust owns fixed-endpoint update checks, pending signed artifacts, explicit installation approval, and restart.
13. First-run state is initialized in Rust. Empty workspaces receive onboarding, meaningful upgraded workspaces bypass it, and optional Source/AI setup reuses existing native authorization and credential boundaries.

## UI and design constraints

- Calm, restrained, premium, neutral palette with an indigo accent.
- Support dark, light, and system themes.
- Avoid gradients, neon glows, glassmorphism, generic dashboard composition, fake content, and dead controls.
- Preserve visible keyboard focus and accessible interaction states.
- Curate icons; do not treat a generic icon library as the visual system.

## Cross-cutting quality requirements

| Concern     | Required behavior                                                            |
| ----------- | ---------------------------------------------------------------------------- |
| Data safety | Destructive operations are explicit, scoped, and recoverable when practical. |
| Concurrency | Database and async flows must avoid re-entrant locks and UI race conditions. |
| Validation  | Data is validated at external and IPC boundaries.                            |
| Errors      | Failures are surfaced with actionable UI or typed errors; no silent loss.    |
| Testing     | Domain behavior is covered closest to the owning layer.                      |
| Performance | Avoid blocking the UI thread and avoid unnecessary database round trips.     |
| Privacy     | Local-first defaults, no telemetry, and opt-in external AI access.           |

## Decision registry

| ID        | Decision                                                                                                                                                                                        | Status                  | Rationale source                                     |
| --------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------- | ---------------------------------------------------- |
| `ADR-001` | Use Tauri 2 rather than Electron.                                                                                                                                                               | Accepted                | `docs/architecture.md`                               |
| `ADR-002` | Use bundled SQLite through `rusqlite`.                                                                                                                                                          | Accepted                | `docs/decisions/001-sqlite-rusqlite.md`              |
| `ADR-003` | Use React 19, TypeScript, Vite, Tailwind, and Zustand.                                                                                                                                          | Accepted                | `AGENTS.md`, `docs/architecture.md`                  |
| `ADR-004` | Use hybrid Hermes/Codex task routing.                                                                                                                                                           | Superseded by `ADR-008` | `WORKFLOW.md` history                                |
| `ADR-005` | Use `.ai/` as the canonical agent state layer.                                                                                                                                                  | Accepted                | `WORKFLOW.md`                                        |
| `ADR-006` | Use Windows DPAPI (`CryptProtectData`/`CryptUnprotectData`) bound to the current user for production credential encryption; provide a parallel test-only ring-based crypto for in-memory tests. | Accepted                | `docs/decisions/006-dpapi-credential-storage.md`     |
| `ADR-007` | Publish verified task commits automatically through a versioned Git hook and guarded publication script.                                                                                        | Accepted                | `WORKFLOW.md`                                        |
| `ADR-008` | Use Codex as the sole engineering agent with planned-task contracts and evidence-based self-review.                                                                                             | Accepted                | `docs/decisions/008-codex-only-workflow.md`          |
| `ADR-009` | Use a self-referencing Task table, local calendar due dates, JSON tags, and full-state IPC updates.                                                                                             | Accepted                | `docs/decisions/009-task-domain-model.md`            |
| `ADR-010` | Enforce Vault linked/managed ownership and safe deletion in trusted Rust code.                                                                                                                  | Accepted                | `docs/decisions/010-vault-file-ownership.md`         |
| `ADR-011` | Use cancellable typed AI streams and explicit Space-isolated context resolved in Rust.                                                                                                          | Accepted                | `docs/decisions/011-ai-streaming-and-context.md`     |
| `ADR-012` | Use explicit, user-authored, scoped Memory with opt-in AI attachment.                                                                                                                           | Accepted                | `docs/decisions/012-explicit-memory.md`              |
| `ADR-013` | Use a persistent native Windows tray lifecycle with visible shortcut/notification readiness and gated signed updates.                                                                           | Accepted                | `docs/decisions/013-native-windows-lifecycle.md`     |
| `ADR-014` | Export a sanitized SQLite workspace snapshot; defer restore and Vault-byte archives.                                                                                                            | Accepted                | `docs/decisions/014-workspace-backup.md`             |
| `ADR-015` | Build Aether's frontend from an internal semantic interface system with a distinctive shell and reusable primitives.                                                                            | Accepted                | `docs/decisions/015-aether-interface-system.md`      |
| `ADR-016` | Require explicit Sources and bounded, metadata-only, non-mutating local directory snapshots before file intelligence.                                                                           | Accepted                | `docs/decisions/016-context-sources-and-indexing.md` |
| `ADR-017` | Use one local typed search repository with Notes FTS5, bounded metadata matching, and deterministic explainable ranking.                                                                        | Accepted                | `docs/decisions/017-universal-search-ranking.md`     |
| `ADR-018` | Compose Space continuity from bounded local read models and curate meaningful Activity in backend domain commands.                                                                              | Accepted                | `docs/decisions/018-deterministic-continuity.md`     |
| `ADR-019` | Compose Pulse from bounded local facts with deterministic, explainable daily relevance.                                                                                                         | Accepted                | `docs/decisions/019-deterministic-pulse.md`          |
| `ADR-020` | Execute only typed, Rust-validated Actions through preview, explicit approval, one-time tokens, and curated audit records.                                                                      | Accepted                | `docs/decisions/020-safe-actions.md`                 |
| `ADR-021` | Route AI through a closed Rust provider registry with transparent deterministic Auto decisions; AI drafts reuse Safe Actions and never approve or execute.                                     | Accepted                | `docs/decisions/021-ai-provider-routing.md`          |
| `ADR-022` | Package sanitized SQLite plus managed Vault bytes in a verified portable archive and restore through a one-time, restart-bound, rollback-safe swap.                                         | Accepted                | `docs/decisions/022-portable-backup-and-safe-restore.md` |
| `ADR-023` | Deliver Windows releases through owner-gated Authenticode and Tauri signatures, draft-first publication, and a narrow Rust-owned Stable updater.                                             | Accepted                | `docs/decisions/023-trusted-release-delivery.md`     |
| `ADR-024` | Gate the shell with upgrade-safe local onboarding initialized in Rust and composed from existing Space, Source, and DPAPI provider boundaries.                                                  | Accepted                | `docs/decisions/024-first-run-onboarding.md`         |
| `ADR-025` | Prepare external testing with allowlisted local diagnostics, manual privacy-minimal evidence, and an explicit repository-readiness versus real-beta boundary.                                  | Accepted                | `docs/decisions/025-public-beta-operations.md`       |

## Architecture change protocol

1. Codex records the problem, constraints, options, and decision ID.
2. For durable or high-risk decisions, Codex creates an ADR under `docs/decisions/`.
3. Codex updates this registry and writes a bounded task contract in `.ai/HANDOFF.md`.
4. Codex implements the accepted decision, verifies it, and performs a separate self-review pass.
5. If implementation evidence invalidates the design, Codex updates or supersedes the ADR before continuing; human input is requested only when the approval gate applies.

## Decision template

```markdown
### `ADR-NNN` — <decision title>

- Status: Proposed | Accepted | Superseded
- Context: <problem and constraints>
- Options considered: <short list>
- Decision: <chosen direction>
- Consequences: <trade-offs and migration impact>
- Evidence: <paths, tests, measurements, or ADR link>
```
