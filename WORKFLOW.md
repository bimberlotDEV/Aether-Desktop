# Aether Codex Development Workflow

This document defines the engineering workflow for **Aether**.

Aether uses a repository-centered Codex workflow.

Codex may operate through different threads, sessions, models, reviewers, or isolated Git worktrees, but the repository and its control documents remain the shared source of truth.

Codex owns repository analysis, planning, architecture, implementation, verification, self-review, documentation, and GitHub publication within the boundaries of the approved task.

The human owner supplies product direction and approval only when a decision cannot be safely inferred or would be destructive, irreversible, materially product-shaping, externally coordinated, or expand authorized scope.

---

# 1. Operating principles

1. One canonical source exists for each kind of project information.
2. The repository is authoritative over chat history.
3. Every implementation task is classified as `direct_codex` or `planned_codex`.
4. Every task has a bounded goal, explicit scope, validation requirements, and a stop condition.
5. Codex may make reversible technical decisions that follow existing constraints.
6. Durable architectural decisions are recorded in ADRs.
7. Complex or risky work requires a written task contract before production code changes.
8. Verification evidence and explicit self-review are part of implementation.
9. Failed checks, risks, assumptions, deviations, and known limitations are never hidden.
10. Documentation changes ship with the behavior they describe.
11. Unrelated issues discovered during a task do not automatically expand task scope.
12. Parallel work is allowed only when ownership and file boundaries are sufficiently independent.
13. Completed publishable implementation work is committed, pushed, and represented by a GitHub pull request.
14. Review, investigation, planning, and experiments do not require artificial implementation commits.
15. Codex should inspect the smallest repository scope necessary to perform the task safely.

---

# 2. Source-of-truth map

| Question                                    | Canonical source                                | Update timing                                 |
| ------------------------------------------- | ----------------------------------------------- | --------------------------------------------- |
| What is true now?                           | `.ai/PROJECT_STATE.md` + current implementation | After a verified change, decision, or blocker |
| What should be done next?                   | `.ai/TODO.md`                                   | During prioritization and task closure        |
| What may be implemented for a planned task? | `.ai/HANDOFF.md`                                | Before and during `planned_codex` work        |
| What architecture is binding?               | `.ai/ARCHITECTURE.md` + ADRs                    | When a durable decision changes               |
| What was completed?                         | `.ai/CHANGELOG.md`                              | After verification and self-review            |
| What helps resume active work?              | `.ai/SESSION_NOTES.md`                          | During active work; refresh next session      |
| What is the engineering process?            | `WORKFLOW.md`                                   | Rarely, after an explicit process decision    |
| What rules always apply?                    | `AGENTS.md`                                     | When durable repository rules change          |
| What is the long-term product direction?    | `IDEA.md`                                       | Planning context only                         |

`IDEA.md` is not implementation authorization.

Chat history is not a canonical engineering record.

If chat context conflicts with current repository state, investigate the repository and control documents before acting.

---

# 3. Context discipline

Codex must avoid unnecessary context loading.

At the start of a task:

1. Read the required control documents.
2. Identify the active task.
3. Identify the relevant domains and ownership boundaries.
4. Search for the smallest relevant implementation surface.
5. Read only relevant source files, tests, ADRs, and documentation.

Do NOT:

* scan the entire repository by default;
* read every ADR;
* read all historical changelog entries;
* load unrelated roadmap documents;
* inspect unrelated modules;
* repeatedly re-read large files without need;
* treat old chat history as the primary source of truth.

Prefer:

* targeted repository search;
* symbol search;
* dependency tracing;
* existing tests;
* relevant ADRs;
* current implementation patterns.

Broader repository inspection is appropriate only when the task genuinely crosses those boundaries.

---

# 4. Task classification

Every implementation task must be classified before production code changes begin.

| Route           | Use when                                                                                                 | Task contract                | Review                            |
| --------------- | -------------------------------------------------------------------------------------------------------- | ---------------------------- | --------------------------------- |
| `direct_codex`  | Small, bounded, reversible, low-risk work with no unresolved design decision                             | Not required                 | Proportionate self-check          |
| `planned_codex` | Features, architecture, security, migrations, integrations, cross-layer changes, ambiguity, or high risk | Required in `.ai/HANDOFF.md` | Formal evidence-based self-review |

