# Project State

> Canonical snapshot of what is true in the Aether repository now.
>
> This file records current verified state. It is not a roadmap, planning document, session log, or historical narrative.

| Field                   | Value                              |
| ----------------------- | ---------------------------------- |
| Schema version          | 2                                  |
| Last updated            | 2026-09-21                         |
| Updated by              | Codex                              |
| Repository              | `bimberlotDEV/Aether-Desktop`      |
| Product maturity        | Alpha                              |
| Current product version | `0.5.0`                            |
| Engineering workflow    | Repository-centered Codex workflow |

---

## Responsibility of this file

This file exists to answer:

> What is true in Aether right now?

It should:

* record completed product milestones;
* record the current product/release state;
* record active blockers;
* record verified quality signals;
* reference important architecture decisions;
* remain concise enough to read at the beginning of every Codex session.

It should NOT:

* duplicate full architecture rationale;
* contain detailed task contracts;
* contain session-by-session history;
* act as the backlog;
* act as a roadmap;
* preserve obsolete collaboration mechanisms as active project concepts.

Durable rationale belongs in:

* `.ai/ARCHITECTURE.md`;
* ADRs under `docs/decisions/`.

Historical implementation records belong in:

* `.ai/CHANGELOG.md`.

Current task detail belongs in:

* `.ai/HANDOFF.md`.

Prioritized future work belongs in:

* `.ai/TODO.md`.

---

# Current product state

Aether is an installable, local-first Windows desktop application.

The current verified product foundation includes:

* Tauri 2 + Rust Windows desktop shell;
* React + TypeScript frontend;
* bundled SQLite persistence;
* Spaces;
* Notes;
* Tasks;
* Vault;
* explicit Memory;
* local Sources;
* Universal Search;
* deterministic Continuity;
* meaningful Activity;
* Pulse 2.0;
* Safe Actions;
* DeepSeek/OpenAI provider support;
* transparent AI routing;
* persisted AI conversations;
* approved AI Task/Note proposals;
* Windows tray lifecycle;
* global shortcut;
* native notifications;
* window-state persistence;
* portable backup;
* verified restore;
* Windows packaging;
* release tooling;
* update-ready trusted-release infrastructure;
* onboarding;
* native subscribed-calendar ingestion primitives (publication pending);
* automated quality gates.

Aether remains local-first.

There is no requirement for a cloud account to use the core product.

---

# Current milestone state

## Product development milestones

| ID        | Name                    | Status         | Exit condition                                                                                                                     |
| --------- | ----------------------- | -------------- | ---------------------------------------------------------------------------------------------------------------------------------- |
| `PHASE0`  | Foundation              | `complete`     | Repository, product specification, tooling, and Tauri foundation verified.                                                         |
| `PHASE1`  | Shell and design system | `complete`     | Application shell, routing, navigation, themes, tokens, and core interface primitives verified.                                    |
| `PHASE2`  | Local database          | `complete`     | SQLite, migrations, repositories, typed IPC, and validation infrastructure verified.                                               |
| `PHASE3`  | Spaces                  | `complete`     | Space lifecycle, hierarchy, templates, configuration, archive/restore, duplication, and deletion verified.                         |
| `PHASE4`  | Notes                   | `complete`     | Persistence, autosave, search, archive/restore, movement, duplication, and Space workflows verified.                               |
| `PHASE5`  | Tasks                   | `complete`     | Global/Space Tasks, subtasks, filtering, editing, completion, archive recovery, and Pulse integration verified.                    |
| `PHASE6`  | Vault                   | `mvp_complete` | Safe linked/managed storage, metadata, UI, open/reveal, filters, Space integration, and deletion behavior verified.                |
| `PHASE7`  | AI integration          | `mvp_complete` | Secure credentials, streaming, persisted conversations, Space-isolated context, provider routing, and approved proposals verified. |
| `PHASE8`  | Memory                  | `mvp_complete` | Explicit global/Space Memory, management UI, deletion, and optional AI-context attachment verified.                                |
| `PHASE9`  | Native desktop          | `complete`     | Tray, shortcut, notifications, window state, packaging, and Windows startup behavior verified.                                     |
| `PHASE10` | Release quality         | `complete`     | CI, audits, backups, packaging, release documentation, integrity checks, and startup smoke testing verified.                       |

---

# Product evolution milestones

