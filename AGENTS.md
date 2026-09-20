# AGENTS.md — Aether Coding Agent Instructions

## Project identity

You are working on **Aether** — a local-first, modular personal workspace for Windows built with Tauri 2, React, TypeScript, Rust, SQLite, and Tailwind CSS.

Aether is a serious long-term desktop product, not a prototype, generic dashboard, or disposable AI-generated application.

The repository is the source of truth.

---

## Architecture

* **Desktop shell:** Tauri 2 + Rust — `src-tauri/`
* **Frontend:** React 19 + TypeScript 6 + Vite 8 — `src/`
* **Styling:** Tailwind CSS v4 with custom design tokens — `src/styles/index.css`
* **State:** Zustand stores — `src/stores/`
* **Routing:** React Router — `src/App.tsx`
* **Database:** SQLite via bundled `rusqlite`
* **Package manager:** pnpm
* **AI providers:** provider abstraction with supported providers configured through the existing Rust boundary
* **Native trust boundary:** frontend → typed invoke wrappers → Tauri commands → Rust repositories/services → SQLite / filesystem / OS / external provider

Do not bypass established architectural boundaries for convenience.

---

## Source of truth and context discipline

The repository and its current implementation are authoritative.

At the start of a task:

1. Read `AGENTS.md`.
2. Follow the session bootstrap defined in `WORKFLOW.md`.
3. Read `.ai/PROJECT_STATE.md`.
4. Read only the architecture documents, ADRs, source files, tests, and other control documents relevant to the current task.

Do NOT inspect the entire repository by default.

Do NOT read every ADR, every historical document, every source file, or every old roadmap entry unless the task genuinely requires it.

Prefer:

* targeted repository search;
* dependency tracing;
* existing implementation patterns;
* relevant tests;
* relevant ADRs;
* current control documents.

Avoid:

* broad repository exploration without purpose;
* re-reading unrelated historical context;
* loading large files that do not affect the current task;
* using chat history as the primary source of truth when the repository already contains the relevant decision.

When documentation conflicts with implementation, investigate the discrepancy rather than silently choosing one.

When historical roadmap text conflicts with `.ai/PROJECT_STATE.md` or the current implementation, treat the current repository state as authoritative.

---

## Design rules (mandatory)

Aether must NOT look like:

* a Tailwind starter dashboard;
* a shadcn/ui demo;
* a purple-gradient ChatGPT clone;
* a generic SaaS admin panel;
* an AI-generated productivity template.

The visual standard is inspired by Linear, Raycast, Arc, Things, and Craft:

* calm;
* restrained;
* premium;
* neutral color palette with indigo accent;
* consistent spacing;
* consistent typography;
* consistent border radii;
* dark/light/system theme support;
* deliberate information hierarchy;
* thoughtful empty states;
* minimal visual noise;
* visually cohesive desktop-first interaction patterns.

Avoid unnecessary decoration.

Every UI decision should support clarity, hierarchy, usability, or product identity.

---

## Forbidden design patterns

Do NOT:

* use random gradients;
* use neon glows;
* use excessive glassmorphism;
* add fake charts;
* add lorem ipsum;
* add dead buttons;
* add fake statistics;
* add fake notifications;
* add fake integrations;
* add placeholder content that appears real;
* add purple-to-blue AI gradients;
* disable focus outlines globally;
* use excessive rounded corners;
* use excessive pill-shaped elements;
* use generic Lucide icons without curation;
* scatter raw CSS throughout components;
* introduce visual patterns that conflict with the existing Aether design system.

Use existing design tokens and reusable primitives whenever possible.

---

## Development commands

```bash
pnpm dev
pnpm tauri:dev
pnpm build
pnpm typecheck
pnpm lint
pnpm test
pnpm format
pnpm check
```

For Rust/native changes, use the relevant commands:

```bash
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
```

Run only the checks relevant to the task during iteration.

Before completing meaningful cross-layer or release-relevant work, run the appropriate complete validation set required by the task or `WORKFLOW.md`.

Do not claim tests passed unless they were actually executed successfully.

---

## Commit style

Use conventional commits.

Examples:

```text
feat(scope): description
fix(scope): description
refactor(scope): description
docs(scope): description
test(scope): description
build(scope): description
chore(scope): description
```

Commit messages must describe the actual task.

Do not mix unrelated changes into one commit.

---

## File conventions

