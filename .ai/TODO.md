# Prioritized Backlog

> Canonical queue of work that is not the active implementation contract.

| Field | Value |
| --- | --- |
| Schema version | 1 |
| Last updated | 2026-07-31 |
| Prioritized by | Hermes (initial inventory recorded by Codex) |

## Responsibility of this file

- Hold prioritized candidate work and technical debt.
- Make dependencies and readiness visible.
- Never authorize Codex to start work; only a `ready` `.ai/HANDOFF.md` does that.
- Preserve product backlog items without expanding them into implementation plans.

## Status values

- `candidate`: captured but not refined.
- `needs_design`: requires Hermes architecture or product decisions.
- `ready_for_handoff`: sufficiently understood to be promoted.
- `active`: represented by the current handoff.
- `blocked`: cannot progress yet.
- `done`: completed and recorded in `.ai/CHANGELOG.md`.
- `deferred`: intentionally postponed.

## Priority queue

| Priority | ID | Work item | Type | Status | Dependencies | Acceptance summary |
| --- | --- | --- | --- | --- | --- | --- |
| P0 | `AIWF-001` | Establish Hermes/Codex collaboration system | Process | `done` | None | Hermes accepted the complete workflow. |
| P0 | `TECH-001` | Audit and harden AI credential storage and locking | Security | `ready_for_handoff` | `ENV-001` done | No deadlock; secrets use Windows DPAPI (`ADR-006` accepted). |
| P1 | `ENV-001` | Restore Rust verification capability | Tooling | `done` | Rust MSVC installation | Rust tests, formatting checks, and Clippy run locally; code findings are tracked separately. |
| P1 | `DOC-001` | Reconcile version and milestone documentation | Documentation | `done` | `AIWF-001` accepted | README, app metadata, and project state agree. |
| P1 | `PHASE5-EPIC` | Design and implement Tasks | Product | `needs_design` | Hermes decomposition | Phase 5 requirements are split into bounded vertical slices. |
| P2 | `AI-UI-EPIC` | Complete the AI user experience | Product | `needs_design` | `TECH-001` | Provider setup, conversations, streaming, cancellation, and explicit context have reviewed UX. |
| P2 | `VAULT-EPIC` | Design and implement Vault | Product | `needs_design` | Security and file lifecycle design | Phase 6 requirements have an approved architecture. |

## Technical debt

| Priority | ID | Area | Description | Evidence |
| --- | --- | --- | --- | --- |
| P1 | `DEBT-001` | Testing | Frontend coverage is only 7 tests; Spaces, Notes, and Tauri boundaries need risk-based coverage. | `pnpm test` on 2026-07-30 |
| P2 | `DEBT-002` | Lint | Four non-failing warnings were in utilities, IconPicker, and SpaceDetail. | ~~`pnpm lint` on 2026-07-30~~ Resolved: 0 warnings after fix (commit `28b82ab`). |
| P2 | `DEBT-003` | Architecture docs | Existing `docs/architecture.md` contains a duplicated Tauri Bridge layer and predates Notes/AI work. | Repository inspection |
| P2 | `DEBT-004` | Packaging | `Cargo.toml` repository field was empty despite configured Git remote. | ~~Repository inspection~~ Resolved: populated with https://github.com/bimberlotDEV/Aether-Desktop. |
| P1 | `DEBT-005` | Rust formatting | `cargo fmt --check` reports pre-existing formatting differences across AI, commands, and conversation repository code. | `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` on 2026-07-31 |
| P1 | `DEBT-006` | Rust lint | Strict Clippy reports 11 library errors and 12 test-build errors, including unused code and correctness/style lints. | `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` on 2026-07-31 |

## Feature status index

| Feature | Status | Next planning action |
| --- | --- | --- |
| Spaces | `substantially_complete` | Define remaining acceptance gaps and regression coverage. |
| Notes | `substantially_complete` | Verify autosave, search, archive, and failure behavior in Tauri. |
| Tasks | `not_started` | Hermes creates an architecture slice and first handoff. |
| Vault | `placeholder` | Hermes designs file ownership and deletion semantics. |
| AI | `prototype` | Resolve security design before UI expansion. |
| Memory | `not_started` | Defer until AI context semantics are stable. |

## New item template

```markdown
| P0-P3 | `AREA-NNN` | <imperative work item> | Product/Defect/Security/Tooling/Docs/Process | `candidate` | <IDs or None> | <observable outcome> |
```

Hermes must assign the ID, priority, dependency chain, and readiness status. Codex may add newly discovered defects or debt, but must not reprioritize existing items.
