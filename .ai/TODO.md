# Prioritized Backlog

> Canonical queue of work that is not the active implementation contract.

| Field | Value |
| --- | --- |
| Schema version | 1 |
| Last updated | 2026-08-10 |
| Prioritized by | Codex within the owner-approved roadmap |

## Responsibility of this file

- Hold prioritized candidate work and technical debt.
- Make dependencies and readiness visible.
- Authorize small `direct_codex` work directly; require a `ready` `.ai/HANDOFF.md` contract for `planned_codex` work.
- Preserve product backlog items without expanding them into implementation plans.

## Status values

- `candidate`: captured but not refined.
- `needs_design`: requires Codex analysis, an ADR, or a human product decision.
- `planned`: sufficiently understood and queued for a task contract.
- `active`: represented by the current Codex task contract.
- `blocked`: cannot progress yet.
- `done`: completed and recorded in `.ai/CHANGELOG.md`.
- `deferred`: intentionally postponed.

## Priority queue

| Priority | ID | Work item | Type | Status | Dependencies | Acceptance summary |
| --- | --- | --- | --- | --- | --- | --- |
| P0 | `AIWF-001` | Establish Hermes/Codex collaboration system | Process | `done` | None | Hermes accepted the complete workflow. |
| P0 | `PROC-003` | Adopt Codex-only engineering workflow | Process | `done` | Project-owner decision | Codex owns planning through publication with planned-task contracts and self-review. |
| P0 | `TECH-001` | Audit and harden AI credential storage and locking | Security | `done` | `ENV-001` done | Merged via PR #3; Windows DPAPI enabled and 33/33 Rust tests pass. |
| P1 | `ENV-001` | Restore Rust verification capability | Tooling | `done` | Rust MSVC installation | Rust tests, formatting checks, and Clippy run locally; code findings are tracked separately. |
| P1 | `DOC-001` | Reconcile version and milestone documentation | Documentation | `done` | `AIWF-001` accepted | README, app metadata, and project state agree. |
| P1 | `PHASE34-CLOSEOUT` | Close Spaces and Notes MVP gaps | Product and testing | `done` | Rust quality gates restored | Space editing/reordering/state refresh and safe Note autosave/archive flows pass regression tests. |
| P1 | `PHASE5-EPIC` | Design and implement Tasks | Product | `done` | Codex decomposition | ADR-009, persistence, Space/global UI, and Pulse attention are complete. |
| P1 | `PHASE5-001` | Build Task persistence and IPC foundation | Product and architecture | `done` | ADR-009 | Migration, repository, commands, types, wrappers, and tests pass. |
| P1 | `PHASE5-002` | Build Tasks UI and Pulse integration | Product | `done` | `PHASE5-001` | Space Tasks and calm global due views meet Phase 5 acceptance. |
| P2 | `AI-UI-EPIC` | Complete the AI user experience | Product | `needs_design` | `TECH-001` | Provider setup, conversations, streaming, cancellation, and explicit context have reviewed UX. |
| P2 | `VAULT-EPIC` | Design and implement Vault | Product | `needs_design` | Security and file lifecycle design | Phase 6 requirements have an approved architecture. |

## Technical debt

| Priority | ID | Area | Description | Evidence |
| --- | --- | --- | --- | --- |
| P1 | `DEBT-001` | Testing | Frontend coverage now includes Spaces invalidation, Note autosave races/teardown, and Tauri argument boundaries. | Resolved by `PHASE34-CLOSEOUT`: 14/14 tests across five files. |
| P2 | `DEBT-002` | Lint | Four non-failing warnings were in utilities, IconPicker, and SpaceDetail. | ~~`pnpm lint` on 2026-07-30~~ Resolved: 0 warnings after fix (commit `28b82ab`). |
| P2 | `DEBT-003` | Architecture docs | Existing `docs/architecture.md` contains a duplicated Tauri Bridge layer and predates Notes/AI work. | Repository inspection |
| P2 | `DEBT-004` | Packaging | `Cargo.toml` repository field was empty despite configured Git remote. | ~~Repository inspection~~ Resolved: populated with https://github.com/bimberlotDEV/Aether-Desktop. |
| P1 | `DEBT-005` | Rust formatting | Rust sources now satisfy the repository formatting gate. | Resolved: `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` passes on 2026-08-10. |
| P1 | `DEBT-006` | Rust lint | Strict Clippy is clean across library, binary, and test targets. | Resolved: `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` passes on 2026-08-10. |

## Feature status index

| Feature | Status | Next planning action |
| --- | --- | --- |
| Spaces | `mvp_complete` | Extend only when a later roadmap phase requires it. |
| Notes | `mvp_complete` | Extend only when a later roadmap phase requires it. |
| Tasks | `mvp_complete` | Extend only when a later roadmap phase requires it. |
| Vault | `placeholder` | Codex designs file ownership and deletion semantics in a planned task. |
| AI | `prototype` | Resolve security design before UI expansion. |
| Memory | `not_started` | Defer until AI context semantics are stable. |

## New item template

```markdown
| P0-P3 | `AREA-NNN` | <imperative work item> | Product/Defect/Security/Tooling/Docs/Process | `candidate` | <IDs or None> | <observable outcome> |
```

Codex assigns the ID, priority, dependency chain, and readiness status within the owner-approved roadmap. Material product reprioritization or scope expansion requires the project owner's direction.
