# Codex Task Contract

## Contract metadata

| Field             | Value                                                   |
| ----------------- | ------------------------------------------------------- |
| Schema version    | 3                                                       |
| Task ID           | `FINAL-RELEASE-PASS-001`                                |
| Status            | `complete`                                              |
| Owner             | Codex                                                   |
| Last updated      | 2026-09-27                                              |
| Related milestone | Aether 35 — final integrated roadmap validation         |
| Classification    | `planned_codex`                                         |
| Branch / worktree | `agent/final-release-pass` / `A:\Aether Desktop`       |

## Objective

Validate Aether 0.5.0 as one integrated Windows desktop product across fresh and
upgrade persistence, MyTimetable, School, Pulse, Tasks, Notes, Search,
Continuity, AI routing/tools/disclosure/cancellation, Safe Actions, security,
offline/error behavior, accessibility, packaging, dependencies, and product
documentation. Fix only defects verified by evidence and leave a truthful,
release-ready branch with an executable owner smoke checklist.

## Success criteria

- [x] Fresh schema and ordered migrations succeed without duplicate versions or active Brightspace behavior.
- [x] Supported persisted-state upgrade and restart paths preserve intended Integration, School, Pulse, AI, Task, Note, Search, and Continuity behavior.
- [x] MyTimetable and School source/group isolation, synchronization state, event projections, and removal/disable behavior pass available automated and manual validation.
- [x] Pulse remains local, bounded, truthful under empty/fresh/stale/disabled/disconnected/degraded states, and exposes no credentials, URLs, or inferred academic deadlines.
- [x] Tasks, Notes, Search, Continuity, and Safe Actions pass relevant regression validation without scope expansion.
- [x] Ordinary and tool-enabled AI retain immutable routing, bounded read-only tools, disclosure-before-cloud-send, cancellation, safe provenance, and hard loop/result budgets.
- [x] Security/privacy inspection finds no secret or sensitive-payload exposure across IPC, errors, logs, projections, provenance, backups, or UI state.
- [x] Key light/dark desktop flows and accessibility basics have no verified release-blocking regression.
- [x] Frozen install, dependency audits, full frontend/Rust quality gates, production build, desktop bundle, and release-configuration checks pass as far as available signing credentials permit.
- [x] Current documentation and project records match implementation truth; the compact owner smoke checklist is ready.
- [x] No verified release blocker remains; task-owned changes are committed, pushed, and represented by a draft PR.

## In scope / allowed domains

- Read-only inspection and validation across the repository, release configuration, tests, documentation, and generated release outputs.
- The smallest focused source/test/documentation correction required for a verified crash, leakage, authorization, persistence, routing, approval, cancellation, migration, build, packaging, core-feature, accessibility, or serious UX defect.
- Current control records in `.ai/**`, current release documentation, and a compact owner smoke checklist.
- Task-owned paths identified by evidence during validation; unrelated or user-owned changes remain untouched.

## Out of scope

- Finance, mobile, local LLM implementation, new integrations, academic Deadline/Assignment/Course domains, or new automation capabilities.
- New AI write tools, Calendar/Task mutation tools, Safe Actions redesign, inferred academic deadlines, Brightspace restoration, or provider expansion.
- Broad dependency upgrades, architecture redesign, opportunistic refactoring, speculative hardening, or visual redesign.
- Merging the draft PR, signing/publishing a public release without owner credentials, or claiming interactive/provider evidence that was not executed.

## Dependencies and verified baseline

- `origin/master` commit `e67bb2e` merges PR #69 (`AI-TOOL-ROUTER-001`); its Windows quality gate passed.
- Branch `agent/final-release-pass` is fast-forwarded to that merged baseline and began clean.
- Completed Integration Core, MyTimetable, School, Calendar/CAL-ICS, Sync Runtime, Pulse, Tasks, Notes, Search, Continuity, Vault, AI Router/providers/native tools/tool loop, Safe Actions, backup, and trusted-release foundations.
- Local Windows toolchain, pnpm lockfile, Rust lockfile, configured Tauri bundlers, and any available live credentials/state for bounded smoke validation.

## Internal defect checklist

| Class | Release-pass rule |
| --- | --- |
| Release blocker | Fix only reproduced startup, migration, persistence, core-flow, build, bundle, or package failures. |
| Correctness bug | Fix only behavior that contradicts current domain contracts or acceptance criteria. |
| Security/privacy bug | Treat authorization crossing, premature disclosure, secret exposure, route drift, hidden writes, or unsafe backup/log/IPC payloads as blocking. |
| UX regression | Fix only reproducible broken/clipped/dead/contradictory/inaccessible production interactions. |
| Documentation drift | Correct current-state and release instructions; preserve clearly historical records. |
| Non-blocking follow-up | Record without implementation when it is an idea, unsupported external smoke, or improvement outside the approved pass. |

## Architecture constraints

- Preserve frontend → typed IPC → Tauri commands → Rust services/repositories → SQLite/filesystem/network boundaries.
- Preserve append-only migrations, native credential ownership, source-scoped School authorization, local deterministic Pulse, immutable AI routes, Rust-owned tool scope/budgets/disclosure, and Safe Actions approval.
- Do not add a durable architecture decision unless a verified release blocker makes it unavoidable; create/update an ADR before such a change.
- Historical migrations and historical documentation remain unchanged except for a proven correctness requirement.

## Risks and safeguards