| Milestone                             | Status             | Verified outcome                                                                                                           |
| ------------------------------------- | ------------------ | -------------------------------------------------------------------------------------------------------------------------- |
| Milestone A — Core personal workspace | `complete`         | Foundational local-first workspace capabilities are operational.                                                           |
| Milestone B — Context Foundation      | `complete`         | Explicit Sources with bounded metadata-only indexing are operational.                                                      |
| Milestone C — Universal Search        | `complete`         | `Ctrl+K` searches commands and permitted local domains with deterministic ranking and provenance.                          |
| Milestone D — Continuity              | `complete`         | Space resume state and meaningful Activity are deterministic, bounded, local, and Space-isolated.                          |
| Milestone E — Pulse 2.0               | `complete`         | Pulse presents factual local relevance without hidden AI or mutation.                                                      |
| Milestone F — Safe Actions            | `complete`         | Typed preview, explicit approval, one-time execution, containment, rollback, and audit behavior are operational.           |
| Milestone G — AI Evolution            | `complete`         | DeepSeek/OpenAI support, transparent routing, provenance, approved Task/Note drafts, and release gates are operational.    |
| Milestone H — Onboarding & UX         | `complete`         | Upgrade-safe onboarding, optional trusted configuration, accessibility, packaging, and release validation are operational. |
| Milestone I — Public Beta             | `blocked_external` | Repository readiness exists; completion requires signed publication and meaningful external tester evidence.               |
| Milestone J — Commercial Readiness    | `blocked_external` | Architecture is decision-ready; implementation depends on beta evidence and owner/legal/business/provider decisions.       |
| Milestone K — 1.0 Launch              | `blocked_external` | Launch gates are defined; completion requires real reliability, support, retention, and owner-defined threshold evidence.  |

---

# Recent completed milestones

| ID                   | Name                                  | Status     | Verified outcome                                                                                                                                            |
| -------------------- | ------------------------------------- | ---------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `M-AUTO-PUBLISH`     | Automatic GitHub task publication     | `complete` | Verified task publication tooling is operational.                                                                                                           |
| `M-RUST-VERIFY`      | Local Rust verification               | `complete` | Rust MSVC toolchain runs formatting, lint, tests, and release builds locally.                                                                               |
| `M-CRED-HARDEN`      | AI credential hardening               | `complete` | Credential storage uses Windows DPAPI and previous locking issues are resolved.                                                                             |
| `M-CODEX-ONLY`       | Codex engineering workflow            | `complete` | Repository instructions, task contracts, self-review, and control documents support Codex-owned engineering without a required secondary engineering agent. |
| `M-RUST-QUALITY`     | Rust quality gates                    | `complete` | Formatting, strict Clippy, tests, and production builds pass.                                                                                               |
| `STAB-001`           | Integrated alpha stabilization        | `complete` | Automated, native, responsive, accessibility, browser, and error-path validation completed with discovered defects repaired.                                |
| `AI-CHAT-003`        | Eliminate stale-load prompt hiding    | `complete` | Prompt visibility race fixed and verified.                                                                                                                  |
| `AI-CHAT-004`        | Reconcile completed AI responses      | `complete` | Persisted completed responses reconcile into the visible conversation without requiring page reload.                                                        |
| `HARD-001`           | Personal-beta database upgrade safety | `complete` | Existing databases upgrade transactionally while preserving repository and Vault invariants.                                                                |
| `UI-001`             | Aether interface redesign             | `complete` | Core product surfaces share a cohesive, premium, responsive, and accessible interface.                                                                      |
| `CTX-001`            | Sources and metadata indexing         | `complete` | Explicit authorized directories can be safely indexed, inspected, rescanned, associated, and revoked.                                                       |
| `SEARCH-001`         | Universal Search                      | `complete` | Local permitted domains and commands are searchable through deterministic bounded search.                                                                   |
| `CONT-001`           | Continuity and meaningful Activity    | `complete` | Space resume state and Activity are deterministic and presentation-safe.                                                                                    |
| `PULSE-002`          | Pulse 2.0                             | `complete` | Pulse surfaces factual local relevance without hidden AI.                                                                                                   |
| `ACTION-001`         | Safe Actions                          | `complete` | Typed proposals require visible review and bounded one-time approval before execution.                                                                      |
| `AI-EVOL-001`        | AI Evolution                          | `complete` | Multi-provider routing, provenance, and approved Task/Note proposals are operational.                                                                       |
| `RELEASE-040`        | Integrated Alpha 0.4.0                | `complete` | Release gates, protected upgrade, packaging, data preservation, and startup verification completed.                                                         |
| `BACKUP-RESTORE-001` | Portable backup and safe restore      | `complete` | Portable archives include managed Vault bytes and use verified, approval-gated, rollback-safe restore.                                                      |
| `RELEASE-TRUST-001`  | Trusted Releases & Updates            | `complete` | Owner-gated release tooling and Rust-owned stable-update architecture are implemented.                                                                      |
| `ONBOARD-001`        | Onboarding & UX                       | `complete` | First-run onboarding and upgrade-safe configuration are implemented and verified.                                                                           |

---

# Current release state

Current repository product version:

