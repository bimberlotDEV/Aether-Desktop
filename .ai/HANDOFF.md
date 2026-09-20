# Codex Task Contract

> Active implementation contract for the current planned Codex task.
>
> This file contains at most one active `planned_codex` task.
>
> When no planned task is active, the contract remains in `idle` state.
>
> Completed task history belongs in `.ai/CHANGELOG.md`, not here.

---

## Contract metadata

| Field             | Value      |
| ----------------- | ---------- |
| Schema version    | 3          |
| Task ID           | `None`     |
| Status            | `idle`     |
| Owner             | `None`     |
| Last updated      | 2026-09-20 |
| Related milestone | `None`     |
| Classification    | `None`     |
| Branch / worktree | `None`     |

---

# Status model

Valid task states:

```text
idle
draft
ready
in_progress
verification
self_review
independent_review
changes_required
blocked
complete
```

Normal flow:

```text
idle
  ↓
draft
  ↓
ready
  ↓
in_progress
  ↓
verification
  ↓
self_review
  ↓
independent_review   # when required
  ↓
complete
  ↓
archive to CHANGELOG
  ↓
idle
```

Interrupt states:

```text
blocked
changes_required
```

`changes_required` returns to `in_progress` under the same Task ID.

A completed contract must not remain in this file indefinitely.

After completion:

1. record durable completion evidence in `.ai/CHANGELOG.md`;
2. update `.ai/PROJECT_STATE.md` if current verified state changed;
3. update `.ai/TODO.md`;
4. update ADR/documentation if required;
5. return this file to the `idle` template.

---

# Objective

`None — no active planned task.`

When a task becomes active, replace this section with one concise observable objective.

A good objective describes the outcome rather than the implementation mechanism.

Example:

```text
Add the Integration Core required for external provider connections while
preserving Aether's local-first architecture and existing Rust trust boundary.
```

Avoid vague objectives such as:

```text
Improve integrations.
```

---

# Context

`None`

For an active task, include only context necessary to understand why the work exists and which current repository facts matter.

Do not duplicate:

* entire roadmap documents;
* historical task logs;
* large architecture documents;
* old chat context.

Reference canonical sources instead.

Example:

```text
Relevant current state:
- Aether 0.5.0 already has SQLite repositories and typed Tauri IPC.
- No generic external integration domain currently exists.
- Pulse and Spaces must remain provider-agnostic.
- ADR-XXX defines the approved integration trust boundary.
```

---

# Success criteria

`None`

For an active task, success criteria must be observable and independently verifiable.

Use checkboxes:

```markdown
- [ ] Integration records persist locally through the Rust repository layer.
- [ ] Provider-specific payloads do not leak into generic UI domains.
- [ ] Connection status and last successful sync are queryable through typed IPC.
- [ ] Existing quality gates remain green.
```

Avoid criteria such as:

```text
- Code looks good.
- Integration is robust.
- Architecture is clean.
```

Every criterion should be capable of being mapped to implementation or verification evidence.

---

# In scope

`None`

Define the smallest approved implementation surface.

Example:

```text
- integration domain types
- SQLite migration
- Rust repository
- typed Tauri commands
- TypeScript invoke wrappers
- repository/unit tests
- required architecture documentation
```

---

# Allowed paths

`None`

For planned tasks, list bounded paths or directories whenever reasonably possible.

Example:

```text
src-tauri/src/db/migrations.rs
src-tauri/src/db/repositories/integrations.rs
src-tauri/src/commands/
src-tauri/src/lib.rs
src/lib/db/
docs/decisions/
.ai/*
```

Directories may be used when exact files do not yet exist.

Allowed paths are a scope boundary, not permission to modify every file listed.

Only change files actually required by the task.

---

# Out of scope

`None`

Explicit non-goals are mandatory for planned work.

Example:

```text
- MyTimetable provider implementation
- Brightspace provider implementation
- Google OAuth
- n8n
- Pulse redesign
- AI scheduling
- unrelated Settings refactors
```

Out-of-scope items must not be implemented merely because they are adjacent or convenient.

If an out-of-scope change becomes necessary for correctness or safety:

1. document why;
2. determine whether the task can remain safely bounded;
3. update the contract if appropriate;
4. request human input only if the resulting scope expansion is material.

---

# Architecture constraints

`None`

