# Project State

> Canonical snapshot of what is true in the repository now. This is not a plan or historical log.

| Field            | Value                         |
| ---------------- | ----------------------------- |
| Schema version   | 1                             |
| Last updated     | 2026-08-28                    |
| Updated by       | Codex                         |
| Repository       | `bimberlotDEV/Aether-Desktop` |
| Branch           | `codex/release-040`           |
| Baseline commit  | `da4ca98` (current `master`)  |
| Product maturity | Alpha                         |

## Responsibility of this file

- Record completed milestones and the current milestone.
- Record active blockers and verified quality signals.
- Point to important decisions without duplicating their full rationale.
- Stay concise enough to read at the start of every agent session.

## Current milestone

| ID                 | Name                                               | Status     | Exit condition                                                                                                                      |
| ------------------ | -------------------------------------------------- | ---------- | ----------------------------------------------------------------------------------------------------------------------------------- |
| `M-AI-WORKFLOW`    | Hermes/Codex collaboration foundation              | `complete` | Hermes accepted `AIWF-001` and promoted `DOC-001`.                                                                                  |
| `M-DOC-VERSIONING` | Version and metadata reconciliation                | `complete` | Hermes accepted `DOC-001`.                                                                                                          |
| `M-AUTO-PUBLISH`   | Automatic GitHub task publication                  | `complete` | Draft PR #1 merged into `master`.                                                                                                   |
| `M-RUST-VERIFY`    | Restore local Rust verification                    | `complete` | Rust MSVC builds the test binaries and runs formatting, lint, and tests locally.                                                    |
| `M-CRED-HARDEN`    | Harden AI credential storage and fix deadlock      | `complete` | PR #3 merged; 33/33 Rust tests pass with Windows DPAPI.                                                                             |
| `M-CODEX-ONLY`     | Codex-only engineering workflow                    | `complete` | ADR-008, repository instructions, workflow, and control templates require no second AI agent.                                       |
| `M-RUST-QUALITY`   | Restore Rust formatting and strict lint gates      | `complete` | Formatting, Clippy, tests, and production builds all pass.                                                                          |
| `PHASE34-CLOSEOUT` | Close Spaces and Notes MVP gaps                    | `complete` | All contract acceptance criteria pass with regression coverage.                                                                     |
| `PHASE5-001`       | Task persistence and IPC foundation                | `complete` | ADR-009 acceptance criteria and all quality gates pass.                                                                             |
| `PHASE5-002`       | Tasks UI and Pulse integration                     | `complete` | Space and global Task workflows, Pulse attention, tests, and quality gates pass.                                                    |
| `PHASE6-001`       | Vault persistence and native filesystem foundation | `complete` | Merged through PR #9 at `b202c8e`.                                                                                                  |
| `PHASE6-002`       | Vault UI and Space integration                     | `complete` | Merged through PR #10 at `c352e67`.                                                                                                 |
| `PHASE7-001`       | AI streaming and context foundation                | `complete` | Merged through PR #11 at `7569d3f`.                                                                                                 |
| `PHASE7-002`       | Complete AI user experience                        | `complete` | Merged through PR #12 at `7838599`.                                                                                                 |
| `PHASE8-001`       | Explicit scoped Memory                             | `complete` | Persistence, global/Space management, AI attachment, tests, and quality gates pass.                                                 |
| `PHASE9-001`       | Native Windows lifecycle and packaging             | `complete` | Tray, shortcut, notifications, window state, MSI/NSIS build, and startup smoke test pass.                                           |
| `PHASE10-001`      | Quality and release preparation                    | `complete` | CI, sanitized export, audits, documentation, packaging, hashes, and startup verification pass.                                      |
| `STAB-001`         | Integrated alpha stabilization                     | `complete` | Automated, browser, desktop, native, responsive, accessibility, and error-path stress checks pass with verified defects repaired.   |
| `AI-CHAT-003`      | Eliminate stale-load prompt hiding and ship 0.3.1  | `complete` | A deterministic load/stream race test passes and the installed 0.3.1 bundle contains the verified frontend.                         |
| `AI-CHAT-004`      | Reconcile completed responses without page reload  | `complete` | Terminal events upsert missing messages and every completed stream reconciles the visible conversation with persisted SQLite state. |
| `HARD-001`         | Safely upgrade personal-beta databases             | `complete` | Populated legacy databases upgrade transactionally with current repository and Vault safety invariants intact.                      |
| `UI-001`           | Reimagine the Aether interface system              | `complete` | Every core surface is cohesive, premium, responsive, accessible, and distinctly Aether without behavior regressions.                |
| `RELEASE-032`      | Consolidate and install Alpha 0.3.2                | `complete` | Merged AI, migration, and UI work ships as a versioned, hashed, installed, and CI-green Windows release candidate.                  |
| `CTX-001`          | Explicit Sources and metadata indexing             | `complete` | Authorized directories can be safely indexed, inspected, rescanned, associated, and revoked without mutating user files.            |
| `SEARCH-001`       | Universal Search                                   | `complete` | Ctrl+K searches commands and permitted local domains quickly with deterministic ranking, provenance, and safe navigation.           |
| `CONT-001`         | Continuity and meaningful Activity                 | `complete` | Every Space has a deterministic resume view and Activity contains only curated, presentation-safe local changes.                    |
| `PULSE-002`        | Pulse 2.0                                          | `complete` | Pulse makes today’s real local relevance visible without noise, hidden AI, or mutation.                                             |
| `ACTION-001`       | Safe Actions                                       | `complete` | Typed preview, approval, one-time execution, containment, rollback, audit, UX, packaging, and exact-head CI pass.                   |
| `AI-EVOL-001`      | AI Evolution                                       | `complete`  | DeepSeek/OpenAI, transparent Auto routing, response provenance, approved Task/Note drafts, local release gates, and exact-head CI pass. |
| `RELEASE-040`      | Cut and install integrated Alpha 0.4.0             | `self_review` | Local gates, protected upgrade, artifacts, data preservation, and startup pass; publication and exact-head CI remain.                  |

