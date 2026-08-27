# Prioritized Backlog

> Canonical queue of work that is not the active implementation contract.

| Field          | Value                                   |
| -------------- | --------------------------------------- |
| Schema version | 1                                       |
| Last updated   | 2026-08-27                              |
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

| Priority | ID                 | Work item                                                  | Type                      | Status | Dependencies                | Acceptance summary                                                                                                      |
| -------- | ------------------ | ---------------------------------------------------------- | ------------------------- | ------ | --------------------------- | ----------------------------------------------------------------------------------------------------------------------- |
| P0       | `AIWF-001`         | Establish Hermes/Codex collaboration system                | Process                   | `done` | None                        | Hermes accepted the complete workflow.                                                                                  |
| P0       | `PROC-003`         | Adopt Codex-only engineering workflow                      | Process                   | `done` | Project-owner decision      | Codex owns planning through publication with planned-task contracts and self-review.                                    |
| P0       | `TECH-001`         | Audit and harden AI credential storage and locking         | Security                  | `done` | `ENV-001` done              | Merged via PR #3; Windows DPAPI enabled and 33/33 Rust tests pass.                                                      |
| P1       | `ENV-001`          | Restore Rust verification capability                       | Tooling                   | `done` | Rust MSVC installation      | Rust tests, formatting checks, and Clippy run locally; code findings are tracked separately.                            |
| P1       | `DOC-001`          | Reconcile version and milestone documentation              | Documentation             | `done` | `AIWF-001` accepted         | README, app metadata, and project state agree.                                                                          |
| P1       | `PHASE34-CLOSEOUT` | Close Spaces and Notes MVP gaps                            | Product and testing       | `done` | Rust quality gates restored | Space editing/reordering/state refresh and safe Note autosave/archive flows pass regression tests.                      |
| P1       | `PHASE5-EPIC`      | Design and implement Tasks                                 | Product                   | `done` | Codex decomposition         | ADR-009, persistence, Space/global UI, and Pulse attention are complete.                                                |
| P1       | `PHASE5-001`       | Build Task persistence and IPC foundation                  | Product and architecture  | `done` | ADR-009                     | Migration, repository, commands, types, wrappers, and tests pass.                                                       |
| P1       | `PHASE5-002`       | Build Tasks UI and Pulse integration                       | Product                   | `done` | `PHASE5-001`                | Space Tasks and calm global due views meet Phase 5 acceptance.                                                          |
| P1       | `AI-UI-EPIC`       | Complete the AI user experience                            | Product                   | `done` | `TECH-001`                  | Provider setup, conversations, streaming, cancellation, explicit context, modes, and Task proposals have reviewed UX.   |
| P1       | `PHASE7-001`       | Harden AI streaming and explicit context foundation        | Security and architecture | `done` | ADR-011                     | Current provider, cancellation, terminal persistence, and Space isolation pass tests.                                   |
| P1       | `PHASE7-002`       | Build AI Settings, chat, context, summary, and proposal UI | Product                   | `done` | `PHASE7-001`                | DeepSeek is usable with visible context, retry/cancel, modes, and previewed transactional proposals.                    |
| P1       | `PHASE8-001`       | Design and implement Memory                                | Product and architecture  | `done` | `PHASE7-002`                | Global and Space Memory is manageable and may be explicitly attached to AI.                                             |
| P1       | `PHASE9-001`       | Complete native Windows features                           | Product and packaging     | `done` | `PHASE8-001`                | Tray, notifications, shortcut, installer metadata, and native smoke tests pass.                                         |
| P1       | `PHASE10-001`      | Complete release audit and pipeline                        | Quality and release       | `done` | `PHASE9-001`                | CI, packaging, documentation, security/privacy, backup export, and alpha release gates pass.                            |
| P0       | `STAB-001`         | Integrated stress test and defect hardening                | Quality and defect        | `done` | `PHASE10-001`               | Full browser/desktop test matrix passes and all reproducible P0/P1 defects are resolved.                                |
| P0       | `HARD-001`         | Safely upgrade personal-beta databases                     | Defect and data lifecycle | `done` | Milestone A                 | Older databases start on 0.3.1 with valid rows and managed-file safety preserved.                                       |
| P1       | `AI-CHAT-001`      | Render submitted AI prompts immediately                    | Defect                    | `done` | `PHASE7-002`                | A submitted prompt is visible before backend streaming starts and is reconciled without duplication.                    |
| P0       | `AI-CHAT-002`      | Prevent AI stream rerender crash                           | Defect                    | `done` | `AI-CHAT-001`               | Streamed message updates rerender without invoking a Promise as a React effect cleanup or showing the error boundary.   |
| P0       | `AI-CHAT-003`      | Keep new prompts visible across stale message loads        | Defect and release        | `done` | `AI-CHAT-002`               | An older load cannot overwrite a newly started stream; Alpha 0.3.1 installs as an explicit Windows patch upgrade.       |
| P0       | `AI-CHAT-004`      | Show completed AI responses without reloading              | Defect                    | `done` | `AI-CHAT-003`               | Missed WebView stream events cannot leave a persisted response hidden until Ctrl+R.                                     |
| P1       | `UI-001`           | Redesign the complete Aether interface system              | Product and accessibility | `done` | Owner direction, ADR-015    | Every core surface feels cohesive, premium, responsive, accessible, and distinctly Aether without behavior regressions. |
| P0       | `RELEASE-032`      | Consolidate and install Alpha 0.3.2                        | Quality and release       | `done` | PRs #34, #35, #36 merged    | Versioned Windows artifacts are validated, hashed, installed safely, and published through green draft PR #37.          |
| P0       | `CTX-001`          | Add explicit Sources and safe metadata indexing            | Product, privacy, data    | `done` | Milestone A and ADR-016     | Users authorize, inspect, rescan, associate, and revoke bounded local directory Sources without file mutation.          |
| P0       | `SEARCH-001`       | Design and implement Universal Search                      | Product, privacy, data    | `done` | `CTX-001`                   | Users search permitted local domains quickly with clear type, scope, provenance, and no implicit AI disclosure.         |
| P0       | `CONT-001`         | Build Continuity and meaningful Activity                   | Product, privacy, data    | `done` | `SEARCH-001`                | Each Space offers a concise, deterministic resume view and Activity shows only meaningful local changes.                |
| P1       | `VAULT-EPIC`       | Design and implement Vault                                 | Product                   | `done` | ADR-010                     | Safe storage and complete global/Space MVP are implemented.                                                             |
| P1       | `PHASE6-001`       | Build Vault persistence and native filesystem foundation   | Product and security      | `done` | ADR-010                     | Safe linked/managed storage commands and tests pass.                                                                    |
| P1       | `PHASE6-002`       | Build Vault UI and Space integration                       | Product                   | `done` | `PHASE6-001`                | Import, metadata, search, open/reveal, and safe removal are usable.                                                     |