For an active task, list only task-relevant binding constraints.

Common Aether constraints include:

* preserve local-first behavior;
* preserve the frontend → typed IPC → Rust trust boundary;
* no frontend SQL;
* no secrets returned to React;
* provider-specific data must be normalized before generic UI use;
* do not bypass Safe Actions for mutations requiring approval;
* migrations are append-only;
* existing user data must remain accessible;
* core functionality must not require AI.

Do not copy the entire `AGENTS.md` into this contract.

Reference it instead.

---

# Dependencies

`None`

Record only real dependencies.

Examples:

```text
- ADR-027 must be accepted before implementation.
- Existing Task repository API is reused.
- Requires no external account or credential.
```

Dependencies may be:

* repository features;
* architecture decisions;
* other task IDs;
* external access;
* owner decisions.

Do not list hypothetical future dependencies.

---

# Risks and safeguards

`None`

For active work, identify meaningful task-specific risks.

Format:

```markdown
- **Duplicate external records**
  - Safeguard: provider + external ID uniqueness and idempotent upsert tests.

- **Credential leakage**
  - Safeguard: credentials remain native-only and never cross IPC.

- **Migration regression**
  - Safeguard: fresh-create and upgrade-path tests.
```

Do not pad this section with generic software risks.

---

# Rollback considerations

`None`

Describe rollback only where meaningful.

Examples:

```text
- Code changes are reversible through the task branch.
- The migration is append-only and therefore cannot be deleted after release.
- New records remain ignored safely by earlier UI paths.
```

For migrations, data ownership, external writes, destructive behavior, updater changes, or security changes, this section must be explicit.

---

# Required validation

`None`

List the exact commands or checks required for this task.

Use the smallest sufficient set during iteration and the appropriate complete gate before completion.

Common frontend validation:

```text
pnpm typecheck
pnpm lint
pnpm test
pnpm build
```

Rust/Tauri validation:

```text
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
```

Additional task-specific validation may include:

```text
git diff --check
fresh database migration test
upgrade migration test
duplicate-sync test
disconnect/reconnect test
manual desktop smoke test
dark/light/system-theme verification
keyboard/focus verification
exact-head GitHub CI
```

Do not list validation that will not realistically be executed.

Do not claim a check passed unless it actually ran successfully.

---

# Independent review requirement

| Field          | Value  |
| -------------- | ------ |
| Required       | `No`   |
| Reason         | `None` |
| Reviewer scope | `None` |

Independent review should normally be required for:

* major architecture changes;
* security-sensitive changes;
* migrations with meaningful data implications;
* OAuth/authentication;
* external integrations with write capabilities;
* Safe Actions changes;
* backup/restore;
* updater/signing logic;
* broad cross-layer AI execution changes.

Independent review is not required merely because another model is available.

When enabled, the reviewer should inspect the completed diff rather than reimplementing the task.

---

# Human decisions required

`None`

Only list decisions that cannot safely be inferred.

Examples:

```text
- OAuth client registration requires owner credentials.
- Product must choose whether calendar sync is read-only or two-way.
```

Do not use this section for routine technical decisions.

If no human input is required:

```text
None
```

---

# Blocking decisions

`None`

A task is `blocked` only when safe progress genuinely cannot continue.

For every blocker record:

```text
Blocker:
Evidence:
Impact:
Safe alternatives attempted:
Smallest human decision/action required:
```

Difficulty, a failed test, or a long-running implementation is not automatically a blocker.

---

# Worktree / ownership gate

| Check                             | State |
| --------------------------------- | ----- |
| Correct branch/worktree confirmed | `N/A` |
| `git status` inspected            | `N/A` |
| User-owned changes identified     | `N/A` |
| Parallel task overlap checked     | `N/A` |
| Serialization points identified   | `N/A` |

For active planned work these checks must be completed before production editing.

Serialization points include:

* migrations;
* central IPC contracts;
* global routing;
* provider registries;
* design tokens;
* database initialization;
* Safe Actions core;
* backup/restore;
* shared architecture decisions.

---

# Readiness review

Current state:

```text
Idle — no active planned task.
```

Before changing `draft` to `ready`, confirm:

* [ ] Task ID is stable.
* [ ] Objective is observable.
* [ ] Success criteria are measurable.
* [ ] In-scope areas are bounded.
* [ ] Out-of-scope behavior is explicit.
* [ ] Allowed paths are sufficiently constrained.
* [ ] Dependencies are known.
* [ ] Meaningful risks are recorded.
* [ ] Rollback considerations are understood.
* [ ] Required validation is defined.
* [ ] Required ADR is accepted or not needed.
* [ ] Human decisions are resolved or not needed.
* [ ] Worktree ownership is safe.
* [ ] Parallel task overlap is absent or coordinated.

If all apply:

```text
Status: ready
```

Otherwise remain:

```text
draft
```

or:

```text
blocked
```

---

# Implementation log

`None`

Keep this section short.

During active work it may contain concise factual checkpoints such as:

```text
2026-09-20
- Added integration migration and repository.
- Repository tests pass.
- Connections UI intentionally not started because it is out of scope.
```

Do not use this as a permanent changelog.

Only record information useful for completing or resuming the active task.

---

# Verification evidence

`None`

After implementation, map validation evidence to actual results.

Example:

```text
pnpm check
PASS — 104/104 tests

cargo test --manifest-path src-tauri/Cargo.toml
PASS — 114/114 tests

git diff --check
PASS
```

Never use guessed or inherited results from an earlier commit.

Verification must correspond to the current task head.

---

# Acceptance evidence

`None`

During self-review, map every success criterion to concrete evidence.

Use this format:

```markdown
### AC1 — Connection records persist locally

Implementation:
- `src-tauri/src/db/repositories/integrations.rs`
- migration `XXX_integrations`

Verification:
- `create_and_reload_integration`
- `update_connection_status`
```

Every success criterion must have evidence before a planned task can be marked `complete`.

---

# Self-review

Current state:

```text
Not applicable — idle.
```

For an active task, self-review must inspect:

* full changed-path list;
* final diff;
* acceptance criteria;
* architecture compliance;
* security/privacy implications;
* scope compliance;
* error handling;
* rollback implications;
* tests;
* staged files;
* accidental secrets;
* unrelated changes.

Outcome must be one of:

```text
complete
changes_required
blocked
```

Material findings must be recorded.

If corrections are made after self-review, rerun the affected checks.

---

# Independent review findings

`None`

When independent review is required, record only meaningful findings.

Format:

```markdown
### Blocking

- None

### Important

- `<finding>`

### Optional

- `<finding>`
```

Accepted blocking findings must be resolved before task completion.

Optional stylistic preferences do not block completion.

---

# Completion evidence

`None`

Once complete, briefly record:

* observable behavior delivered;
* major changed areas;
* validation results;
* acceptance evidence;
* architecture implications;
* known limitations;
* follow-up work intentionally excluded.

This section is temporary.

Its durable result must move into `.ai/CHANGELOG.md` before this contract returns to `idle`.

---

# Publication state

| Field         | Value  |
| ------------- | ------ |
| Commit        | `None` |
| Remote branch | `None` |
| Draft PR      | `None` |
| Exact-head CI | `None` |

Publication is required only for publishable implementation work.

Review-only, investigation-only, planning-only, or local experiment tasks do not need artificial commits or PRs.

---

# Closure checklist

Before setting a planned task to `complete`:

* [ ] Objective satisfied.
* [ ] All success criteria satisfied.
* [ ] Scope remained bounded.
* [ ] Required validation passed.
* [ ] Acceptance evidence recorded.
* [ ] Self-review completed.
* [ ] Blocking independent-review findings resolved, if applicable.
* [ ] Relevant documentation updated.
* [ ] Relevant ADR updated/created, if applicable.
* [ ] No unresolved security issue remains.
* [ ] No unresolved data-safety issue remains.
* [ ] No unrelated changes are staged.
* [ ] Publishable work committed and pushed.
* [ ] Draft PR exists when required.
* [ ] Exact-head CI passed when required.
* [ ] `.ai/CHANGELOG.md` updated.
* [ ] `.ai/PROJECT_STATE.md` updated if current state changed.
* [ ] `.ai/TODO.md` updated.
* [ ] `.ai/SESSION_NOTES.md` updated or cleared appropriately.

After closure, reset this file to the idle contract.

---

# Next task

`None`

The next planned task must come from the approved roadmap/backlog and receive its own Task ID and bounded contract.

Do not automatically begin adjacent work after completing the current task.