* Always use strict TypeScript.
* Use `@/` path aliases for `src` imports where appropriate.
* Keep domain logic outside React components.
* Keep external API logic outside presentation components.
* Use repository/service interfaces for data access.
* Keep SQL inside the Rust persistence layer.
* Do not expose database implementation details directly to React.
* Prefer small, focused modules over large multifunction files.
* Reuse existing abstractions before introducing new ones.
* Avoid `any` unless documented and genuinely unavoidable.
* Avoid speculative abstractions.
* Avoid premature generalization.
* Avoid unnecessary dependencies.
* Avoid duplicate representations of the same domain concept.

---

## Current project status

Aether is an active alpha product.

Do not infer the current implementation state from this file.

The canonical current state is determined by:

1. `.ai/PROJECT_STATE.md`;
2. the current repository implementation;
3. current architecture documentation and ADRs;
4. current release documentation.

`IDEA.md` contains long-term product direction and historical planning context.

Do NOT implement roadmap items merely because they appear in `IDEA.md`.

Only implement work that belongs to the current approved task or approved roadmap.

---

## Task classification

Before implementation, classify each task as one of:

### `direct_codex`

Use for work that is:

* small;
* clear;
* reversible;
* low risk;
* local in scope;
* governed by existing conventions;
* unlikely to affect architecture or migrations.

Examples:

* small UI fix;
* copy correction;
* focused test repair;
* isolated bug fix;
* straightforward styling adjustment;
* small refactor fully covered by existing architecture.

### `planned_codex`

Use for work involving:

* new product features;
* architecture;
* security;
* migrations;
* new domain entities;
* external integrations;
* OAuth;
* cross-layer changes;
* Safe Actions;
* AI behavior;
* significant data model changes;
* ambiguous requirements;
* high-risk changes;
* multi-module work;
* changes that materially affect product behavior.

A `planned_codex` task requires a complete `.ai/HANDOFF.md` task contract with status `ready` before production code changes begin.

If a `direct_codex` task reveals hidden architecture, security, migration, product, or scope risk:

1. reclassify it as `planned_codex`;
2. update `.ai/HANDOFF.md`;
3. complete the readiness gate;
4. continue only after the task contract is valid.

---

## Task execution rules

Every task must have a bounded goal.

Before changing code, determine:

* the exact goal;
* success criteria;
* in-scope areas;
* explicitly out-of-scope areas;
* validation required;
* stop condition.

Do not expand scope simply because adjacent improvements are visible.

If unrelated issues are discovered:

* report them;
* record them in the appropriate backlog if necessary;
* do NOT fix them unless they are required to complete the current task safely.

Do not perform opportunistic refactors.

Do not introduce:

* speculative abstractions;
* unused extension points;
* placeholder systems;
* dead code;
* fake future integrations;
* dependencies for hypothetical future work;
* architectural layers with no current use.

Prefer the smallest implementation that cleanly satisfies the approved architecture and acceptance criteria.

Stop once the approved goal and acceptance criteria are satisfied.

---

## Scope discipline

When a task specifies `OUT OF SCOPE`, treat it as binding.

Do not implement out-of-scope functionality unless the current task cannot be completed safely without it.

If additional work becomes necessary, explicitly document why.

Do not silently broaden a milestone.

Examples:

If implementing MyTimetable integration:

* do not also implement Brightspace;
* do not redesign Pulse;
* do not add Google Calendar;
* do not introduce n8n;
* do not refactor unrelated Tasks code.

If implementing Connections UI:

* do not build OAuth unless explicitly required;
* do not redesign Settings;
* do not change unrelated providers.

---

## Codex collaboration model

Aether uses a repository-centered Codex workflow.

Different Codex threads, models, sessions, or isolated Git worktrees may be used for separate tasks.

The repository and its control documents remain the shared source of truth.

Each implementation task must have:

* one clearly defined owner;
* one clearly defined scope;
* clear acceptance criteria;
* clear file/domain ownership where parallel work is involved.

Parallel work is allowed only when tasks are sufficiently independent.

Do NOT allow multiple worktrees or agents to independently modify the same conflict-prone core areas without explicit coordination.

Conflict-prone areas include:

* the same migration;
* the same shared domain abstraction;
* the same architecture decision;
* global routing;
* core settings infrastructure;
* central Safe Actions logic;
* shared database infrastructure;
* provider registries;
* central IPC contracts.

Prefer sequential work when architecture or domain ownership overlaps.

No Hermes planning, handoff, review, or acceptance is required unless explicitly requested for a separate workflow.

---

## Session behavior

At the start of every session:

1. read the required control documents defined by `WORKFLOW.md`;
2. determine the active task;
3. determine task classification;
4. inspect only task-relevant code and documentation;
5. establish the scope before implementation.

