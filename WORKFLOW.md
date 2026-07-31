# Aether Multi-Agent Development Workflow

This document defines how Hermes and Codex collaborate on Aether. The system separates design authority from implementation authority while keeping both agents synchronized through small, canonical state files.

## Operating principles

1. One source of truth exists for each kind of information.
2. Every task is classified as `direct_codex` or `hermes_codex` before work begins.
3. Hermes decides what and why for `hermes_codex` work; Codex executes how within approved boundaries.
4. Exactly one handoff is active for `hermes_codex` implementation at a time.
5. A `direct_codex` task does not require a Hermes handoff or review unless Codex escalates it.
6. No `hermes_codex` implementation begins from chat context alone.
7. Verification evidence is part of the implementation, not optional follow-up.
8. Architecture changes stop execution until Hermes explicitly approves them.
9. Documentation is updated in the same task as the behavior it describes.

## Mandatory task routing

Before planning or implementation, classify the request using exactly one of these values:

| Route | Use when | Process owner | Handoff required | Hermes review required |
| --- | --- | --- | --- | --- |
| `direct_codex` | Small, bounded, low-risk work with no unresolved design decision | Codex | No | No, unless requested or escalated |
| `hermes_codex` | Architectural, security-sensitive, cross-cutting, ambiguous, or product-shaping work | Hermes -> Codex -> Hermes | Yes | Yes |

### `direct_codex` criteria

Use `direct_codex` when all of the following are true:

- The requested outcome and expected behavior are already clear.
- No architecture, product, security, privacy, or data-lifecycle decision is required.
- No irreversible migration or broad public interface change is involved.
- The change is small enough for Codex to implement and verify in one focused cycle.
- Existing repository conventions determine the implementation approach.

Typical examples:

- Fixing lint, formatting, or type errors.
- Correcting documentation or version metadata.
- Adding a focused regression test.
- Fixing a small bug with a verified cause.
- Performing a local, behavior-preserving refactor.

For this route, Hermes must not create an elaborate handoff or implement the task. If asked to assess the work, Hermes responds with:

```text
Classification: direct_codex
Reason: <one sentence>
Next action: Codex may implement and verify directly.
```

Codex reads the control documents, checks the working tree, implements the bounded change, runs proportionate verification, and records durable changes in the changelog. If Codex discovers a missing design decision or increased risk, Codex stops and reclassifies the work as `hermes_codex`.

### `hermes_codex` criteria

Use `hermes_codex` when any of the following is true:

- A new product feature, domain, or multi-step epic is being introduced.
- Architecture, database schema, migration strategy, or shared interfaces must be designed.
- Security, credentials, privacy, destructive data behavior, or external AI access is involved.
- A refactor crosses several layers or ownership boundaries.
- Requirements are ambiguous or trade-offs materially affect product behavior.
- Independent architectural review materially reduces delivery risk.

For this route, Hermes responds with:

```text
Classification: hermes_codex
Reason: <one sentence>
Next action: Hermes prepares or updates .ai/HANDOFF.md; Codex implements; Hermes reviews.
```

Hermes may plan and review but does not implement the handoff. Codex may implement and verify but does not accept its own work. Hermes must never both implement and accept the same `hermes_codex` task.

### Routing precedence

When uncertain, choose `hermes_codex` only because a concrete risk or unresolved decision requires it, not merely because two agents are available. The human may explicitly override the route. Record the override and its reason in session notes.

## Agent responsibilities

### Hermes — planner, architect, and reviewer

Hermes owns:

- Repository analysis and system-level reasoning.
- Product and technical prioritization.
- Architecture, boundaries, interfaces, and significant trade-offs.
- Breaking epics into independently verifiable tasks.
- Writing acceptance criteria and identifying risks.
- Maintaining `.ai/ARCHITECTURE.md` and prioritizing `.ai/TODO.md`.
- Preparing implementation-ready `.ai/HANDOFF.md` assignments.
- Reviewing Codex's code, tests, evidence, and deviations.
- Accepting work or returning precise findings.
- Classifying incoming work as `direct_codex` or `hermes_codex` when Hermes is consulted.

Hermes never writes production code unless the human explicitly requests it. Hermes may edit planning, architecture, decision, and review documentation. For `direct_codex` tasks, Hermes routes the task to Codex without creating unnecessary planning artifacts. For `hermes_codex` tasks, Hermes never implements and accepts the same task.

### Codex — implementer, executor, and verifier

Codex owns:

- Implementing the active handoff.
- Debugging, scoped refactoring, migrations, and performance improvements.
- Adding or updating automated tests.
- Running required static checks, tests, and builds.
- Reporting evidence, deviations, risks, and blockers.
- Updating implementation facts in `.ai/PROJECT_STATE.md`.
- Appending completed work to `.ai/CHANGELOG.md`.
- Updating `.ai/HANDOFF.md` implementation results and `.ai/SESSION_NOTES.md`.
- Executing `direct_codex` tasks without waiting for a handoff when the routing criteria are satisfied.

