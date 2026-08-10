# Project State

> Canonical snapshot of what is true in the repository now. This is not a plan or historical log.

| Field | Value |
| --- | --- |
| Schema version | 1 |
| Last updated | 2026-08-10 |
| Updated by | Codex |
| Repository | `bimberlotDEV/Aether-Desktop` |
| Branch | `codex/phase6-vault-foundation` |
| Baseline commit | `820952e` (PR #8 merge) |
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
| `M-CRED-HARDEN` | Harden AI credential storage and fix deadlock | `complete` | PR #3 merged; 33/33 Rust tests pass with Windows DPAPI. |
| `M-CODEX-ONLY` | Codex-only engineering workflow | `complete` | ADR-008, repository instructions, workflow, and control templates require no second AI agent. |
| `M-RUST-QUALITY` | Restore Rust formatting and strict lint gates | `complete` | Formatting, Clippy, tests, and production builds all pass. |
| `PHASE34-CLOSEOUT` | Close Spaces and Notes MVP gaps | `complete` | All contract acceptance criteria pass with regression coverage. |
| `PHASE5-001` | Task persistence and IPC foundation | `complete` | ADR-009 acceptance criteria and all quality gates pass. |
| `PHASE5-002` | Tasks UI and Pulse integration | `complete` | Space and global Task workflows, Pulse attention, tests, and quality gates pass. |
| `PHASE6-001` | Vault persistence and native filesystem foundation | `complete` | Merged through PR #9 at `b202c8e`. |
| `PHASE6-002` | Vault UI and Space integration | `complete` | Merged through PR #10 at `c352e67`. |
| `PHASE7-001` | AI streaming and context foundation | `self_review` | Current provider, cancellation, persistence, and explicit context gates pass; publication pending. |

The active task is `PHASE7-001`, governed by the self-review contract in `.ai/HANDOFF.md`.

## Completed product milestones

| Milestone | Status | Evidence summary |
| --- | --- | --- |
| Phase 0 — Foundation | `complete` | Repository structure, product specification, tooling, and Tauri foundation exist. |
| Phase 1 — Shell and design system | `complete` | App shell, routes, sidebar, command palette, tokens, and themes exist. |
| Phase 2 — Local database | `complete` | SQLite, migrations, repositories, commands, TS wrappers, and tests exist. |
| Phase 3 — Spaces | `complete` | CRUD, hierarchy, templates, module editing, archive/restore, favourite, duplication, deletion, synchronized views, and accessible reorder flows are implemented. |
| Phase 4 — Notes | `complete` | Persistence, serialized autosave with teardown flush, current-Space full-content search, pin, archive/restore/delete, move, and duplication are implemented. |
| Phase 7 — AI integration | `prototype` | DeepSeek provider, conversation persistence, context commands, and credential prototype exist; UI and production hardening do not. |

## Not started or placeholder milestones

| Milestone | Status | Notes |
| --- | --- | --- |
| Phase 5 — Tasks | `complete` | Global and Space Tasks, subtasks, full editing, search/filtering, completion, archive recovery, Pulse attention, persistence, and Activity events are implemented. |
| Phase 6 — Vault | `mvp_complete` | Safe linked/managed storage, global/Space UI, metadata, filters, open/reveal, and removal workflows pass all gates. |
| Phase 8 — Memory | `not_started` | No explicit memory domain located. |
| Phase 9 — Native desktop | `partial` | Icons, Tauri bundle config, and a built executable exist; remaining native features are not verified. |
| Phase 10 — Release quality | `not_started` | No complete release audit or release pipeline verified. |

## Quality snapshot

| Check | Last result | Date | Notes |
| --- | --- | --- | --- |
| `pnpm check` | Pass | 2026-08-10 | Typecheck and lint clean; Vitest 31/31 across ten files. |
| `pnpm build` | Pass | 2026-08-10 | TypeScript and Vite production build pass. |
| `cargo test` | Pass | 2026-08-10 | 56/56 tests pass, including AI streaming, cancellation, context isolation, and Vault safety. |
| `cargo build` | Pass | 2026-08-10 | Windows production build compiles with DPAPI. |
| `cargo fmt --check` | Pass | 2026-08-10 | Repository Rust formatting is clean; `DEBT-005` resolved. |
| `cargo clippy --all-targets -- -D warnings` | Pass | 2026-08-10 | Library, binary, and test targets are warning-free; `DEBT-006` resolved. |

## Active blockers

| ID | Scope | Blocker | Owner | Resolution path |
| --- | --- | --- | --- | --- |
None. `BLOCK-001` was resolved by installing Rust 1.97.1 with the stable MSVC toolchain and Visual Studio 2022 C++ build tools.

## Active risks

| ID | Severity | Summary | Tracking task |
| --- | --- | --- | --- |
| `RISK-001` | ~~High~~ Resolved | Credential crypto no longer acquires the database mutex; 33/33 tests pass and PR #3 is merged. | `TECH-001` |
| `RISK-002` | ~~High~~ Resolved | Path-derived encryption was replaced with current-user-bound Windows DPAPI in merged PR #3. | `TECH-001` |
| `RISK-003` | ~~High~~ Resolved | README, package metadata, Tauri metadata, and agent phase guidance disagree on version/maturity. | `DOC-001` — reconciled all metadata to 0.3.0-alpha. |

## Decision index

Full rationale belongs in `.ai/ARCHITECTURE.md` or a dedicated ADR under `docs/decisions/`.

| ID | Decision | Status |
| --- | --- | --- |
| `ADR-001` | Tauri 2 is the Windows desktop shell. | Accepted |
| `ADR-002` | SQLite via bundled `rusqlite` is the local persistence layer. | Accepted |
| `ADR-003` | Frontend database access goes through typed Tauri invoke wrappers. | Accepted |
| `ADR-004` | Hybrid Hermes/Codex task routing. | Superseded by `ADR-008` |
| `ADR-005` | `.ai/` documents are the collaboration source of truth. | Accepted |
| `ADR-007` | Verified task commits are automatically pushed and published through guarded repository tooling. | Accepted |
| `ADR-008` | Codex is the sole engineering agent; complex work uses task contracts and evidence-based self-review. | Accepted |
| `ADR-009` | Tasks use one persistent tree with nullable Space ownership and local-date due semantics. | Accepted |
| `ADR-010` | Vault distinguishes linked and managed ownership and enforces deletion safety in Rust. | Accepted |
| `ADR-011` | AI uses cancellable typed streams and explicit Space-isolated context resolved in Rust. | Accepted |

## Next: Publish `PHASE7-001`, then build the complete AI Settings/chat/context/proposal UI in `PHASE7-002`.

## Reusable update template

```markdown
## Current milestone
| ID | Name | Status | Exit condition |
| --- | --- | --- | --- |
| `M-...` | <name> | `planned|active|blocked|self_review|complete` | <observable condition> |

## Active blockers
| ID | Scope | Blocker | Owner | Resolution path |
| --- | --- | --- | --- | --- |

## Quality snapshot
| Check | Last result | Date | Notes |
| --- | --- | --- | --- |
```