---

## 4.1 `direct_codex`

Use `direct_codex` only when all relevant conditions are true:

* The requested outcome is clear.
* Existing repository conventions determine the approach.
* No unresolved security decision exists.
* No unresolved privacy decision exists.
* No migration or data-lifecycle decision exists.
* No meaningful architecture change exists.
* The change is small enough to implement and verify in one focused cycle.
* The work is reversible.
* The work does not materially alter a shared or external interface.

Examples:

* focused UI fixes;
* copy corrections;
* isolated styling fixes;
* straightforward bug fixes;
* focused tests;
* small refactors governed entirely by existing architecture.

A direct task follows this cycle:

```text
inspect
→ implement
→ verify
→ self-check
→ update durable state if necessary
→ publish if applicable
```

---

## 4.2 `planned_codex`

Use `planned_codex` when any of the following apply:

* A new feature is introduced.
* A domain changes.
* An architecture boundary changes.
* A database migration is required.
* A shared interface changes.
* Security or privacy is involved.
* Credentials or OAuth are involved.
* External data access is involved.
* Safe Actions are involved.
* The task crosses multiple layers.
* Several ownership boundaries are affected.
* Requirements contain material ambiguity.
* Failure could cause data loss.
* Failure could create security exposure.
* Rollback would be difficult.
* The task introduces a durable dependency or architectural decision.
* The task implements a new external integration.

A planned task requires a bounded contract in:

```text
.ai/HANDOFF.md
```

The contract must reach:

```text
status: ready
```

before production implementation begins.

---

# 5. Human approval gate

Codex asks the human owner only when completion requires a decision or action that cannot be safely inferred from repository evidence and approved scope.

Human approval is required for:

* irreversible destructive behavior not already authorized;
* materially different product outcomes;
* meaningful product-scope expansion;
* external spending;
* new credentials or access grants;
* public publication beyond the established repository workflow;
* external organizational coordination;
* conflicting acceptance criteria that repository evidence cannot resolve;
* changes that materially alter the product's long-term direction.

Ordinary:

* architecture;
* implementation;
* testing;
* debugging;
* refactoring;
* documentation;
* commit;
* push;
* draft-PR work

remain Codex-owned when they fit the approved task and repository workflow.

Do not ask the human for routine technical decisions that can safely be inferred.

---

# 6. Mandatory session startup

At the start of an implementation session, Codex reads the following control documents in this order:

1. `AGENTS.md`
2. `.ai/PROJECT_STATE.md`
3. `.ai/HANDOFF.md`
4. `.ai/ARCHITECTURE.md`
5. `.ai/TODO.md`
6. `.ai/SESSION_NOTES.md`
7. the newest relevant `.ai/CHANGELOG.md` entry

This does NOT mean Codex must read the entirety of each large historical file.

Read only the relevant current section where possible.

After control-document bootstrap:

1. inspect Git status;
2. identify current branch/worktree;
3. verify whether user-owned changes exist;
4. identify relevant open PR state if applicable;
5. identify the active task and ownership boundaries;
6. inspect only relevant implementation paths.

Do not begin production changes before confirming task ownership.

---

# 7. Task definition

Before implementation, every task must establish:

* **Task ID**
* **Classification**
* **Goal**
* **Context**
* **Success criteria**
* **In-scope areas**
* **Out-of-scope areas**
* **Dependencies**
* **Risks**
* **Validation**
* **Rollback considerations when applicable**
* **Stop condition**

A useful task shape is:

```text
GOAL
Deliver one observable outcome.

SUCCESS CRITERIA
- observable result
- observable result
- observable result

IN SCOPE
- bounded domain
- bounded files/layers

OUT OF SCOPE
- adjacent feature
- unrelated refactor
- future integration

VALIDATION
- required tests/checks

STOP CONDITION
Stop when the approved success criteria are satisfied.
```

`OUT OF SCOPE` is binding.

---

# 8. Planned-task readiness gate

A `planned_codex` task may move from `draft` to `ready` only when all required items are present:

* stable task ID;
* objective;
* relevant context;
* classification;
* observable acceptance criteria;
* bounded allowed paths or domains;
* explicit out-of-scope behavior;
* dependencies;
* known risks;
* rollback considerations where applicable;
* blocking decisions;
* validation commands;
* relevant ADR when a durable architecture boundary changes;
* confirmation that user-owned or parallel worktree changes do not conflict.

If a required item cannot be safely inferred, Codex sets the task to:

```text
blocked
```

and asks the human owner only for the smallest necessary decision.

---

# 9. Delivery cycle

## 9.1 Analyse

Codex:

1. verifies repository facts;
2. separates evidence from assumptions;
3. identifies relevant ownership boundaries;
4. chooses `direct_codex` or `planned_codex`;
5. determines the minimum implementation surface;
6. checks whether an ADR is required;
7. confirms scope and validation.

For planned work, update the required control documents before implementation.

---

## 9.2 Plan

For a planned task, Codex updates as needed:

* `.ai/HANDOFF.md`
* `.ai/TODO.md`
* `.ai/PROJECT_STATE.md`
* `.ai/ARCHITECTURE.md`
* relevant ADR
* `.ai/SESSION_NOTES.md`

Planning should be proportional.

Do not turn a bounded feature into a speculative multi-phase redesign.

---

## 9.3 Implement

Codex:

1. sets a planned task to `in_progress`;
2. reads only relevant implementation paths and dependencies;
3. makes the smallest cohesive change that satisfies the task;
4. preserves unrelated user changes;
5. preserves unrelated worktree changes;
6. follows existing architecture;
7. adds tests closest to the behavior owner;
8. avoids opportunistic refactors;
9. avoids speculative abstractions;
10. avoids unrequested adjacent features.

If an unrelated issue is discovered:

* document it;
* optionally add it to `.ai/TODO.md`;
* continue the current task;
* do not fix it unless required for correctness or safety.

---

## 9.4 Verify

Use task-specific verification during iteration.

Default frontend gate:

```bash
pnpm typecheck
pnpm lint
pnpm test
pnpm build
```

For Rust/Tauri changes:

```bash
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
```

For migrations, verify:

* fresh database creation;
* upgrade from the immediately relevant prior schema;
* migration ordering;
* repository behavior;
* foreign-key integrity;
* backup/restore compatibility where relevant.

For UI work, verify where applicable:

* supported desktop window sizes;
* keyboard behavior;
* focus behavior;
* dark theme;
* light theme;
* system theme;
* meaningful empty states;
* loading/error states.

For integrations, verify:

* initial synchronization;
* repeated synchronization;
* duplicate handling;
* update handling;
* transient failure behavior;
* invalid credentials/token state where applicable;
* provider unavailable behavior;
* local data integrity after failed sync.

An in-scope failure must be fixed.

A genuinely pre-existing or external failure must be recorded with:

* exact command;
* exact failure;
* expected impact;
* tracking reference if available.

Do not hide failures.

---

# 10. Self-review

Implementation completion does not equal task acceptance.

After implementation, Codex performs a distinct review pass.

The self-review must:

1. inspect the complete changed-path list;
2. inspect the final diff;
3. map every acceptance criterion to evidence;
4. verify scope discipline;
5. verify architecture compliance;
6. verify security boundaries;
7. inspect relevant error paths;
8. inspect rollback implications where applicable;
9. inspect destructive behavior;
10. confirm no secrets are staged;
11. confirm no unrelated changes are staged;
12. verify test evidence;
13. rerun affected validation after corrections.

Possible review outcomes:

```text
complete
changes_required
blocked
```

`changes_required` returns to implementation under the same task ID.

Material findings and their resolution should be recorded for planned tasks.

---

# 11. Independent review

A separate Codex session or model may review completed work.

Independent review is encouraged for:

* architecture changes;
* security-sensitive work;
* migrations;
* OAuth;
* external integrations;
* Safe Actions;
* AI tool execution;
* backup/restore;
* broad cross-layer changes.

The reviewer:

* reviews the diff;
* checks relevant surrounding architecture;
* checks acceptance criteria;
* checks security;
* checks maintainability;
* checks tests;
* checks scope compliance.

The reviewer must NOT:

* rewrite working code merely because it prefers another style;
* broaden the task;
* perform unrelated cleanup;
* turn a review into a redesign exercise.

Review findings should be classified where useful as:

```text
blocking
important
optional
```

Prefer the smallest safe correction.

---

# 12. Builder/reviewer workflow

When useful, Aether may use a builder/reviewer sequence:

```text
architecture/task definition
        ↓
implementation session
        ↓
self-review
        ↓
independent review
        ↓
targeted fixes
        ↓
final verification
        ↓
publish
```

The reviewer does not need to reproduce the builder's full analysis.

The builder fixing review findings should fix only accepted findings unless new correctness or safety issues are discovered.

---

# 13. Parallel sessions and worktrees

Multiple Codex sessions may work on Aether concurrently only when ownership is clearly separated.

Each active implementation worktree must have:

* its own branch;
* one clearly defined task;
* one owner;
* bounded file/domain ownership;
* independent acceptance criteria.

Good parallelism:

```text
worktree A → Connections UI
worktree B → Calendar visual components
worktree C → documentation/tests
```

Poor parallelism:

```text
worktree A → integration migration
worktree B → same migration
worktree C → shared integration repository architecture
```

Parallel work must not independently modify the same serialization point without coordination.

---

# 14. Serialization points

Treat the following as conflict-prone serialization points:

* migrations;
* global types;
* application routing;
* native application state;
* central IPC contracts;
* design tokens;
* provider registries;
* shared integration abstractions;
* Safe Actions core;
* database initialization;
* backup/restore infrastructure;
* global settings infrastructure;
* architecture documentation for the same decision.

Work touching a serialization point should normally be coordinated or sequential.

---

# 15. Task design and scaling

One task should deliver one coherent, independently verifiable outcome.

Large epics must be decomposed before implementation.

Prefer vertical slices.

Example:

```text
Integration epic

1. Integration domain model
2. Persistence
3. Connection status
4. Connections UI
5. MyTimetable connector
6. Calendar read model
7. Pulse integration
8. Brightspace connector
```

Do not implement the entire epic in one task unless there is a strong architectural reason.

Each slice should be testable and reviewable.

---

# 16. Scope control

Scope expansion is not automatic.

If Codex identifies useful adjacent work:

1. determine whether it is required for the current acceptance criteria;
2. if required, document why;
3. if not required, record it as follow-up work;
4. do not implement it during the current task.

Examples:

While implementing MyTimetable:

Do not automatically add:

* Brightspace;
* Google Calendar;
* Gmail;
* n8n;
* Pulse redesign;
* finance integration.

While implementing Connections UI:

Do not automatically add:

* OAuth;
* provider-specific APIs;
* unrelated Settings redesign.

---

# 17. Blocking rules

Codex enters `blocked` only when:

* a required human product decision is missing;
* safe completion exceeds authorized scope;
* user-owned changes overlap required edits;
* parallel task ownership conflicts;
* an unauthorized destructive or data-loss risk appears;
* required credentials or external access are unavailable;
* acceptance criteria materially conflict;
* required infrastructure does not exist and cannot safely be created in scope.

Difficulty is not itself a blocker.

Long runtime is not itself a blocker.

An initially failed check is not itself a blocker.

Codex first exhausts safe in-scope diagnostics and alternatives.

---

# 18. Update project memory

After successful implementation and self-review, update durable state only where necessary.

Recommended order:

1. `.ai/CHANGELOG.md`
2. `.ai/PROJECT_STATE.md`
3. `.ai/HANDOFF.md`
4. `.ai/SESSION_NOTES.md`
5. `.ai/TODO.md`

For completed planned tasks:

* durable results move to `CHANGELOG`;
* project state reflects the new verified state;
* completed task details leave the active handoff;
* `.ai/HANDOFF.md` returns to the idle template when no planned task is active.

Do not duplicate the same information unnecessarily across every control document.

---

# 19. Session notes

`.ai/SESSION_NOTES.md` exists to resume active work efficiently.

Keep it concise.

It should record:

* active task ID;
* current branch/worktree;
* exact current state;
* last verified action;
* unresolved issue if any;
* next concrete action.

Do not turn session notes into a permanent historical log.

Durable history belongs in:

```text
.ai/CHANGELOG.md
```

---

# 20. Documentation hygiene

* Use ISO dates.
* Use stable task IDs.
* Use stable decision IDs.
* Use `None` for intentionally empty required fields.
* Clearly label assumptions.
* Replace assumptions with evidence when available.
* Never store secrets in `.ai/`.
* Never store personal user data in `.ai/`.
* Keep `SESSION_NOTES.md` temporary.
* Keep `CHANGELOG.md` append-only and newest-first except factual corrections.
* Update architecture documentation only when architecture actually changes.
* Update state in the same PR as the behavior it describes.
* Do not document future functionality as if it already exists.

---

# 21. GitHub publication

Publishable implementation work must be published at a verified task boundary.

Standard flow:

1. Start from current `master`.
2. Create an appropriate task branch.
3. Implement the task.
4. Run required verification.
5. Complete self-review.
6. Resolve blocking review findings.
7. Stage only task-owned paths.
8. Commit with a conventional commit.
9. Push.
10. Create or update the draft PR.
11. Confirm remote branch and PR state.

Preferred branch naming:

```text
agent/<short-description>
```

Examples:

```text
agent/integration-core
agent/mytimetable-sync
agent/connections-ui
agent/pulse-calendar
```

---

# 22. Publication exceptions

The following task types do not require a production implementation commit or PR unless they intentionally change repository files:

* review-only;
* investigation-only;
* planning-only;
* architecture analysis without changes;
* debugging investigation;
* local experiment;
* feasibility research.

Do not create meaningless commits simply to satisfy publication mechanics.

---

# 23. Automatic GitHub publication

For publishable implementation tasks:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/publish-task.ps1 `
  -Message "feat(scope): description" `
  -Paths <task-owned-paths>
```

One-time hook setup:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/install-git-hooks.ps1
```

The versioned post-commit hook may automatically push ordinary commits.

Never publish:

* `.env*`;
* databases;
* private keys;
* credentials;
* tokens;
* actual secret material;
* personal data;
* backups;
* unrelated user changes;
* knowingly failing in-scope implementation represented as complete.

GitHub publication does not imply human approval.

A draft PR is not automatically merged.

---

# 24. Branch/worktree safety

Before editing:

* confirm branch;
* confirm worktree;
* inspect `git status`;
* preserve unrelated modifications;
* identify user-owned changes;
* identify parallel-task ownership.

Never:

* reset user-owned changes;
* force-push without explicit authorization;
* rewrite unrelated history;
* stage unrelated paths;
* silently absorb another task's work into the current task.

If ownership is ambiguous, stop editing the overlapping files until ownership is resolved.

---

# 25. Database and migration workflow

Database changes are always `planned_codex`.

Before migration implementation:

1. inspect current schema;
2. inspect relevant repositories;
3. determine data-lifecycle implications;
4. determine upgrade behavior;
5. determine rollback implications;
6. update the task contract;
7. create/update ADR if architecture changes.

Migrations are append-only.

Never modify an already-applied migration to change production history.

Migration verification should cover:

* fresh creation;
* upgrade;
* constraints;
* relationships;
* repository behavior;
* failure behavior;
* compatibility with backup/restore where relevant.

Migrations are serialization points and should not be developed concurrently without coordination.

---

# 26. Integration workflow

New external integrations are `planned_codex`.

Before implementing a provider integration, define:

* provider;
* authentication method;
* requested permissions/scopes;
* normalized Aether domain model;
* sync direction;
* polling/webhook strategy;
* duplicate semantics;
* update semantics;
* deletion semantics;
* error states;
* token storage;
* disconnect behavior;
* last-sync behavior;
* retry behavior;
* offline behavior;
* privacy implications.

Provider-specific API models must not leak unnecessarily into generic presentation layers.

Prefer:

```text
Provider API
→ connector/service
→ normalized Aether model
→ repository
→ IPC
→ UI
```

---

# 27. Safe Actions workflow

Any new mutation initiated through AI or automation must determine whether Safe Actions applies.

Preferred pattern:

```text
AI/tool proposes
→ Rust validates
→ user previews consequence
→ user approves
→ bounded operation executes
→ Activity records result
```