Milestones A through G are merged and verified on `master` through PR #43. `RELEASE-040` is creating an honest, protected Windows upgrade boundary for the integrated product.

## Completed product milestones

| Milestone                         | Status         | Evidence summary                                                                                                                                                           |
| --------------------------------- | -------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Phase 0 — Foundation              | `complete`     | Repository structure, product specification, tooling, and Tauri foundation exist.                                                                                          |
| Phase 1 — Shell and design system | `complete`     | App shell, routes, sidebar, command palette, tokens, and themes exist.                                                                                                     |
| Phase 2 — Local database          | `complete`     | SQLite, migrations, repositories, commands, TS wrappers, and tests exist.                                                                                                  |
| Phase 3 — Spaces                  | `complete`     | CRUD, hierarchy, templates, module editing, archive/restore, favourite, duplication, deletion, synchronized views, and accessible reorder flows are implemented.           |
| Phase 4 — Notes                   | `complete`     | Persistence, serialized autosave with teardown flush, current-Space full-content search, pin, archive/restore/delete, move, and duplication are implemented.               |
| Phase 7 — AI integration          | `mvp_complete` | Secure credentials, current cancellable streaming, persisted conversations, explicit Space-isolated context, response modes, and confirmed Task proposals are implemented. |
| Milestone B — Context Foundation  | `complete`     | Explicit Sources, bounded metadata-only indexing, change reconciliation, inspection, Space association, and safe revocation are implemented.                               |
| Milestone C — Universal Search    | `complete`     | Ctrl+K searches commands and permitted local domains with bounded deterministic ranking, provenance, privacy-safe results, and accessible navigation.                      |
| Milestone D — Continuity          | `complete`     | Space resume views and meaningful Activity are deterministic, bounded, local, and Space-isolated.                                                                          |
| Milestone E — Pulse 2.0           | `complete`     | Today, Continue, New, Recent, and the suggested next step use factual local signals without hidden AI or mutation.                                                         |
| Milestone F — Safe Actions        | `complete`     | Eight typed actions require visible review and one-time approval; native execution is contained, non-destructive, audited, and release-gated.                              |

## Not started or placeholder milestones

| Milestone                  | Status         | Notes                                                                                                                                                                   |
| -------------------------- | -------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Phase 5 — Tasks            | `complete`     | Global and Space Tasks, subtasks, full editing, search/filtering, completion, archive recovery, Pulse attention, persistence, and Activity events are implemented.      |
| Phase 6 — Vault            | `mvp_complete` | Safe linked/managed storage, global/Space UI, metadata, filters, open/reveal, and removal workflows pass all gates.                                                     |
| Phase 8 — Memory           | `mvp_complete` | Explicit user-authored global/Space Memory, management UI, safe deletion, and AI context integration are implemented.                                                   |
| Phase 9 — Native desktop   | `complete`     | Tray lifecycle, shortcut, notifications, window-state, icons/metadata, MSI and NSIS installers are verified.                                                            |
| Phase 10 — Release quality | `complete`     | Windows CI, dependency monitoring, sanitized export, accessibility/security/performance review, release docs, MSI/NSIS bundles, hashes, and startup smoke are verified. |

## Quality snapshot