## Technical debt

| Priority | ID         | Area              | Description                                                                                                      | Evidence                                                                                                         |
| -------- | ---------- | ----------------- | ---------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------- |
| P1       | `DEBT-001` | Testing           | Frontend coverage now includes Spaces invalidation, Note autosave races/teardown, and Tauri argument boundaries. | Resolved by `PHASE34-CLOSEOUT`: 14/14 tests across five files.                                                   |
| P2       | `DEBT-002` | Lint              | Four non-failing warnings were in utilities, IconPicker, and SpaceDetail.                                        | ~~`pnpm lint` on 2026-07-30~~ Resolved: 0 warnings after fix (commit `28b82ab`).                                 |
| P2       | `DEBT-003` | Architecture docs | Existing `docs/architecture.md` contains a duplicated Tauri Bridge layer and predates Notes/AI work.             | Repository inspection                                                                                            |
| P2       | `DEBT-004` | Packaging         | `Cargo.toml` repository field was empty despite configured Git remote.                                           | ~~Repository inspection~~ Resolved: populated with https://github.com/bimberlotDEV/Aether-Desktop.               |
| P1       | `DEBT-005` | Rust formatting   | Rust sources now satisfy the repository formatting gate.                                                         | Resolved: `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` passes on 2026-08-10.                      |
| P1       | `DEBT-006` | Rust lint         | Strict Clippy is clean across library, binary, and test targets.                                                 | Resolved: `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` passes on 2026-08-10. |

## Feature status index

| Feature        | Status                | Next planning action                                                                  |
| -------------- | --------------------- | ------------------------------------------------------------------------------------- |
| Spaces         | `mvp_complete`        | Extend only when a later roadmap phase requires it.                                   |
| Notes          | `mvp_complete`        | Extend only when a later roadmap phase requires it.                                   |
| Tasks          | `mvp_complete`        | Extend only when a later roadmap phase requires it.                                   |
| Vault          | `mvp_complete`        | Extend only when a later roadmap phase requires previews or indexing.                 |
| AI             | `mvp_complete`        | Validate live provider behavior during the release smoke test.                        |
| Memory         | `mvp_complete`        | Extend only after a separately reviewed automatic-suggestion consent design.          |
| Native desktop | `alpha_complete`      | Activate signing/updater only after owner-controlled trust infrastructure exists.     |
| Backup/export  | `foundation_complete` | Design restore and Vault-byte archive semantics as a separate later task.             |
| Context engine | `continuity_complete` | Validate later intelligence only through separately reviewed, explicit user controls. |

## New item template

```markdown
| P0-P3 | `AREA-NNN` | <imperative work item> | Product/Defect/Security/Tooling/Docs/Process | `candidate` | <IDs or None> | <observable outcome> |
```

Codex assigns the ID, priority, dependency chain, and readiness status within the owner-approved roadmap. Material product reprioritization or scope expansion requires the project owner's direction.