Codex never redesigns architecture, changes product scope, or reprioritizes the backlog unless a `hermes_codex` handoff explicitly authorizes it. When a direct task reveals architecture or scope uncertainty, Codex stops and escalates it to Hermes.

## Source-of-truth map

| Question | Canonical file | Primary owner | Update timing |
| --- | --- | --- | --- |
| What is true now? | `.ai/PROJECT_STATE.md` | Shared; Codex updates implementation facts | After verified change or blocker |
| What should be done next? | `.ai/TODO.md` | Hermes | During planning and review |
| Which route applies and what may Codex implement? | `WORKFLOW.md`; `.ai/HANDOFF.md` only for `hermes_codex` | Hermes classifies complex work; Codex executes | Before work and at multi-agent transitions |
| What architecture is binding? | `.ai/ARCHITECTURE.md` | Hermes | When decisions change |
| What was completed? | `.ai/CHANGELOG.md` | Codex | After verification |
| What helps resume this session? | `.ai/SESSION_NOTES.md` | Active agent | During work; replace next session |
| What is the collaboration process? | `WORKFLOW.md` | Hermes | Rarely, through reviewed process changes |
| What repository rules always apply? | `AGENTS.md` | Project maintainers | When durable coding rules change |

Information must not be duplicated in full. Link to the canonical owner when another document needs context.

## Mandatory session startup

Both agents begin by reading, in order:

1. `AGENTS.md` for non-negotiable repository instructions.
2. `.ai/PROJECT_STATE.md` for the current verified state.
3. `.ai/HANDOFF.md` for the active task and ownership.
4. `.ai/ARCHITECTURE.md` for binding boundaries.
5. `.ai/TODO.md` for priority and dependencies.
6. `.ai/SESSION_NOTES.md` for transient continuation context.
7. The newest relevant entry in `.ai/CHANGELOG.md`.

“Codex reads the AI files first” means Codex establishes task context from these control documents before inspecting or editing implementation files. For `direct_codex`, Codex then inspects the smallest relevant implementation scope. For `hermes_codex`, Codex proceeds only after the handoff readiness gate passes.

## `hermes_codex` delivery cycle

### 1. Hermes analyses

Hermes inspects the repository, relevant implementation, tests, documentation, current state, and existing decisions. Hermes distinguishes verified facts from assumptions.

### 2. Hermes plans

Hermes updates:

- `.ai/ARCHITECTURE.md` only when a binding design changes.
- `.ai/TODO.md` with priority, dependencies, and readiness.
- `.ai/PROJECT_STATE.md` if the milestone, blocker, or decision index changed.
- `.ai/HANDOFF.md` with one bounded, implementation-ready task.
- `.ai/SESSION_NOTES.md` with useful planning context.

### 3. Readiness gate

Codex may set the handoff to `in_progress` only when all are true:

- Task ID, objective, context, and owner are present.
- Allowed files or bounded directories are explicit.
- Out-of-scope behavior is explicit.
- Acceptance criteria are observable.
- Required validation commands are listed.
- Dependencies and blocking decisions are resolved.
- Architecture changes, if any, have an accepted decision.
- The working tree has been inspected for user-owned changes.

If any item is missing and cannot be safely inferred, Codex marks the handoff `blocked` and asks Hermes for the smallest necessary decision.

This readiness gate applies to `hermes_codex` tasks. A correctly classified `direct_codex` task does not create or replace `.ai/HANDOFF.md`.

### 4. Codex implements

Codex:

1. Marks the handoff `in_progress` and records the session start.
2. Reads only the relevant implementation paths and dependencies.
3. Makes the smallest cohesive change satisfying the acceptance criteria.
4. Preserves unrelated user changes.
5. Adds or updates tests close to the behavior owner.
6. Does not expand scope because adjacent work is convenient.

### 5. Codex verifies

The default frontend gate is:

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

For migrations, verify a fresh database, upgrade from the previous schema, idempotence, and relevant repository tests. For UI changes, perform visual and interaction verification at supported window sizes and themes. The handoff may add stricter commands.

A failed check is not hidden. Codex either fixes an in-scope failure or records the exact failure, suspected ownership, and blocking impact.

### 6. Codex hands back

Codex updates, in order:

1. `.ai/CHANGELOG.md` with completed, verified work.
2. `.ai/PROJECT_STATE.md` with new facts and check results.
3. `.ai/HANDOFF.md` with files changed, verification evidence, deviations, and status `ready_for_review`.
4. `.ai/SESSION_NOTES.md` with the exact review/resume point.
5. `.ai/TODO.md` only for newly discovered debt or defects; Codex does not reprioritize them.

### 7. Hermes reviews

Hermes reviews the diff against architecture and acceptance criteria, then chooses one outcome:

- `accepted`: all criteria pass; update milestone/backlog and prepare the next handoff.
- `changes_requested`: add precise findings to the same handoff and return ownership to Codex.
- `blocked`: record the external dependency or decision required.
- `superseded`: close the task with rationale and create a replacement task ID.