Do not begin with a broad repository rewrite proposal.

Do not redesign systems that already satisfy the task.

Do not assume older chat context is authoritative if repository state exists.

---

## Review rules

Review work is distinct from implementation work.

When asked to review existing code or a branch:

* inspect the diff;
* inspect only relevant surrounding architecture;
* compare implementation against acceptance criteria;
* check correctness;
* check security;
* check architecture;
* check maintainability;
* check UX where applicable;
* check test coverage;
* check whether scope was exceeded.

Do NOT rewrite working code merely to express stylistic preference.

Do NOT recommend large redesigns when a small correction is sufficient.

Classify review findings where useful as:

* blocking;
* important;
* optional.

Every finding should point to:

* concrete behavior;
* concrete file or layer;
* concrete risk;
* smallest reasonable correction.

A reviewer should not silently turn a review task into a new implementation milestone.

---

## Self-review requirements

After implementation, perform a distinct self-review.

The self-review must:

* inspect the final diff;
* verify scope discipline;
* verify no unrelated files changed;
* verify architectural boundaries;
* verify security-sensitive behavior;
* verify migrations if applicable;
* verify tests;
* map each acceptance criterion to evidence.

Do not rely only on the fact that tests passed.

Check whether the implementation actually satisfies the product requirement.

---

## Completion report

At task completion, report:

* what behavior changed;
* files or major areas changed;
* tests/checks executed;
* acceptance criteria evidence;
* known limitations;
* relevant architecture implications;
* follow-up work discovered but intentionally not implemented.

Do not produce a long narrative when a concise engineering summary is sufficient.

---

## Automatic GitHub publication

Implementation tasks that satisfy their acceptance criteria must be committed and pushed unless the task is explicitly marked as:

* review-only;
* investigation-only;
* planning-only;
* documentation analysis;
* local experiment;
* temporary debugging.

For publishable implementation work:

* use a task branch from `main` or `master`;
* publish through `scripts/publish-task.ps1`;
* stage only task-owned paths;
* use conventional commit messages;
* ensure validation required by the task passes before publication.

The repository's versioned `post-commit` hook may automatically push ordinary commits after one-time installation with:

```text
scripts/install-git-hooks.ps1
```

Never publish:

* secrets;
* `.env*`;
* databases;
* private keys;
* credential files;
* tokens;
* personal data;
* backups;
* failed checks presented as successful work;
* unrelated user changes.

A pushed draft PR is not automatically approved or merged.

Do not merge without the workflow's required approval/review state.

---

## Database architecture

* SQLite via bundled `rusqlite`.
* No system SQLite dependency.
* Database location:

```text
%APPDATA%/com.aether.desktop/aether.db
```

* All database access goes through the Rust layer.
* No frontend SQL.
* Versioned migrations live in:

```text
src-tauri/src/db/migrations.rs
```

* Repository pattern:

```text
Rust repositories
→ domain/native services where needed
→ Tauri commands
→ TypeScript invoke wrappers
→ frontend hooks/stores/components
```

* Migrations are append-only.
* Never modify a previously applied migration.
* Add new migrations as new entries in `MIGRATIONS`.
* Preserve foreign-key integrity.
* Preserve backup/restore compatibility.
* Treat user data as sensitive.

---

## Adding a new domain entity

When adding a new persistent domain entity:

1. Add migration SQL to `MIGRATIONS`.
2. Create or extend the Rust repository in `src-tauri/src/db/repositories/`.
3. Add domain/service logic where required.
4. Add Tauri commands in the appropriate command module.
5. Register commands in `src-tauri/src/lib.rs`.
6. Add strict TypeScript types.
7. Add Zod schemas where runtime validation is required.
8. Add invoke wrappers in the appropriate frontend database/API module.
9. Add hooks/store integration if needed.
10. Add Rust repository tests using an in-memory database where appropriate.
11. Add frontend tests for user-visible behavior where appropriate.
12. Update database documentation if the schema is durable.
13. Update or create an ADR if the change introduces a durable architectural decision.

Do not skip layers merely to reduce implementation time.

---

## Security and privacy rules

Aether is local-first and may contain highly sensitive user information.

Treat all user data as sensitive by default.

Do NOT:

* expose secrets to React;
* return raw credentials over IPC;
* log API keys;
* log tokens;
* store secrets in localStorage;
* commit secrets;
* automatically upload user content;
* send local files or notes to an AI provider without explicit authorized context;
* expose absolute Source roots unnecessarily;
* allow arbitrary shell execution through AI actions;
* allow AI-generated actions to bypass user approval where approval is required;
* weaken existing Safe Actions boundaries.

