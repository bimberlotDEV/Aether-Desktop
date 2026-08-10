# Aether Codex Development Workflow

This document defines the single-agent engineering workflow for Aether. Codex owns analysis, planning, architecture, implementation, verification, self-review, documentation, and GitHub publication. The human owner supplies product direction and approval only when a decision cannot be safely inferred or would be destructive, irreversible, or materially expand scope.

## Operating principles

1. One canonical source exists for each kind of project information.
2. Every task is classified as `direct_codex` or `planned_codex` before implementation.
3. Codex may make reversible technical decisions that follow existing constraints and records durable decisions in an ADR.
4. Complex or risky work requires a written task contract before code changes.
5. Verification evidence and explicit self-review are part of implementation.
6. Failed checks, risks, assumptions, and deviations are never hidden.
7. Documentation changes ship with the behavior they describe.
8. Completed work is committed, pushed, and represented by a GitHub pull request.

## Task classification

| Route | Use when | Task contract | Review |
| --- | --- | --- | --- |
| `direct_codex` | Small, bounded, low-risk work with no unresolved design decision | Not required | Proportionate Codex self-check |
| `planned_codex` | Features, architecture, security, migrations, cross-layer changes, ambiguity, or high risk | Required in `.ai/HANDOFF.md` | Formal Codex self-review against acceptance criteria |

### `direct_codex`

Use this route only when all are true:

- The requested outcome is clear.
- Existing repository conventions determine the approach.
- No unresolved security, privacy, migration, data-lifecycle, or product decision exists.
- The change is small enough to implement and verify in one focused cycle.
- The work is reversible and does not materially change a public interface.

Codex reads the control documents, inspects the smallest relevant scope, implements, verifies, updates durable state, self-checks the diff, and publishes.

### `planned_codex`

Use this route when any are true:

- A feature, domain, architecture boundary, migration, or shared interface changes.
- Security, credentials, privacy, destructive behavior, or external data access is involved.
- The task crosses layers or several ownership boundaries.
- Requirements contain material trade-offs or ambiguity.
- Failure could cause data loss, security exposure, or difficult rollback.

Codex first analyses the repository and writes a bounded task contract in `.ai/HANDOFF.md`. The contract must reach `ready` before implementation begins. After implementation and verification, Codex performs a separate self-review pass and records its evidence before marking the task `complete`.

### Human approval gate

Codex asks the human owner only when completion requires:

- An irreversible or destructive action not already authorized.
- A product choice with materially different user outcomes.
- New external credentials, spending, publication, or coordination authority.
- A scope expansion beyond the request.
- Resolution of conflicting acceptance criteria that repository evidence cannot settle.

Ordinary architecture, implementation, testing, refactoring, documentation, commit, push, and draft-PR work remains Codex-owned.

## Codex responsibilities

Codex owns the complete engineering cycle:

- Repository analysis and current-state verification.
- Prioritization within the user-approved roadmap.
- Architecture, ADRs, interfaces, and trade-off documentation.
- Breaking epics into bounded, independently verifiable tasks.
- Implementation, debugging, refactoring, migrations, and performance work.
- Automated tests, static checks, builds, and runtime verification.
- Security and scope review of its own diff after implementation.
- Maintenance of all `.ai/` control documents.
- Conventional commits, pushes, and draft pull requests.
- Clear escalation to the human owner when the approval gate applies.

Self-review must be evidence-based. Codex re-reads the task contract, inspects the final diff, maps each acceptance criterion to evidence, checks scope and unsafe/destructive behavior, and reruns the relevant gates. It must not treat implementation completion as automatic acceptance.

## Source-of-truth map

| Question | Canonical file | Update timing |
| --- | --- | --- |
| What is true now? | `.ai/PROJECT_STATE.md` | After a verified change, decision, or blocker |
| What should be done next? | `.ai/TODO.md` | During prioritization and task closure |
| What may be implemented for a planned task? | `.ai/HANDOFF.md` | Before and during `planned_codex` work |
| What architecture is binding? | `.ai/ARCHITECTURE.md` and ADRs | When a durable decision changes |
| What was completed? | `.ai/CHANGELOG.md` | After verification and self-review |
| What helps resume the current session? | `.ai/SESSION_NOTES.md` | During work; replace next session |
| What is the engineering process? | `WORKFLOW.md` | Rarely, with an explicit process decision |
| What rules always apply? | `AGENTS.md` | When durable repository rules change |

## Mandatory session startup

Codex reads these files in order before inspecting implementation code:

1. `AGENTS.md`
2. `.ai/PROJECT_STATE.md`
3. `.ai/HANDOFF.md`
4. `.ai/ARCHITECTURE.md`
5. `.ai/TODO.md`
6. `.ai/SESSION_NOTES.md`
7. The newest relevant `.ai/CHANGELOG.md` entry

Then Codex checks the Git worktree, current branch, remote synchronization, and relevant open pull request.

## Planned-task readiness gate

A `planned_codex` task may move from `draft` to `ready` only when all are present:

- Stable task ID, objective, context, and classification.
- Observable acceptance criteria.
- Explicit allowed paths or bounded directories.
- Out-of-scope behavior and non-goals.
- Dependencies, risks, rollback considerations, and blocking decisions.
- Required validation commands.
- Accepted ADR when the task changes a durable architecture boundary.
- Confirmation that user-owned worktree changes do not conflict.

