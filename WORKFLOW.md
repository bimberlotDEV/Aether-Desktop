# Aether Multi-Agent Development Workflow

This document defines how Hermes and Codex collaborate on Aether. The system separates design authority from implementation authority while keeping both agents synchronized through small, canonical state files.

## Operating principles

1. One source of truth exists for each kind of information.
2. Hermes decides what and why; Codex executes how within approved boundaries.
3. Exactly one handoff is active for implementation at a time.
4. No implementation begins from chat context alone.
5. Verification evidence is part of the implementation, not optional follow-up.
6. Architecture changes stop execution until Hermes explicitly approves them.
7. Documentation is updated in the same task as the behavior it describes.

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

Hermes never writes production code unless the human explicitly requests it. Hermes may edit planning, architecture, decision, and review documentation.

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

Codex never redesigns architecture, changes product scope, or reprioritizes the backlog unless `.ai/HANDOFF.md` explicitly authorizes it. When the approved design cannot be implemented safely, Codex stops and returns evidence to Hermes.

## Source-of-truth map

| Question | Canonical file | Primary owner | Update timing |
| --- | --- | --- | --- |
| What is true now? | `.ai/PROJECT_STATE.md` | Shared; Codex updates implementation facts | After verified change or blocker |
| What should be done next? | `.ai/TODO.md` | Hermes | During planning and review |
| What may Codex implement now? | `.ai/HANDOFF.md` | Hermes plans; Codex reports | At every ownership transition |
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

“Codex reads the AI files first” means Codex establishes task context from these control documents before inspecting or editing implementation files. After the readiness gate passes, Codex reads the product files named by the handoff and any directly necessary dependencies.

## End-to-end delivery cycle

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

An epic is never placed directly in `HANDOFF.md`; Hermes decomposes it in `.ai/TODO.md` first.

## Scaling and parallel work

- One repository worktree has one active `HANDOFF.md` and one Codex owner.
- Parallel work uses separate Git branches and worktrees with non-overlapping file ownership.
- Each worktree carries its own handoff state; integration happens through a dedicated review/integration task.
- Shared migrations, global types, routing, and design tokens are serialization points and must not be edited concurrently without an integration plan.
- Cross-cutting changes use an architecture task first, followed by ordered implementation tasks.
- Completed handoff detail moves to the changelog; live state files remain short.

## Blocking and escalation

Codex stops and marks the handoff `blocked` when:

- A required architecture decision is missing.
- The safe solution exceeds allowed paths or scope.
- User-owned changes overlap required edits.
- A destructive migration or data-loss risk was not authorized.
- Required credentials, tools, or external services are unavailable.
- Acceptance criteria conflict.

The block report contains evidence, impact, attempted safe alternatives, the decision owner, and one concrete resolution request.

## Documentation hygiene

- Use ISO dates (`YYYY-MM-DD`) and stable task/decision IDs.
- Use the literal `None` instead of omitting a required field.
- Label assumptions and replace them with evidence as soon as possible.
- Never store secrets or personal data in `.ai/`.
- Keep `SESSION_NOTES.md` temporary; promote durable facts to their canonical file.
- Keep `CHANGELOG.md` append-only and newest first.
- Update state in the same pull request or commit as implementation.

## Definition of done

A task is complete only when:

- Acceptance criteria are satisfied.
- Required tests and checks pass, or an authorized exception is documented.
- New behavior has appropriate tests.
- No unapproved scope or architecture changes remain.
- Relevant documentation is current.
- Changelog, project state, handoff, and session notes are updated.
- Hermes records `accepted` in the handoff.

Until Hermes accepts it, Codex's work is `ready_for_review`, not complete.