External integrations must use the least privilege required.

OAuth or API tokens must be stored through an approved secure native mechanism.

Sensitive integration logic belongs behind the Rust/native trust boundary unless an ADR explicitly defines otherwise.

---

## Safe Actions rules

External or local mutations initiated through AI or automation must preserve the existing Safe Actions model where appropriate.

A model must not directly:

* approve its own action;
* bypass preview;
* replace approved arguments after approval;
* perform arbitrary destructive operations;
* silently broaden permissions.

Prefer:

```text
AI/tool proposes action
→ Aether validates
→ user sees consequence
→ user approves
→ bounded action executes
→ Activity records result
```

Read-only operations may use lighter boundaries if explicitly safe and bounded.

---

## External integrations

New integrations should follow a connector/service pattern rather than embedding provider-specific logic throughout the UI.

Prefer architecture similar to:

```text
provider connector
→ native service
→ normalized domain model
→ repository
→ typed IPC
→ frontend
```

Provider-specific payloads should be normalized before reaching presentation components.

Do not make Pulse, Spaces, or generic modules depend directly on provider-specific API schemas.

When polling external systems:

* support clear sync status;
* store last successful sync time where useful;
* handle duplicate external records;
* handle updates;
* handle deletion/removal semantics intentionally;
* handle transient failures without corrupting local state;
* avoid excessive polling.

Use webhooks where they are reliable and justified.

---

## Dependency policy

Do not add a dependency merely because it simplifies a small piece of work.

Before adding a dependency, determine:

* whether the platform or existing stack already provides the capability;
* maintenance quality;
* security implications;
* bundle/native cost;
* long-term product value;
* whether a small internal implementation would be clearer.

Any meaningful new dependency must be justified in the task summary.

Avoid dependencies for trivial utilities.

---

## Testing philosophy

Tests should protect behavior, architecture boundaries, persistence rules, and regressions.

Prioritize:

* domain logic;
* repositories;
* migrations;
* IPC contracts;
* user-visible workflows;
* error handling;
* security boundaries;
* sync behavior;
* Safe Actions;
* destructive or recovery operations.

Avoid meaningless tests that only mirror implementation details.

Do not delete or weaken tests merely to make a task pass.

If existing tests conflict with intentionally changed behavior, update them deliberately and explain why.

---

## Architecture decisions

Create or update an ADR when work introduces a durable decision involving:

* trust boundaries;
* persistence strategy;
* integration architecture;
* provider abstractions;
* synchronization model;
* security model;
* backup/restore behavior;
* cross-domain architecture;
* major dependency introduction;
* plugin/module architecture;
* AI action boundaries.

Do not create ADRs for trivial implementation details.

---

## Performance rules

Do not optimize prematurely.

However:

* avoid loading large datasets unnecessarily;
* avoid repeated expensive IPC calls in tight render loops;
* avoid reading entire external data sources when incremental sync is possible;
* avoid blocking the UI thread with native scanning or network work;
* preserve bounded search and context behavior;
* keep Pulse deterministic and efficient;
* prefer pagination/bounded queries where data may grow.

Measure or justify meaningful performance changes.

---

## Product behavior rules

Aether should remain understandable without AI.

AI should enhance the product rather than become the only interface.

Do not make core user data inaccessible without a model/provider connection.

Do not silently convert deterministic product behavior into AI-generated behavior.

Prefer deterministic logic for:

* deadlines;
* task state;
* sync status;
* sorting;
* filtering;
* permissions;
* audit history;
* source ownership;
* destructive actions.

Use AI where reasoning, summarization, transformation, or assistance provides genuine value.

---

## Documentation rules

Update documentation when behavior or architecture materially changes.

Do not update documentation speculatively.

Do not document features that do not exist.

Do not leave durable architecture documented only in chat.

Important implementation decisions should live in:

* `.ai/`;
* `docs/`;
* ADRs;
* code comments only where local reasoning requires them.

Avoid excessive comments that merely restate code.

---

## Future roadmap

Long-term product direction is documented in `IDEA.md`.

Treat it as planning context, not implementation authorization.

Current implementation priority is controlled by:

* `.ai/PROJECT_STATE.md`;
* `.ai/TODO.md`;
* `.ai/HANDOFF.md`;
* the approved task;
* current repository state.

Do not skip ahead to future roadmap phases unless explicitly instructed.