| Check                                       | Last result | Date       | Notes                                                                                                                 |
| ------------------------------------------- | ----------- | ---------- | --------------------------------------------------------------------------------------------------------------------- |
| `pnpm check`                                | Pass        | 2026-08-27 | Safe Actions branch: 77/77 tests across 29 files.                                                                     |
| `pnpm build`                                | Pass        | 2026-08-27 | Production build passes; Actions is a lazy 10.01 kB chunk.                                                            |
| `pnpm audit --audit-level high`             | Pass        | 2026-08-27 | No known vulnerabilities.                                                                                             |
| `cargo test`                                | Pass        | 2026-08-27 | 85/85 tests pass, including containment, symlink escape, one-time execution, and rollback.                            |
| `cargo build`                               | Pass        | 2026-08-11 | Windows production build compiles with DPAPI and single-instance support.                                             |
| `cargo fmt --check`                         | Pass        | 2026-08-27 | Repository Rust formatting is clean.                                                                                  |
| `cargo clippy --all-targets -- -D warnings` | Pass        | 2026-08-27 | Library, binary, and test targets are warning-free.                                                                   |
| `pnpm tauri:build`                          | Pass        | 2026-08-27 | Safe Actions candidate x64 MSI and NSIS bundles build successfully.                                                   |
| GitHub Actions                              | Pass        | 2026-08-27 | Run 33092882774 passes frontend quality/build, Rust format, strict lint, and Rust tests on exact implementation head. |
| Release startup smoke                       | Pass        | 2026-08-27 | Exact workspace release executable started and remained responsive.                                                   |

## Active blockers

| ID                                                                                                                              | Scope | Blocker | Owner | Resolution path |
| ------------------------------------------------------------------------------------------------------------------------------- | ----- | ------- | ----- | --------------- |
| None. `BLOCK-001` was resolved by installing Rust 1.97.1 with the stable MSVC toolchain and Visual Studio 2022 C++ build tools. |

## Active risks

| ID         | Severity          | Summary                                                                                          | Tracking task                                                                                                                                       |
| ---------- | ----------------- | ------------------------------------------------------------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------- |
| `RISK-001` | ~~High~~ Resolved | Credential crypto no longer acquires the database mutex; 33/33 tests pass and PR #3 is merged.   | `TECH-001`                                                                                                                                          |
| `RISK-002` | ~~High~~ Resolved | Path-derived encryption was replaced with current-user-bound Windows DPAPI in merged PR #3.      | `TECH-001`                                                                                                                                          |
| `RISK-003` | ~~High~~ Resolved | README, package metadata, Tauri metadata, and agent phase guidance disagree on version/maturity. | `DOC-001` reconciled product maturity; `PHASE9-001` normalized machine versions to MSI-compatible 0.3.0 while retaining Alpha as the product phase. |

## Decision index

Full rationale belongs in `.ai/ARCHITECTURE.md` or a dedicated ADR under `docs/decisions/`.

| ID        | Decision                                                                                                                     | Status                  |
| --------- | ---------------------------------------------------------------------------------------------------------------------------- | ----------------------- |
| `ADR-001` | Tauri 2 is the Windows desktop shell.                                                                                        | Accepted                |
| `ADR-002` | SQLite via bundled `rusqlite` is the local persistence layer.                                                                | Accepted                |
| `ADR-003` | Frontend database access goes through typed Tauri invoke wrappers.                                                           | Accepted                |
| `ADR-004` | Hybrid Hermes/Codex task routing.                                                                                            | Superseded by `ADR-008` |
| `ADR-005` | `.ai/` documents are the collaboration source of truth.                                                                      | Accepted                |
| `ADR-007` | Verified task commits are automatically pushed and published through guarded repository tooling.                             | Accepted                |
| `ADR-008` | Codex is the sole engineering agent; complex work uses task contracts and evidence-based self-review.                        | Accepted                |
| `ADR-009` | Tasks use one persistent tree with nullable Space ownership and local-date due semantics.                                    | Accepted                |
| `ADR-010` | Vault distinguishes linked and managed ownership and enforces deletion safety in Rust.                                       | Accepted                |
| `ADR-011` | AI uses cancellable typed streams and explicit Space-isolated context resolved in Rust.                                      | Accepted                |
| `ADR-012` | Memory is explicit, user-authored, scoped, attributable, and attached to AI only by choice.                                  | Accepted                |
| `ADR-013` | Native Windows lifecycle uses tray persistence, non-fatal shortcut registration, OS notifications, and gated signed updates. | Accepted                |
| `ADR-014` | Workspace export uses a sanitized, integrity-checked SQLite snapshot and excludes credentials and Vault file bytes.          | Accepted                |
| `ADR-015` | Build Aether's frontend from an internal semantic interface system with a distinctive shell and reusable primitives.         | Accepted                |
| `ADR-016` | Require explicit Sources and bounded, metadata-only, non-mutating local directory snapshots before file intelligence.        | Accepted                |

## Next

Milestones A through G are complete. Alpha 0.4.0 release consolidation is active. Public signing, updater activation, automated restore, and Vault-byte archives remain separately governed work.

## Reusable update template

```markdown
## Current milestone

| ID      | Name   | Status   | Exit condition |
| ------- | ------ | -------- | -------------- |
| `M-...` | <name> | `planned | active         | blocked | self_review | complete` | <observable condition> |

## Active blockers

| ID  | Scope | Blocker | Owner | Resolution path |
| --- | ----- | ------- | ----- | --------------- |

## Quality snapshot

| Check | Last result | Date | Notes |
| ----- | ----------- | ---- | ----- |
```
