# Architecture and Engineering Constraints

> Concise agent-facing map of the current system and its binding patterns. Detailed product intent remains in `IDEA.md`; durable decisions may also live in `docs/decisions/`.

| Field | Value |
| --- | --- |
| Schema version | 1 |
| Last updated | 2026-07-30 |
| Architecture owner | Hermes |
| Implementation verifier | Codex |

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

| Path | Responsibility | Change authority |
| --- | --- | --- |
| `src/routes/` | Routed product surfaces and page composition | Codex within handoff scope |
| `src/components/` | Reusable UI components | Codex within handoff scope |
| `src/hooks/` | UI-facing orchestration and async state | Codex within handoff scope |
| `src/stores/` | Cross-component client state | Codex within handoff scope |
| `src/lib/db/` | TypeScript schemas and Tauri invoke wrappers | Codex within handoff scope |
| `src/styles/index.css` | Design tokens and shared styling foundation | Codex; token changes require Hermes approval |
| `src-tauri/src/commands.rs` | IPC boundary and command orchestration | Codex within handoff scope |
| `src-tauri/src/db/repositories/` | Persistent domain operations | Codex within handoff scope |
| `src-tauri/src/db/migrations.rs` | Append-only schema evolution | Codex; migration design must be in handoff |
| `src-tauri/src/ai/` | Provider abstraction and credentials prototype | Codex; security redesign requires Hermes handoff |
| `docs/` | Human-facing architecture and decisions | Hermes owns intent; Codex updates implementation facts |
| `.ai/` | Agent coordination and live engineering state | Split ownership defined in `WORKFLOW.md` |

## Binding architecture rules

1. UI code never executes SQL directly.
2. Persistent domain access follows Rust repository -> Tauri command -> typed TS wrapper -> hook/store -> UI.
3. New database changes use new append-only migration entries; applied migrations are never rewritten.
4. Domain logic stays outside React components.
5. TypeScript remains strict; `any` requires an explicit rationale.
6. External AI context is explicit and scoped; no implicit bulk access to user data.
7. Secrets must never be committed, logged, placed in handoff files, or stored using reversible project-derived key material.
8. Design tokens are preferred over scattered raw CSS values.
9. Architecture changes require an explicit Hermes decision in the handoff before Codex implements them.

## UI and design constraints

- Calm, restrained, premium, neutral palette with an indigo accent.
- Support dark, light, and system themes.
- Avoid gradients, neon glows, glassmorphism, generic dashboard composition, fake content, and dead controls.
- Preserve visible keyboard focus and accessible interaction states.
- Curate icons; do not treat a generic icon library as the visual system.

## Cross-cutting quality requirements

| Concern | Required behavior |
| --- | --- |
| Data safety | Destructive operations are explicit, scoped, and recoverable when practical. |
| Concurrency | Database and async flows must avoid re-entrant locks and UI race conditions. |
| Validation | Data is validated at external and IPC boundaries. |
| Errors | Failures are surfaced with actionable UI or typed errors; no silent loss. |
| Testing | Domain behavior is covered closest to the owning layer. |
| Performance | Avoid blocking the UI thread and avoid unnecessary database round trips. |
| Privacy | Local-first defaults, no telemetry, and opt-in external AI access. |

## Decision registry

| ID | Decision | Status | Rationale source |
| --- | --- | --- | --- |
| `ADR-001` | Use Tauri 2 rather than Electron. | Accepted | `docs/architecture.md` |
| `ADR-002` | Use bundled SQLite through `rusqlite`. | Accepted | `docs/decisions/001-sqlite-rusqlite.md` |
| `ADR-003` | Use React 19, TypeScript, Vite, Tailwind, and Zustand. | Accepted | `AGENTS.md`, `docs/architecture.md` |
| `ADR-004` | Separate Hermes planning/review from Codex implementation/verification. | Accepted | `WORKFLOW.md` |
| `ADR-005` | Use `.ai/` as the canonical agent state layer. | Accepted | `WORKFLOW.md` |
| `ADR-006` | Production credential storage mechanism. | Proposed | Must be decided before `TECH-001` becomes ready. |

## Architecture change protocol

1. Hermes records the problem, constraints, options, and decision ID.
2. For material or irreversible decisions, Hermes creates an ADR under `docs/decisions/`.
3. Hermes updates this decision registry and creates a bounded handoff.
4. Codex implements the accepted decision without substituting a different design.
5. If implementation evidence invalidates the design, Codex stops, marks the handoff `blocked`, and records the evidence; Hermes decides the revision.

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