If any item cannot be safely inferred, Codex marks the contract `blocked` and asks the human for the smallest necessary decision.

## Delivery cycle

### 1. Analyse and plan

Codex verifies repository facts, separates assumptions from evidence, selects a route, and for `planned_codex` work updates:

- `.ai/ARCHITECTURE.md` and an ADR when architecture changes.
- `.ai/TODO.md` with priority, dependencies, and task status.
- `.ai/PROJECT_STATE.md` with the active milestone or blocker.
- `.ai/HANDOFF.md` with a bounded task contract.
- `.ai/SESSION_NOTES.md` with the exact resume point.

### 2. Implement

Codex:

1. Sets a planned task to `in_progress`.
2. Reads only relevant implementation paths and dependencies.
3. Makes the smallest cohesive change satisfying the contract.
4. Preserves unrelated user changes.
5. Adds tests closest to the behavior owner.
6. Avoids opportunistic scope expansion.

### 3. Verify

Default frontend gate:

```bash
pnpm typecheck
pnpm lint
pnpm test
pnpm build
```

For Rust or Tauri changes, also run:

```bash
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
```

For migrations, verify fresh creation, upgrade from the prior schema, idempotence, and repository behavior. For UI work, verify supported window sizes, keyboard behavior, and dark/light/system themes. A task contract may add stricter checks.

An in-scope failure is fixed. A pre-existing or external failure is recorded with its exact command, impact, and tracking ID.

### 4. Self-review

Codex performs a distinct final review pass:

1. Inspect the complete diff and changed-path list.
2. Map every acceptance criterion to code or test evidence.
3. Check architecture compliance, security boundaries, error paths, and rollback implications.
4. Confirm no unrelated changes or secrets are staged.
5. Record findings and corrections in `.ai/HANDOFF.md` for planned tasks.
6. Rerun affected checks after any correction.

Possible outcomes are `complete`, `changes_required`, or `blocked`. Codex may correct its own findings, but it must record material findings and their resolution.

### 5. Update project memory

Codex updates, in order:

1. `.ai/CHANGELOG.md`
2. `.ai/PROJECT_STATE.md`
3. `.ai/HANDOFF.md` for planned tasks
4. `.ai/SESSION_NOTES.md`
5. `.ai/TODO.md`

Completed contract detail moves to the changelog; `.ai/HANDOFF.md` returns to the `idle` template when no planned task is active.

### 6. Publish

Codex commits and pushes only after verification and self-review. A draft PR remains the default so the human can inspect or merge it, but no second AI reviewer is required.

## State model

```text
TODO candidate
    -> planned
    -> HANDOFF draft
    -> ready
    -> in_progress
    -> self_review
    -> complete
    -> CHANGELOG + PROJECT_STATE
```

`blocked` is an interrupt state with evidence, impact, attempted alternatives, and one concrete request to the human owner. `changes_required` returns to `in_progress` under the same task ID.

## Task design and scaling

- One task delivers one coherent outcome and uses a stable ID.
- Epics are decomposed into ordered vertical slices before implementation.
- One worktree has at most one active planned task contract.
- Parallel Codex sessions use separate branches/worktrees and non-overlapping file ownership.
- Migrations, global types, routing, native state, and design tokens are serialization points.
- Cross-cutting work begins with an ADR and ordered implementation slices.
- Integration work receives its own verification and self-review pass.

## Blocking rules

Codex stops and records a blocker when:

- A required human product decision is missing.
- A safe solution exceeds authorized scope.
- User-owned changes overlap required edits.
- An unauthorized destructive or data-loss risk appears.
- Credentials, tools, or external services are unavailable.
- Acceptance criteria conflict.

Difficulty, long runtime, or an initially failed check is not itself a blocker; Codex first exhausts safe in-scope diagnostics and alternatives.

## Documentation hygiene

- Use ISO dates and stable task/decision IDs.
- Use `None` for intentionally empty required fields.
- Label assumptions until replaced by evidence.
- Never store secrets or personal data in `.ai/`.
- Keep `SESSION_NOTES.md` temporary.
- Keep `CHANGELOG.md` append-only and newest first, except factual corrections.
- Update state in the same PR as the implementation it describes.

## Automatic GitHub publication

Every completed task is published at a verified task boundary:

1. Start from current `master` on `agent/<short-description>`.
2. Run task-specific verification and self-review.
3. Stage only task-owned paths.
4. Use a conventional commit message.
5. Publish with `scripts/publish-task.ps1`.
6. The versioned post-commit hook also pushes ordinary commits automatically.
7. Confirm the branch and draft PR exist on GitHub.

One-time hook setup:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/install-git-hooks.ps1
```

Standard publication:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/publish-task.ps1 `
  -Message "docs(workflow): adopt Codex-only delivery" `
  -Paths WORKFLOW.md, AGENTS.md
```

Never publish `.env*`, databases, private keys, actual secret material, failed in-scope checks, or unrelated changes. GitHub publication does not imply human approval or automatic merge.

## Definition of done

A task is complete when:

- The requested outcome and all acceptance criteria are satisfied.
- Required tests and checks pass, or an authorized/pre-existing limitation is precisely documented.
- Relevant tests and documentation are current.
- Codex completed and recorded its self-review.
- No unresolved scope, architecture, security, or data-safety issue remains.
- The task-owned commit is pushed and a draft PR exists when working off `master`.

No Hermes handoff, review, or acceptance is required.
