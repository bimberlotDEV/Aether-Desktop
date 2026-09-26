# Codex Task Contract

## Contract metadata

| Field             | Value                                           |
| ----------------- | ----------------------------------------------- |
| Schema version    | 3                                               |
| Task ID           | `REPO-HEALTH-032`                               |
| Status            | `complete`                                      |
| Owner             | Codex                                           |
| Last updated      | 2026-09-26                                      |
| Related milestone | Aether 32 — repository health and cleanup audit |
| Classification    | `planned_codex`                                 |
| Branch / worktree | `agent/repo-health-cleanup`                     |

## Objective

Audit the merged post-PULSE-003 repository for verified dead or stale code, contract and documentation drift, unsafe or unbounded behavior, migration inconsistency, unnecessary direct dependencies, and regressions; apply only safe evidence-backed cleanup and publish a validated draft PR.

## Context

- `origin/master` at `53b790c` contains merged Brightspace retirement, AI Router phase 1, and PULSE-003.
- This is a cross-repository health task, not feature work or authorization for architectural redesign.
- Brightspace is retired; historical records may remain when clearly historical, while shared ICS infrastructure and MyTimetable must remain operational.
- The owner supplied the complete audit areas, exclusions, validation matrix, smoke plan, publication requirements, and stop condition.

## Success criteria

- [ ] All requested repository domains, ADRs, migrations, dependency manifests, trust boundaries, test structure, and current control records are inspected and findings are classified as Fix now, Safe cleanup, Follow-up, or False positive / intentional.
- [ ] Verified active Brightspace behavior and misleading fixtures/copy are absent; retained references are historical or generic/shared, and MyTimetable/shared ICS behavior remains intact.
- [ ] High-confidence dead production code, stale contracts, duplicated active logic, unsafe leakage, and practical unbounded overview reads found in scope are removed or corrected without speculative refactoring.
- [ ] Frontend/backend IPC, provider registry, Calendar/School/Pulse projections, AI routing/privacy/provenance, and local domain lifecycle boundaries remain internally consistent.
- [ ] Migration numbering, fresh initialization, supported upgrade coverage, foreign keys, and schema documentation are consistent without rewriting historical migrations.
- [ ] Clearly unused direct dependencies are removed; risky upgrades and architectural issues are recorded as follow-up rather than broadened into this task.
- [ ] Required focused and full validation passes, the final diff is self-reviewed against every acceptance criterion, and repository records accurately describe current state.
- [ ] Task-owned changes are committed and pushed on `agent/repo-health-cleanup`, and a draft PR is open.

## In scope

- Repository-wide evidence-based health inspection across Rust, React/TypeScript, migrations, dependencies, tests, docs, ADRs, and `.ai` records.
- Brightspace retirement; Integration Core/provider catalog/subscribed-calendar/MyTimetable/sync/credentials; Calendar/School/Pulse; AI routing/privacy/approval/provenance; Tasks/Continuity/Notes/Vault; frontend/Rust cleanup; security, bounds, and errors.
- Safe deletion or correction that directly satisfies the owner's cleanup rules.
- Focused regression tests required by verified fixes and concise current documentation/record corrections.

## Allowed paths

- Any tracked repository path when a verified audit finding requires a safe change and the path is recorded in the implementation log.
- Dependency lockfiles only when a direct dependency is proven unused and removed from its manifest.
- `src-tauri/src/db/migrations.rs` may be changed only by appending a new migration for a verified current schema defect; historical migrations must not be rewritten.
- `.ai/HANDOFF.md`, `.ai/PROJECT_STATE.md`, `.ai/TODO.md`, `.ai/SESSION_NOTES.md`, `.ai/CHANGELOG.md`, architecture/database/current feature docs, and ADR status annotations as required for factual consistency.

## Out of scope

- AI tools, local LLM support, finance, mobile, OAuth, new integrations/providers, new product features, academic Deadline invention, or Pulse redesign.
- Broad style-only refactors, speculative abstractions, major dependency upgrades, migration squashing/rewriting, or opportunistic adjacent work.
- Removing compatibility required for persisted user data, shared ICS infrastructure, or historical records that remain clearly marked as historical/retired.
- Live external-service requirements or destructive manipulation of owner data.

## Architecture constraints

- Preserve frontend → typed invoke wrapper → Tauri command → Rust service/repository → SQLite/filesystem/provider boundaries.
- Preserve native secret custody, minimal IPC, Safe Actions separation, explicit School connection/group authorization, provider-neutral normalized models, and deterministic local product behavior.
- Prefer deletion of proven dead code over compatibility shims, but retain persisted-data compatibility and append-only migrations.
- Every production change must map to a verified finding and use the smallest reasonable correction.
- No durable architecture boundary may change without revising this contract and adding/updating an ADR first.

## Dependencies