```text
0.5.0
```

Current verified repository quality snapshot is based on the latest recorded 0.5.0 readiness work.

The product remains Alpha.

Alpha indicates product maturity and evidence level, not whether the application is functional.

Aether is already:

* installable;
* usable;
* persistent;
* upgradeable;
* test-covered;
* packageable;
* backup-capable;
* restore-capable;
* AI-enabled;
* native Windows software.

Public Beta is not considered complete until the external evidence requirements are satisfied.

---

# Quality snapshot

| Check                                       | Last verified result | Verified date | Notes                                                                           |
| ------------------------------------------- | -------------------- | ------------- | ------------------------------------------------------------------------------- |
| `pnpm check`                                | Pass                 | 2026-08-29    | 100/100 frontend tests across 33 files; typecheck and lint pass.                |
| `pnpm build`                                | Pass                 | 2026-08-29    | Alpha 0.5.0 production frontend build passes.                                   |
| `pnpm audit --audit-level high`             | Pass                 | 2026-08-29    | No known high-severity vulnerabilities reported.                                |
| `cargo test`                                | Pass                 | 2026-08-29    | 108/108 all-feature Rust tests pass.                                            |
| `cargo build --release`                     | Pass                 | 2026-08-29    | Optimized Aether 0.5.0 Windows executable builds and starts.                    |
| `cargo fmt --check`                         | Pass                 | 2026-08-29    | Rust formatting clean.                                                          |
| `cargo clippy --all-targets -- -D warnings` | Pass                 | 2026-08-29    | Relevant targets/features warning-free.                                         |
| `pnpm tauri:build`                          | Pass                 | 2026-08-29    | Aether 0.5.0 x64 MSI and NSIS bundles build successfully.                       |
| GitHub Actions                              | Pass                 | 2026-08-29    | Frontend quality/build and Rust validation pass on the recorded readiness head. |
| Release startup smoke                       | Pass                 | 2026-08-29    | Optimized Aether 0.5.0 starts and exposes a responsive native window.           |

These are historical verification results.

Do not assume they remain current after new code changes.

Every task must run the validation required by its own scope.

---

# Active blockers

The repository currently has no known blocker preventing normal local Aether feature development.

The following blockers apply only to external beta/commercial/launch progression.

| ID           | Scope       | Blocker                                                                                                                          | Owner                                       | Resolution path                                                                   |
| ------------ | ----------- | -------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------- | --------------------------------------------------------------------------------- |
| `BETA-EXT`   | Milestone I | No owner-signed published beta or meaningful external activation/retention/reliability evidence exists.                          | Product owner                               | Complete trusted publication, enroll real testers, and collect evidence.          |
| `COMM-EXT`   | Milestone J | Required legal/business/provider/payment/domain decisions and beta economics evidence do not yet exist.                          | Product owner + relevant external expertise | Resolve owner decision register after sufficient beta evidence exists.            |
| `LAUNCH-EXT` | Milestone K | Final thresholds, eligible observation windows, reliability evidence, retention evidence, and support evidence do not yet exist. | Product owner                               | Define thresholds before evaluation and complete the required observation period. |

These blockers must NOT prevent unrelated local product development unless a task explicitly depends on them.

---

# Active engineering risks

No unresolved high-severity repository engineering risk is currently recorded in this snapshot.

Previously resolved risks include:

* database-lock interaction with credential encryption;
* path-derived credential encryption;
* inconsistent product version/maturity documentation.

If new risks are identified, record only active risks here.

Resolved risks should move to historical records rather than accumulating indefinitely in this file.

---

# Current architecture summary

Aether uses the following trusted boundary:

```text
React UI
    ↓
hooks / Zustand stores
    ↓
typed TypeScript invoke wrappers
    ↓
Tauri IPC
    ↓
Rust commands
    ↓
repositories / native services
    ↓
SQLite / filesystem / Windows APIs / approved providers
```

Core principles:

* local-first;
* no frontend SQL;
* SQLite owned by Rust;
* credentials remain behind the native trust boundary;
* user data is sensitive by default;
* AI context is explicit and bounded;
* Spaces isolate context by default;
* destructive or external mutations use bounded approval models where appropriate;
* core behavior remains usable without AI;
* deterministic product behavior remains deterministic.

Full binding architecture lives in:

```text
.ai/ARCHITECTURE.md
docs/decisions/
```

---

# Decision index

Full rationale belongs in `.ai/ARCHITECTURE.md` or dedicated ADR files.