The model may not:

* approve its own mutation;
* silently replace approved arguments;
* gain arbitrary shell access;
* bypass established permission boundaries.

Read-only operations may use lighter paths only when clearly bounded and safe.

---

# 28. AI-related work

AI features must preserve Aether's product principle:

> Aether must remain usable and understandable without AI.

AI-related tasks should distinguish between:

* deterministic application behavior;
* model reasoning;
* provider transport;
* context selection;
* permission boundaries;
* user approval;
* persisted provenance.

Do not replace deterministic logic such as:

* task state;
* deadline calculation;
* permissions;
* audit records;
* sync state;
* destructive-action rules

with model-generated behavior.

---

# 29. Review evidence

For planned tasks, the completion evidence should include:

```text
Acceptance Criterion
→ implementation evidence
→ test/verification evidence
```

Example:

```text
AC1: Duplicate external events are prevented.

Evidence:
- unique provider/external ID constraint
- repository upsert path
- repeated-sync Rust test
```

Do not use vague evidence such as:

```text
"implemented correctly"
```

---

# 30. Completion report

At the end of an implementation task, provide a concise engineering summary containing:

### Behavior

What changed for the user or system.

### Major changed areas

Relevant files/modules/domains.

### Validation

Commands actually executed and their result.

### Acceptance criteria

Evidence that each criterion was satisfied.

### Architecture

Any meaningful architectural consequence.

### Known limitations

Anything intentionally unsupported.

### Follow-up

Useful work discovered but intentionally left out of scope.

Do not pad the completion report with unnecessary narrative.

---

# 31. Definition of done

A publishable implementation task is complete only when:

* the requested outcome is implemented;
* every acceptance criterion is satisfied;
* scope remained bounded;
* required tests/checks pass;
* relevant documentation is current;
* architecture boundaries remain valid;
* security/privacy requirements remain valid;
* self-review is complete;
* blocking independent review findings are resolved when independent review was required;
* no unresolved data-safety issue remains;
* no unresolved architecture issue remains;
* task-owned changes are committed;
* the branch is pushed;
* a draft PR exists when working off `master`.

A documented pre-existing or authorized external limitation may remain if it does not invalidate the task's acceptance criteria.

---

# 32. State model

Planned task:

```text
TODO candidate
    ↓
planned
    ↓
HANDOFF draft
    ↓
ready
    ↓
in_progress
    ↓
verification
    ↓
self_review
    ↓
independent_review (when required)
    ↓
complete
    ↓
CHANGELOG + PROJECT_STATE
```

Interrupt states:

```text
blocked
changes_required
```

`changes_required` returns to `in_progress` under the same task ID.

`blocked` must contain:

* evidence;
* impact;
* attempted safe alternatives;
* one concrete request to the human owner.

---

# 33. Recommended multi-session workflow

Aether may use specialized Codex sessions such as:

```text
Aether 00 — Architecture & Roadmap
Aether 01 — Integration Core
Aether 02 — School & Calendar
Aether 03 — Pulse & UX
Aether 04 — n8n & Automations
Aether 05 — GitHub / Development
Aether 06 — AI Tools & Safe Actions
Aether 99 — QA / Bugs / Releases
```

These session names are organizational conventions, not architecture.

The repository remains the shared memory between them.

A new session should not require re-explaining the entire project.

It should bootstrap from:

```text
AGENTS.md
WORKFLOW.md
.ai/
relevant architecture
relevant source
```

---

# 34. Model-independent workflow

Repository rules must not depend on a particular AI model.

Different models may be chosen based on task difficulty, cost, speed, or review needs.

The same:

* task contract;
* architecture rules;
* scope rules;
* validation standards;
* security rules;
* publication standards

apply regardless of model.

Do not encode model-specific behavior into architecture or repository conventions.

---

# 35. Final engineering principle

Aether should evolve through small, verified, reviewable improvements.

Prefer:

```text
understand
→ bound
→ implement
→ verify
→ review
→ document
→ publish
```

over:

```text
inspect everything
→ redesign everything
→ implement everything
→ hope tests pass
```

The objective is not maximum code generation.

The objective is durable product progress with minimal unnecessary scope, context, risk, and technical debt.
