# Prioritized Backlog

> Canonical queue of Aether work that is not the active implementation contract.
>
> This file answers:
>
> **What should Aether work on next?**
>
> Active `planned_codex` work belongs in `.ai/HANDOFF.md`.
> Completed history belongs in `.ai/CHANGELOG.md`.

---

## Metadata

| Field                   | Value                                   |
| ----------------------- | --------------------------------------- |
| Schema version          | 2                                       |
| Last updated            | 2026-09-20                              |
| Prioritized by          | Codex within the owner-approved roadmap |
| Current product version | `0.5.0`                                 |
| Product maturity        | Alpha                                   |

---

# Responsibility of this file

This file must:

* hold prioritized candidate work;
* hold technical debt;
* expose dependencies;
* expose readiness;
* identify blocked external milestones;
* authorize bounded `direct_codex` work where appropriate;
* identify work requiring a `planned_codex` task contract.

This file must NOT:

* contain the active task implementation plan;
* duplicate `.ai/HANDOFF.md`;
* act as a historical changelog;
* contain detailed architecture rationale;
* turn speculative roadmap ideas into active tasks;
* preserve obsolete engineering workflows as active backlog items.

---

# Status values

Valid statuses:

```text id="k8s8fv"
candidate
needs_design
planned
active
blocked
done
deferred
```

Definitions:

### `candidate`

Captured work that has not yet been sufficiently refined.

### `needs_design`

Requires:

* architecture analysis;
* ADR;
* product decision;
* external research;
* security analysis;
* or another prerequisite

before a task contract can be safely written.

### `planned`

Sufficiently understood and prioritized.

A planned item may be promoted into `.ai/HANDOFF.md`.

### `active`

Represented by the current active `.ai/HANDOFF.md` contract.

Only one planned task per worktree may normally be active.

### `blocked`

Cannot responsibly progress until a dependency, external requirement, or owner decision is resolved.

### `done`

Completed and recorded in `.ai/CHANGELOG.md`.

### `deferred`

Intentionally postponed.

---

# Current development direction

The current owner-approved internal product direction is:

```text id="p9s9o8"
Connected Personal Workspace
```

The goal is to extend Aether from a strong local-first workspace into a connected personal operating environment while preserving:

* local-first ownership;
* explicit permissions;
* Rust/native trust boundaries;
* provider-agnostic core domains;
* deterministic product behavior;
* bounded AI context;
* Safe Actions for meaningful mutations;
* offline usefulness.

The first connected domains should focus on real personal utility:

1. Integration Core
2. Connections
3. Calendar
4. MyTimetable
5. Brightspace
6. School Space improvements
7. Pulse calendar/deadline relevance
8. GitHub
9. Automation / n8n
10. additional integrations only after the core pattern is proven

This section defines direction, not automatic implementation authorization.

---

# Priority queue