- Merged Aether 31 / PULSE-003 on `origin/master` at `53b790c`.
- Existing Brightspace retirement, AI Router, Integration/Calendar/School, Pulse, Safe Actions, and local-domain implementations and tests.
- No new dependency is expected.

## Risks and safeguards

- **Repository-wide scope:** maintain a categorized findings checklist and change only verified issues; do not convert audit observations into speculative rewrites.
- **Persisted compatibility:** trace callers, IPC, migrations, and upgrade tests before deleting types/commands/schema behavior.
- **Security/privacy:** inspect flows and serialized projections manually; do not infer safety from string search alone.
- **Provider regressions:** retain shared ICS and run provider registry, MyTimetable, Calendar/School, and Pulse regressions after cleanup.
- **AI regressions:** preserve native routing/privacy/provenance and run routing/cancellation/Safe Actions coverage without adding tools or local backends.
- **Documentation history:** correct current claims and mark historical decisions retired/superseded rather than rewriting history.
- **Dependency removal:** verify direct usage across code/build/scripts and rebuild both stacks before removal is accepted.

## Rollback considerations

Changes are isolated on the task branch and should be decomposed into reviewable cleanup and record commits where useful. No destructive data operation is authorized. A new migration is strongly disfavored; if unavoidable it must be append-only, independently tested for fresh and upgrade paths, and documented before publication.

## Required validation