| ID        | Decision                                                                                                                      | Status                  |
| --------- | ----------------------------------------------------------------------------------------------------------------------------- | ----------------------- |
| `ADR-001` | Tauri 2 is the Windows desktop shell.                                                                                         | Accepted                |
| `ADR-002` | SQLite via bundled `rusqlite` is the local persistence layer.                                                                 | Accepted                |
| `ADR-003` | Frontend database access uses typed Tauri invoke wrappers.                                                                    | Accepted                |
| `ADR-004` | Historical multi-agent task-routing approach.                                                                                 | Superseded by `ADR-008` |
| `ADR-005` | `.ai/` documents are the engineering state source of truth.                                                                   | Accepted                |
| `ADR-007` | Verified task commits are automatically pushed/published through guarded repository tooling.                                  | Accepted                |
| `ADR-008` | Codex is the engineering agent; complex work uses explicit task contracts and evidence-based self-review.                     | Accepted                |
| `ADR-009` | Tasks use one persistent tree with nullable Space ownership and local-date due semantics.                                     | Accepted                |
| `ADR-010` | Vault distinguishes linked and managed ownership and enforces deletion safety in Rust.                                        | Accepted                |
| `ADR-011` | AI uses cancellable typed streams and explicit Space-isolated context resolved in Rust.                                       | Accepted                |
| `ADR-012` | Memory is explicit, user-authored, scoped, attributable, and attached to AI only by choice.                                   | Accepted                |
| `ADR-013` | Native Windows lifecycle uses tray persistence, non-fatal shortcut registration, OS notifications, and gated trusted updates. | Accepted                |
| `ADR-014` | Workspace export uses a sanitized and integrity-checked SQLite snapshot while excluding credentials.                          | Accepted                |
| `ADR-015` | Aether uses an internal semantic interface system with a distinctive shell and reusable primitives.                           | Accepted                |
| `ADR-016` | Sources require explicit authorization and bounded metadata-only indexing before file intelligence.                           | Accepted                |
| `ADR-022` | Portable archives include verified managed Vault bytes and restore through explicit approval and rollback-safe replacement.   | Accepted                |

Historical superseded ADRs remain valid historical records.

They must not be interpreted as active architecture.

---

# Engineering workflow state

Aether uses a repository-centered Codex engineering workflow.

Multiple Codex:

* chats;
* sessions;
* models;
* reviewers;
* worktrees

may operate on the repository when tasks are properly isolated.

This does NOT mean multiple agents own the same implementation simultaneously.

Rules:

* one implementation task has one clear owner;
* planned work uses `.ai/HANDOFF.md`;
* parallel work uses separate branches/worktrees;
* overlapping serialization points are coordinated or worked sequentially;
* the repository carries durable context between sessions;
* chat history is not required to resume engineering work.

The binding workflow lives in:

```text
AGENTS.md
WORKFLOW.md
```

---

# Current development direction

Repository-owned commercial-readiness work through Milestone K is already implemented or decision-ready.

However, external beta/commercial/launch completion depends on real-world evidence and owner decisions.

That does not prevent Aether from continuing to evolve as a personal product.

The next internally approved product milestone should be defined through:

```text
.ai/TODO.md
.ai/HANDOFF.md
```

before implementation.

Potential future domains mentioned in roadmap material are not automatically active tasks.

---

# Recommended next product area

The next proposed development area is:

```text
Connected Personal Workspace
```

Potential slices include:

1. Integration Core (INT-CORE-001 complete)
2. Connections settings (INT-CONN-001 complete)
3. external calendar event model
4. MyTimetable integration
5. School Calendar
6. Brightspace integration
7. Pulse calendar/deadline integration
8. GitHub integration
9. automation integration
10. additional personal modules

This section is directional only.

It does NOT authorize implementation by itself.

Approved work must still enter `.ai/TODO.md` and, where required, `.ai/HANDOFF.md`.

---

# Reusable update template

```markdown
## Current milestone

| ID | Name | Status | Exit condition |
| --- | --- | --- | --- |
| `M-...` | <name> | `planned / active / blocked / self_review / complete` | <observable condition> |

## Active blockers

| ID | Scope | Blocker | Owner | Resolution path |
| --- | --- | --- | --- | --- |

## Active engineering risks

| ID | Severity | Summary | Tracking task |
| --- | --- | --- | --- |

## Quality snapshot

| Check | Last result | Date | Notes |
| --- | --- | --- | --- |
```

---

# Maintenance rules

Keep this file concise.

When updating:

* record only verified facts;
* do not record speculative future implementation as current state;
* remove obsolete active-state wording;
* preserve important historical decisions through ADR references rather than narrative;
* do not duplicate full task contracts;
* do not include secrets;
* do not include private personal content;
* use exact versions/dates where known;
* do not claim quality gates are current if they have not been rerun after later changes.

When a milestone completes:

```text
HANDOFF
→ verification
→ self-review
→ CHANGELOG
→ PROJECT_STATE
```

When a new task starts, this file should normally require only a small update rather than a rewrite.
