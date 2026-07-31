# Project State

> Canonical snapshot of what is true in the repository now. This is not a plan or historical log.

| Field | Value |
| --- | --- |
| Schema version | 1 |
| Last updated | 2026-07-31 |
| Updated by | Codex |
| Repository | `bimberlotDEV/Aether-Desktop` |
| Branch | `agent/env-001-rust-verification` |
| Baseline commit | `312308a` (`ai-backend`) |
| Product maturity | Alpha |

## Responsibility of this file

- Record completed milestones and the current milestone.
- Record active blockers and verified quality signals.
- Point to important decisions without duplicating their full rationale.
- Stay concise enough to read at the start of every agent session.

## Current milestone

| ID | Name | Status | Exit condition |
| --- | --- | --- | --- |
| `M-AI-WORKFLOW` | Hermes/Codex collaboration foundation | `complete` | Hermes accepted `AIWF-001` and promoted `DOC-001`. |
| `M-DOC-VERSIONING` | Version and metadata reconciliation | `complete` | Hermes accepted `DOC-001`. |
| `M-AUTO-PUBLISH` | Automatic GitHub task publication | `complete` | Draft PR #1 merged into `master`. |
| `M-RUST-VERIFY` | Restore local Rust verification | `complete` | Rust MSVC builds the test binaries and runs formatting, lint, and tests locally. |

No product feature is active while the collaboration foundation is fresh.

## Completed product milestones

| Milestone | Status | Evidence summary |
| --- | --- | --- |
| Phase 0 — Foundation | `complete` | Repository structure, product specification, tooling, and Tauri foundation exist. |
| Phase 1 — Shell and design system | `complete` | App shell, routes, sidebar, command palette, tokens, and themes exist. |
| Phase 2 — Local database | `complete` | SQLite, migrations, repositories, commands, TS wrappers, and tests exist. |
| Phase 3 — Spaces | `substantially_complete` | Space CRUD, hierarchy, templates, modules, archive, favourite, and reorder flows exist. |
| Phase 4 — Notes | `substantially_complete` | Notes persistence, editor, autosave, search, pin, archive, move, and duplicate flows exist. |
| Phase 7 — AI integration | `prototype` | DeepSeek provider, conversation persistence, context commands, and credential prototype exist; UI and production hardening do not. |

## Not started or placeholder milestones

| Milestone | Status | Notes |
| --- | --- | --- |
| Phase 5 — Tasks | `not_started` | No task domain implementation located. |
| Phase 6 — Vault | `placeholder` | Empty-state UI only. |
| Phase 8 — Memory | `not_started` | No explicit memory domain located. |
| Phase 9 — Native desktop | `partial` | Icons, Tauri bundle config, and a built executable exist; remaining native features are not verified. |
| Phase 10 — Release quality | `not_started` | No complete release audit or release pipeline verified. |

## Quality snapshot

| Check | Last result | Date | Notes |
| --- | --- | --- | --- |
| `pnpm check` | Pass | 2026-07-31 | typecheck clean, lint 0 warnings 0 errors (DEBT-002 resolved), test 7/7. |
| `cargo test --no-run` | Pass | 2026-07-31 | Both Windows test binaries build with Rust 1.97.1 MSVC. |
| `cargo test` | Partial: 30/33 verified pass | 2026-07-31 | 29 non-credential tests and `test_missing_key` pass; `test_store_and_get` deadlocks after 30 seconds, and the two remaining credential mutation tests share that path (`TECH-001`). |
| `cargo fmt --check` | Fail | 2026-07-31 | Pre-existing formatting differences tracked as `DEBT-005`. |
| `cargo clippy --all-targets -- -D warnings` | Fail | 2026-07-31 | Tooling works; existing 11 library/12 test-build findings tracked as `DEBT-006`. |

## Active blockers

| ID | Scope | Blocker | Owner | Resolution path |
| --- | --- | --- | --- | --- |
None. `BLOCK-001` was resolved by installing Rust 1.97.1 with the stable MSVC toolchain and Visual Studio 2022 C++ build tools.

## Active risks

| ID | Severity | Summary | Tracking task |
| --- | --- | --- | --- |
| `RISK-001` | High | AI credential mutation operations acquire the same non-reentrant database mutex recursively; `test_store_and_get` reproducibly exceeded 30 seconds. | `TECH-001` |
| `RISK-002` | High | The credential encryption key is derived from the database path and is not suitable as production secret storage. | `TECH-001` |
| `RISK-003` | ~~High~~ Resolved | README, package metadata, Tauri metadata, and agent phase guidance disagree on version/maturity. | `DOC-001` — reconciled all metadata to 0.3.0-alpha. |

## Decision index

Full rationale belongs in `.ai/ARCHITECTURE.md` or a dedicated ADR under `docs/decisions/`.

| ID | Decision | Status |
| --- | --- | --- |
| `ADR-001` | Tauri 2 is the Windows desktop shell. | Accepted |
| `ADR-002` | SQLite via bundled `rusqlite` is the local persistence layer. | Accepted |
| `ADR-003` | Frontend database access goes through typed Tauri invoke wrappers. | Accepted |
| `ADR-004` | Hybrid routing uses `direct_codex` for small low-risk work and `hermes_codex` for complex or risky work. | Accepted |
| `ADR-005` | `.ai/` documents are the collaboration source of truth. | Accepted |
| `ADR-007` | Verified task commits are automatically pushed and published through guarded repository tooling. | Accepted |

## Next: Hermes reviews the backlog and promotes the next task from `.ai/TODO.md`.

## Reusable update template

```markdown
## Current milestone
| ID | Name | Status | Exit condition |
| --- | --- | --- | --- |
| `M-...` | <name> | `planned|active|blocked|ready_for_review|complete` | <observable condition> |

## Active blockers
| ID | Scope | Blocker | Owner | Resolution path |
| --- | --- | --- | --- | --- |

## Quality snapshot
| Check | Last result | Date | Notes |
| --- | --- | --- | --- |
```