- Focused tests for every changed production area and regressions covering verified fixes.
- Fresh database initialization and existing supported migration-upgrade tests.
- Provider registry/Brightspace-retirement sanity, MyTimetable, Pulse, and AI routing regressions.
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`
- `pnpm test`
- `pnpm typecheck`
- `pnpm lint`
- `pnpm build`
- `git diff --check`
- Owner smoke plan recorded for startup, Connections, MyTimetable, School, Pulse, Tasks, Notes, Search, cloud AI/cancellation, Safe Actions, restart persistence, disconnected sources, Brightspace absence, and console/native errors.

## Independent review requirement

| Field          | Value                                                                                                                                        |
| -------------- | -------------------------------------------------------------------------------------------------------------------------------------------- |
| Required       | No                                                                                                                                           |
| Reason         | The owner requested a full audit and the repository workflow requires a distinct evidence-based self-review, not a separate agent/session.   |
| Reviewer scope | Final diff, deletion safety, persisted compatibility, trust boundaries, bounds, migrations, dependencies, docs, tests, and scope discipline. |

## Human decisions required

None at readiness. The owner explicitly bounded permitted fixes, exclusions, validation, and publication. Any newly discovered architectural decision or destructive/persisted-data choice will block that finding and be recorded for owner direction rather than assumed.

## Blocking decisions

None.

## Worktree / ownership gate

| Check                             | State                                                                                                                                               |
| --------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------- |
| Correct branch/worktree confirmed | Pass — `agent/repo-health-cleanup`                                                                                                                  |
| Latest master confirmed           | Pass — fetched `origin/master`; HEAD equals `53b790c`, merged PULSE-003                                                                             |
| `git status` inspected            | Pass — clean before contract update; apparent `Cargo.toml` modification had an identical blob hash and was a stale stat entry                       |
| User-owned changes identified     | None                                                                                                                                                |
| Parallel task overlap checked     | Pass — latest required work is merged on master; this branch owns the repository-health audit                                                       |
| Serialization points identified   | Migrations, central IPC, provider registry/runtime, AI routing/privacy, Pulse/School read models, dependency manifests, and current control records |

## Readiness review

Passed. The task has a stable ID, exact merged base, bounded audit/fix rule, explicit exclusions, success criteria, dependencies, risks, rollback, validation, publication, and stop condition. Production audit and evidence-backed cleanup may begin.

## Findings checklist

### Fix now

- **Retired-provider leakage:** Pulse event and trust queries accepted any subscribed-calendar provider except Brightspace. They now require the supported `my_timetable` provider and explicit School bindings/groups; a legacy unsupported-provider regression proves fail-closed behavior.
- **Over-broad Integration IPC:** normal connection reads serialized configuration, validators, cursors, credential lifecycle metadata, execution scope, and database timestamps. These fields remain available to native services but are skipped at the serialization boundary and absent from the strict frontend schema.
- **Over-broad School IPC:** School schedule events reused the complete persisted ExternalEvent record. The read now selects and serializes a dedicated presentation projection only.
- **Dependency security:** Vitest 4.1.10 and its mocker had a moderate path-traversal advisory. The test-only dependency is updated to 4.1.11; `pnpm audit --audit-level moderate` reports no known vulnerabilities.
- **Current documentation drift:** README and architecture/database documentation described superseded product and boundary details. Current docs now identify connected Pulse, AI routing, retired Brightspace, supported MyTimetable/ICS behavior, minimized IPC, and backup compatibility accurately.

### Safe cleanup

- Removed 21 registered but unused Tauri commands and their TypeScript wrappers: broad settings/profile/Space/entity getters, generic Integration/ExternalEvent/subscribed-calendar creation/read/status endpoints, provider catalog/model listing, and unused conversation/context endpoints.
- Removed the now-unreachable generic ExternalEvent range read model, stale MyTimetable profile endpoint, redundant ICS validation wrapper, three repository helpers used only by removed endpoints, and their obsolete tests/schemas/imports.
- Removed unused frontend `@tauri-apps/plugin-updater`; native updater ownership remains unchanged.
- Removed a duplicate ADR-027 index row and resolved `DEBT-003` through the focused architecture-document audit.

### Follow-up

- None required by verified repository health. Product additions and architectural expansion remain governed by the existing backlog and are outside this audit.

### False positive / intentional

- The legacy native SQLite-only export remains registered despite no current UI caller because ADR-014/ADR-022 explicitly retain it for compatibility.
- Shared CAL-ICS, subscribed-calendar persistence/runtime, legacy Brightspace rows, and Brightspace regression fixtures remain required for MyTimetable operation, safe cleanup, migration history, and retirement guarantees; no active Brightspace provider path remains.
- AI privacy approval scaffolding and explicit message-deletion repository code retain documented dead-code annotations for an approved future UI boundary; this audit did not authorize AI tools or new disclosure UX.
- Core-domain list reads serve dedicated user-requested screens or low-cardinality configuration, while overview/search/Calendar/Pulse reads are bounded. No additional practical unbounded overview read was verified.
- Historical ADRs, migrations, and changelog entries preserve their original Brightspace and architecture context rather than rewriting history.

## Implementation log

- 2026-09-26: Read the owner request and mandatory control documents; fetched `origin/master`, verified the apparent `Cargo.toml` edit contained no content difference, fast-forwarded to merged PULSE-003 `53b790c`, created the clean requested branch, and completed the readiness gate.
- 2026-09-26: Audited every ADR and migration plus targeted active Rust/TypeScript domains, IPC parity, manifests, tests, security boundaries, query bounds, and current documentation.
- 2026-09-26: Minimized Integration and School serialization, closed unsupported-provider Pulse reads, deleted verified dead IPC/read-model code, removed one unused dependency, patched the Vitest advisory, and corrected current documentation/control records without adding a migration or architecture decision.
- 2026-09-26: Completed focused regressions, full frontend/native validation, dependency audit, and final scope/security/compatibility self-review.

## Verification evidence

- Focused native: Pulse 12/12, School 13/13, Integration 8/8, MyTimetable 21/21, migrations 18/18, AI routing 7/7.
- Focused frontend IPC/Connections/School: 47/47 across four files.
- Full native: 225/225; full frontend: 139/139 across 37 files.
- TypeScript typecheck, Oxlint, Vite production build, Rust formatting, strict Clippy, and moderate-level dependency audit pass.

### Owner smoke plan

Automated evidence is complete. A release-candidate desktop smoke should verify: cold startup; Connections display; MyTimetable connect/refresh/disconnect; School source/group selection and timetable views; Pulse schedule/tasks/conflicts/trust; Task lifecycle; Note create/autosave/search; Universal Search; Cloud only and Automatic AI routing; stream cancellation; Safe Action preview/approve/cancel; restart persistence; disabled/disconnected source truthfulness; absence of Brightspace setup/sync/School content; and no unexpected webview-console or native errors.

## Acceptance evidence

- Requested audit domains and every ADR/migration were inspected; findings are classified above.
- Brightspace is absent from active provider/School/Pulse behavior while cleanup, historical, migration, and regression references remain intentionally intact.
- IPC parity is exact after deletion. The sole unused TypeScript wrapper is the documented legacy backup-export compatibility path.
- No migration was required: numbering remains sequential through 019, and all 18 fresh/idempotent/supported-upgrade migration tests pass.
- No production dependency was added. One unused frontend package was removed and one vulnerable test-only package received a patch update.
- Security review confirms credentials, feed URLs, validators, cursors, hashes, provenance, external IDs, and persistence timestamps do not cross the cleaned presentation boundaries.

## Self-review

- Inspected the final diff for scope, caller deletion safety, IPC registration/wrapper parity, persistence compatibility, provider authorization, serialization minimization, migration immutability, dependency intent, and documentation accuracy.
- No unrelated feature, schema, migration, provider, AI tool, local runtime, or visual change was introduced.
- The retained compatibility/dead-code exceptions are explicit in the findings and supported by ADRs or current comments.

## Publication state

Implementation commit `578c43d` is pushed on `agent/repo-health-cleanup`; draft PR [#67](https://github.com/bimberlotDEV/Aether-Desktop/pull/67) is open. Publication records are finalized in the follow-up commit.

## Stop condition

Stop when requested audit coverage is evidenced; safe verified fixes and records are complete; all required focused/full checks pass; the repository is clean; task-owned commits are pushed on `agent/repo-health-cleanup`; and a draft PR is open. Do not begin AI tools, finance, mobile, local LLM, or another integration.