| Priority | ID                 | Work item                                          | Type                               | Status         | Dependencies                                          | Acceptance summary                                                                                                                                                       |
| -------- | ------------------ | -------------------------------------------------- | ---------------------------------- | -------------- | ----------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| P0       | `INT-CORE-001`     | Build Integration Core foundation                  | Product / Architecture             | `done`         | Existing Rust/SQLite/IPC architecture                 | Generic provider-neutral integration records, local persistence, typed IPC, and tests are complete; no provider implementation was added. |
| P0       | `INT-CONN-001`     | Build Connections management UI                    | Product / UX                       | `candidate`    | `INT-CORE-001`                                        | Users can inspect supported connections, connection state, last sync, errors, connect/disconnect actions, and provider capabilities without fake connected states.       |
| P0       | `CAL-CORE-001`     | Build external Calendar domain                     | Product / Data                     | `candidate`    | `INT-CORE-001`                                        | Aether can normalize, persist, query, update, and deduplicate external calendar events independently of provider-specific schemas.                                       |
| P0       | `SCHOOL-MTT-001`   | Connect MyTimetable schedule data                  | Product / Integration              | `candidate`    | `CAL-CORE-001`                                        | A user-configured MyTimetable calendar feed imports lessons, times, rooms, updates, and source provenance safely and idempotently.                                       |
| P0       | `SCHOOL-BSP-001`   | Connect Brightspace calendar/deadline data         | Product / Integration              | `needs_design` | `CAL-CORE-001`, provider capability research          | Brightspace deadlines/calendar data can be synchronized without scraping credentials or pretending unsupported API access exists.                                        |
| P0       | `SCHOOL-SPACE-001` | Upgrade School Space for connected study data      | Product / UX                       | `candidate`    | `SCHOOL-MTT-001`, `SCHOOL-BSP-001`                    | School Spaces can surface upcoming lessons, deadlines, courses, and relevant Aether Tasks through existing Space architecture.                                           |
| P0       | `PULSE-003`        | Add connected schedule/deadline relevance to Pulse | Product / UX / Data                | `candidate`    | `CAL-CORE-001`, at least one working school connector | Pulse surfaces today's real schedule and upcoming deadlines through deterministic local read models without provider-specific UI coupling or hidden AI.                  |
| P1       | `AI-CAL-001`       | Add bounded AI calendar/school read tools          | Product / AI / Security            | `needs_design` | Calendar + school read models                         | Aether can answer bounded questions such as “what do I have tomorrow?” using explicit local tool results instead of sending unrestricted database context.               |
| P1       | `ACTION-CAL-001`   | Add Safe Actions for approved scheduling mutations | Product / Security / AI            | `needs_design` | `AI-CAL-001`, calendar mutation design                | Aether may propose a study/task scheduling change, show its consequence, require approval, execute once, and audit the result.                                           |
| P1       | `GITHUB-INT-001`   | Add GitHub development integration                 | Product / Integration              | `candidate`    | `INT-CORE-001`                                        | Users can connect GitHub and surface bounded repository, issue, PR, build, and activity information without making Aether dependent on GitHub.                           |
| P1       | `AUTO-CORE-001`    | Define Automation integration boundary             | Product / Architecture             | `needs_design` | `INT-CORE-001`, Safe Actions                          | Aether can trigger and receive external workflow events through a generic automation boundary without embedding workflow-engine logic throughout the app.                |
| P1       | `N8N-001`          | Add optional n8n workflow connector                | Product / Integration / Automation | `candidate`    | `AUTO-CORE-001`                                       | Aether can invoke approved n8n workflows, read bounded execution status, and receive trusted callback events while remaining fully usable without n8n.                   |
| P2       | `RFWS-001`         | Add RF Webstudio business Space/dashboard          | Product                            | `deferred`     | Integration patterns proven                           | Leads, clients, projects, tasks, and business status can be represented through reusable Aether domains instead of bespoke one-off architecture.                         |
| P2       | `EMAIL-001`        | Evaluate email integration                         | Product / Integration / Privacy    | `deferred`     | Integration architecture proven                       | A documented safe scope exists for email metadata/action extraction without unrestricted mailbox access or automatic outbound messages.                                  |
| P2       | `FIN-001`          | Evaluate personal finance module                   | Product / Privacy / Data           | `deferred`     | Owner-approved finance scope                          | Finance information can remain local-first, explicit, and separate from unrelated Aether context.                                                                        |

---

# Next recommended planned task

The next planned task should be:

```text id="da9szx"
INT-CORE-001 — Build Integration Core foundation
```

Recommended scope:

* integration domain model;
* provider identifier;
* capability model;
* local connection status;
* last sync metadata;
* error state;
* provider-neutral repository;
* SQLite migration;
* Rust repository/service layer;
* typed Tauri IPC;
* TypeScript types/wrappers;
* tests;
* required architecture documentation.

Explicitly out of scope for this task:

```text id="wbr73c"
MyTimetable
Brightspace
Google Calendar
GitHub
n8n
OAuth
Pulse changes
School UI
AI tools
Safe Actions expansion
```

`INT-CORE-001` must receive its own ready `.ai/HANDOFF.md` contract before production implementation.

---

# Completed foundation

The following major product work is already complete and should not remain mixed into the active priority queue.

## Core product

| ID        | Outcome                 | Status |
| --------- | ----------------------- | ------ |
| `PHASE0`  | Foundation              | `done` |
| `PHASE1`  | Shell and design system | `done` |
| `PHASE2`  | Local persistence       | `done` |
| `PHASE3`  | Spaces                  | `done` |
| `PHASE4`  | Notes                   | `done` |
| `PHASE5`  | Tasks                   | `done` |
| `PHASE6`  | Vault MVP               | `done` |
| `PHASE7`  | AI integration MVP      | `done` |
| `PHASE8`  | Memory MVP              | `done` |
| `PHASE9`  | Native desktop          | `done` |
| `PHASE10` | Release quality         | `done` |

---

## Product evolution

| ID                   | Outcome                                     | Status |
| -------------------- | ------------------------------------------- | ------ |
| `CTX-001`            | Explicit Sources and safe metadata indexing | `done` |
| `SEARCH-001`         | Universal Search                            | `done` |
| `CONT-001`           | Continuity and meaningful Activity          | `done` |
| `PULSE-002`          | Pulse 2.0                                   | `done` |
| `ACTION-001`         | Safe Actions                                | `done` |
| `AI-EVOL-001`        | AI provider evolution and approved drafts   | `done` |
| `BACKUP-RESTORE-001` | Portable backup and safe restore            | `done` |
| `RELEASE-TRUST-001`  | Trusted Windows release/update architecture | `done` |
| `ONBOARD-001`        | Upgrade-safe onboarding                     | `done` |

Detailed historical completion evidence belongs in `.ai/CHANGELOG.md`.

---

# External / commercial milestones

These milestones remain valid but must not dominate normal internal feature development.

| Priority | ID           | Work item                     | Status    | Dependencies / blocker                                                                       |
| -------- | ------------ | ----------------------------- | --------- | -------------------------------------------------------------------------------------------- |
| P2       | `BETA-001`   | Operate canonical Public Beta | `blocked` | Requires owner signing/publication and meaningful real tester evidence.                      |
| P3       | `COMM-001`   | Commercial readiness          | `blocked` | Requires beta evidence plus legal/business/provider/infrastructure owner decisions.          |
| P3       | `LAUNCH-001` | Aether 1.0 launch gate        | `blocked` | Requires real eligible retention/reliability/support evidence and owner-approved thresholds. |

These are external progression gates.

They do not block unrelated local product development.

Do not repeatedly select these as the next implementation task unless their external prerequisites become available.

---

# Historical process decisions

Historical process work should not appear as active engineering backlog.

The following are complete historical records:

```text id="3v5mbv"
PROC-003
M-CODEX-ONLY
```

Any superseded multi-agent or historical collaboration workflow belongs in:

* `.ai/CHANGELOG.md`;
* relevant historical ADRs.

It must not be treated as current backlog or active architecture.

---

# Technical debt

Only unresolved technical debt belongs in this section.

| Priority | ID         | Area              | Status      | Description                                                                                                                                          | Evidence / next action                              |
| -------- | ---------- | ----------------- | ----------- | ---------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------- |
| P2       | `DEBT-003` | Architecture docs | `candidate` | Verify whether `docs/architecture.md` still contains obsolete duplicated bridge descriptions or other stale structure after later architecture work. | Perform focused documentation audit before editing. |

Resolved debt should not remain in the active debt queue.

Previously resolved items belong in `.ai/CHANGELOG.md`, including:

* frontend regression coverage gaps;
* lint warnings;
* Cargo repository metadata;
* Rust formatting;
* strict Clippy cleanup.