- **False confidence:** distinguish automated, structural, interactive, credential-dependent, signing-dependent, and unexecuted evidence.
- **Scope expansion:** require a reproduced defect and the smallest correction before source edits.
- **Sensitive state:** use isolated/temp databases and sanitized output; never inspect or publish user secrets or personal production data.
- **Destructive persistence testing:** do not mutate the owner's real application database; use test fixtures, temporary profiles, or owner-driven smoke steps.
- **Release artifact claims:** report unsigned/configuration-only limits explicitly and do not fabricate signing or update-channel success.
- **Cross-domain conflicts:** keep one owner and sequential edits in this worktree; reclassify/update this contract if a discovered fix changes architecture or migrations.

## Rollback considerations

Validation-only outputs are disposable. Any verified fix must remain a focused commit
that can be reverted independently. No destructive database rewrite or irreversible
external release action is authorized.

## Required validation

- Focused regression tests for every verified correction.
- Fresh-install and available upgrade/migration fixtures; migration ordering/uniqueness inspection.
- `pnpm install --frozen-lockfile`, `pnpm check`, `pnpm build`, and `pnpm audit --audit-level moderate`.
- `cargo test --manifest-path src-tauri/Cargo.toml`.
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`.
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`.
- Configured Rust advisory/dependency check when the repository/toolchain supports it.
- `pnpm tauri:build` plus release/updater/signing structural verification where secrets are unavailable.
- `git diff --check`, secret-bearing-field flow inspection, final diff review, acceptance mapping, and owner smoke checklist.

## Independent review requirement

| Field          | Value                                                                                       |
| -------------- | ------------------------------------------------------------------------------------------- |
| Required       | No                                                                                          |
| Reason         | Repository workflow requires a distinct self-review; the owner did not request delegation. |
| Reviewer scope | Final diff, scope, migrations, boundaries, security/privacy, UX, packaging, tests, and docs. |

## Human decisions / blockers

None at readiness. Missing provider credentials, signing material, or owner-only
interactive evidence will be reported as explicit validation limitations rather
than silently treated as passing.

Verified during validation: `cargo audit` identified RUSTSEC-2026-0285 in the
locked `rustls 0.23.42`. A compatible lockfile-only update to `0.23.45` or newer
is authorized as the smallest release-blocking security correction; no dependency
or architecture boundary changes.

## Worktree / readiness gate

- Required control documents and the pasted task were read.
- `origin/master` was refreshed and confirmed to contain merged AI tool-router work.
- The requested branch exists at the merged master baseline and the worktree is clean apart from this task contract update.
- No user-owned or parallel changes conflict; one worktree owns the cross-domain release pass sequentially.
- Stable goal, scope, exclusions, risks, validation, rollback, and stop condition are defined; no new ADR is required for validation-only work.
- Readiness gate passed; release validation and evidence-driven fixes may proceed.

## Stop condition

Stop after all available required validation is executed, every verified release
blocker is fixed or explicitly reported, self-review and records are complete,
task-owned changes are committed and pushed, and a draft PR is open. Do not begin
any subsequent roadmap task.

## Completion evidence

- Baseline: refreshed `origin/master` contains merged PR #69 and the passing AI
  tool-router Windows quality gate; this branch began at merge commit `e67bb2e`.
- Migrations: 19 IDs are unique and contiguous from `001_init` through
  `019_ai_router_provenance`; fresh, idempotent, legacy, Integration, School,
  Calendar-group, validator/generation, AI-provenance, and rollback fixtures pass.
- Domain coverage: 247/247 Rust tests cover MyTimetable lifecycle/offline/failure,
  Integration scheduling, School source/group isolation and restart, Calendar/ICS,
  Pulse bounds/degradation/privacy, Tasks, Notes, Search, Continuity, backup,
  updater approval, Safe Actions, AI routes/tools/disclosure/cancellation/budgets,
  and retired Brightspace behavior.
- Frontend coverage: 142/142 tests across 37 files pass. A focused regression now
  prevents Universal Search from presenting AI as DeepSeek-only.
- Security/dependencies: no secret-shaped native logging or tracked sensitive
  release file was found. `pnpm audit --audit-level moderate` reports none.
  `cargo audit` initially found RUSTSEC-2026-0285; locking `rustls 0.23.45` and
  `rustls-webpki 0.103.15` removes all vulnerabilities. Eight allowed transitive
  maintenance/yanked warnings remain and are documented as non-blocking.
- Release/build: release identity `0.5.0`, production frontend, optimized Rust,
  strict Clippy, format, and final unsigned x64 MSI/NSIS bundles pass. Protected
  updater configuration fails closed without `AETHER_UPDATER_PUBLIC_KEY`.
  Authenticode/updater signing cannot be claimed without owner trust inputs.
- UI/accessibility: local production UI inspection at 1280×800 covered light/dark
  Settings, Connections, Pulse, Spaces, Tasks, Vault, Search, AI, Actions, Memory,
  Sources, and Activity. No persistent overflow, unnamed visible interactive
  control, console error, Brightspace surface, or raw secret was observed; Search
  dismisses with Escape. Native-only/live-data behavior remains in automated
  evidence and the owner checklist.
- Self-review: the final diff is limited to the compatible TLS lock update, one
  provider-neutral copy correction plus regression, and truthful release records.
  No migration, architecture change, new dependency, new feature, write tool,
  deadline inference, route behavior, or unrelated refactor was introduced.
- Owner evidence: `docs/final-release-owner-smoke.md` records the compact live
  MyTimetable/provider/restart/cancellation/light-dark/signing acceptance pass.
- Publication: fix commit `58ae866` is pushed to
  `origin/agent/final-release-pass` with records commit `818ce50`; draft PR
  [#70](https://github.com/bimberlotDEV/Aether-Desktop/pull/70) is open.