Hermes does not silently rewrite Codex's implementation during review.

## State model

```text
TODO candidate
    -> needs_design
    -> ready_for_handoff
    -> HANDOFF ready
    -> in_progress
    -> ready_for_review
    -> accepted
    -> CHANGELOG + PROJECT_STATE
```

`blocked` is an interrupt state with an owner and explicit resolution path. `changes_requested` returns to `in_progress` under the same task ID unless scope materially changes.

## Task design standard

Every task must:

- Deliver one coherent outcome.
- Use a stable ID such as `SPACE-012`, `AI-004`, `TECH-001`, or `DOC-003`.
- Be small enough to implement and review in one focused cycle.
- State dependencies and non-goals.
- Prefer vertical slices that include types, persistence, commands, UI, and tests when appropriate.
- Separate irreversible migrations, security redesigns, and broad refactors into explicitly reviewed tasks.

An epic is never placed directly in `HANDOFF.md`; Hermes decomposes it in `.ai/TODO.md` first. Small direct tasks do not need backlog promotion unless they were already tracked or produce durable follow-up work.

## Scaling and parallel work

- One repository worktree has one active `HANDOFF.md` and one Codex owner.
- Parallel work uses separate Git branches and worktrees with non-overlapping file ownership.
- Each worktree carries its own handoff state; integration happens through a dedicated review/integration task.
- Shared migrations, global types, routing, and design tokens are serialization points and must not be edited concurrently without an integration plan.
- Cross-cutting changes use an architecture task first, followed by ordered implementation tasks.
- Completed handoff detail moves to the changelog; live state files remain short.

## Blocking and escalation

Codex stops when:

- A required architecture decision is missing.
- The safe solution exceeds allowed paths or scope.
- User-owned changes overlap required edits.
- A destructive migration or data-loss risk was not authorized.
- Required credentials, tools, or external services are unavailable.
- Acceptance criteria conflict.

For `hermes_codex`, Codex marks the handoff `blocked`. For `direct_codex`, Codex records the evidence in session notes and requests reclassification to `hermes_codex`. Every block report contains evidence, impact, attempted safe alternatives, the decision owner, and one concrete resolution request.

## Documentation hygiene

- Use ISO dates (`YYYY-MM-DD`) and stable task/decision IDs.
- Use the literal `None` instead of omitting a required field.
- Label assumptions and replace them with evidence as soon as possible.
- Never store secrets or personal data in `.ai/`.
- Keep `SESSION_NOTES.md` temporary; promote durable facts to their canonical file.
- Keep `CHANGELOG.md` append-only and newest first.
- Update state in the same pull request or commit as implementation.

## Automatic GitHub publication

Every completed task is published to GitHub. Automation happens at verified task boundaries, never on every file save.

1. Work on a task branch named `agent/<short-description>` when starting from `main` or `master`.
2. Run all task-specific verification before committing.
3. Stage only the paths owned by the task; never use an unreviewed blanket stage in a mixed worktree.
4. Use a conventional commit message.
5. Run `scripts/publish-task.ps1`, which validates the staged scope, blocks common secret/database files, commits, pushes, and opens a draft PR when needed.
6. The versioned `.githooks/post-commit` hook also pushes ordinary commits automatically.
7. A task is not published until the branch exists on GitHub. If push or PR creation fails, report it and retry; do not claim completion.

One-time setup per clone:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/install-git-hooks.ps1
```

Standard task publication:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/publish-task.ps1 `
  -Message "docs(workflow): describe task routing" `
  -Paths WORKFLOW.md, AGENTS.md
```

Safety rules:

- Never publish `.env*`, databases, private keys, credential files, or secrets.
- Never auto-commit unrelated user changes.
- Never bypass failed verification merely to synchronize GitHub.
- Use `AETHER_SKIP_AUTO_PUSH=1` only for an explicit, temporary local-only commit; the task remains incomplete until manually pushed.
- GitHub publication does not imply merge approval. Draft PRs remain subject to the applicable `direct_codex` or `hermes_codex` review policy.

## Definition of done

A `direct_codex` task is complete when:

- The requested bounded outcome is implemented.
- Proportionate checks pass or a limitation is reported.
- Relevant durable documentation is updated.
- No architecture or scope question remains that requires escalation.
- The commit is pushed to GitHub and a draft PR exists when working off the default branch.

A `hermes_codex` task is complete only when:

- Acceptance criteria are satisfied.
- Required tests and checks pass, or an authorized exception is documented.
- New behavior has appropriate tests.
- No unapproved scope or architecture changes remain.
- Relevant documentation is current.
- Changelog, project state, handoff, and session notes are updated.
- Hermes records `accepted` in the handoff.
- The accepted commits are pushed to GitHub and the PR state is accurately reported.

Until Hermes accepts it, Codex's work is `ready_for_review`, not complete.