---

# Feature status index

| Feature                     | Status                   | Next planning action                                                                    |
| --------------------------- | ------------------------ | --------------------------------------------------------------------------------------- |
| Spaces                      | `mvp_complete`           | Extend only when a connected/product domain requires additional Space behavior.         |
| Notes                       | `mvp_complete`           | Extend only when a later feature requires additional Note capabilities.                 |
| Tasks                       | `mvp_complete`           | Reuse for deadlines, planning, and approved external-to-local task creation.            |
| Vault                       | `mvp_complete`           | Extend only through separately reviewed preview/indexing/security work.                 |
| AI                          | `mvp_complete`           | Extend through bounded tools and Safe Actions rather than unrestricted database access. |
| Memory                      | `mvp_complete`           | Preserve explicit user control; automatic suggestion requires separate consent design.  |
| Native desktop              | `trusted_delivery_ready` | Continue using existing trusted release/update architecture.                            |
| Backup / restore            | `complete`               | Extend only through separately reviewed encryption/scheduling/cloud work.               |
| Context engine              | `continuity_complete`    | Preserve deterministic local relevance and explicit context boundaries.                 |
| Integration Core            | `complete`               | Provider-neutral persistence and typed IPC are available; provider work remains separate. |
| Connections                 | `not_started`            | Build after Integration Core.                                                           |
| Calendar                    | `not_started`            | Build normalized external-event domain after Integration Core.                          |
| MyTimetable                 | `not_started`            | Implement after Calendar Core.                                                          |
| Brightspace                 | `needs_design`           | Determine supported calendar/API path before implementation.                            |
| School connected experience | `not_started`            | Build after schedule/deadline sources exist.                                            |
| GitHub                      | `not_started`            | Build after generic Integration Core is proven.                                         |
| Automation / n8n            | `needs_design`           | Define generic automation boundary before provider implementation.                      |

---

# Prioritization rules

Codex may prioritize work inside the owner-approved roadmap.

Use these rules:

1. Prefer prerequisites over dependent UI.
2. Prefer reusable core domains over one-off provider logic.
3. Prefer read-only integrations before mutation-capable integrations.
4. Prefer deterministic local data models before AI features.
5. Prefer one working provider before adding many providers.
6. Preserve existing architecture rather than introducing a second parallel system.
7. Do not prioritize blocked commercial/launch milestones over usable internal product progress unless their external blockers are resolved.
8. Do not automatically implement deferred roadmap ideas.
9. Material product reprioritization still belongs to the owner.

---

# Backlog promotion rules

A backlog item may move:

```text id="967q3q"
candidate
→ needs_design
→ planned
→ active
→ done
```

Not every task requires every state.

Example:

```text id="u9ifm5"
INT-CORE-001
planned
→ HANDOFF ready
→ active
→ done
```

For `planned_codex` work:

```text id="wb6np4"
TODO item
→ bounded HANDOFF contract
→ readiness gate
→ implementation
→ verification
→ review
→ CHANGELOG
→ TODO done
```

---

# New item template

Use:

```markdown id="n1lvnm"
| P0-P3 | `AREA-NNN` | <imperative work item> | Product / Defect / Security / Tooling / Docs / Process | `candidate` | <IDs or None> | <observable outcome> |
```

New items require:

* stable ID;
* realistic priority;
* dependency chain;
* status;
* observable acceptance summary.

Do not add speculative low-value backlog entries merely because an idea is possible.

---

# Backlog hygiene

Maintain this file as a queue, not a museum.

When an item is completed:

1. record durable completion evidence in `.ai/CHANGELOG.md`;
2. update `.ai/PROJECT_STATE.md` if current product state changed;
3. mark/remove the completed backlog entry as appropriate;
4. keep only enough completion state here to understand major dependencies.

Avoid hundreds of historical `done` rows.

Historical details belong in the changelog.

Keep the top of this file focused on what Aether should actually work on next.
