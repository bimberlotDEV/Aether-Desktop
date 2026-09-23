# Codex Task Contract

> Active implementation contract for the current planned Codex task.
> Completed task history belongs in `.ai/CHANGELOG.md`.

## Contract metadata

| Field | Value |
| --- | --- |
| Schema version | 3 |
| Task ID | `SCHOOL-BSP-001` |
| Status | `complete` |
| Owner | `Codex` |
| Last updated | 2026-09-23 |
| Related milestone | Connected Personal Workspace — Brightspace |
| Classification | `planned_codex` |
| Branch / worktree | `agent/brightspace` / current isolated worktree |

## Objective

Deliver a zero-admin, calendar-only Brightspace connector that validates and stores a renewable HTTPS iCalendar URL exclusively behind Aether's native encrypted credential boundary, reuses CAL-ICS and the Integration Sync Runtime, normalizes only `ExternalEvent` records, and exposes truthful setup and lifecycle controls in Settings → Connections.

## Context

`SCHOOL-BSP-CAP-001` established renewable Brightspace iCalendar subscriptions as the safe V1 capability. Integration Core, `subscribed_calendars`, CAL-ICS, Calendar Core, DPAPI storage, the closed native sync runtime, and the MyTimetable connection pattern already exist. Brightspace V1 is not an LMS API integration and must not infer richer school entities from calendar text.

## Success criteria

- Brightspace is configured with `provider_id = "brightspace"`, `auth_type = "ics_feed"`, advertised capabilities `calendar_read` and `manual_refresh`, and desktop-running manual/periodic/start/resume sync modes.
- Validation, connect, replacement, refresh, and disconnect preserve native-only bearer-secret handling and existing CAL-ICS transport protections.
- Failed validation creates no connection; secret write failure rolls back; failed replacement preserves the old credential, state, and cache; successful replacement is runtime-visible before refresh; disconnect removes the credential and cascades cached provider data.
- The closed runtime dispatch accepts only the approved Brightspace ICS tuple and normalizes Brightspace events as provider-neutral `ExternalEvent` records with `event_kind = general` unless future deterministic structured metadata is separately approved.
- Settings → Connections labels Brightspace as calendar-only, explains the renewable subscription URL, shows safe validation metadata, and supports refresh, replacement, and disconnect without displaying the URL.
- Focused and full requested validation pass, followed by self-review and a pushed draft PR.

## In scope

- Thin Brightspace ICS provider profile and native commands.
- Shared subscribed-calendar provider lifecycle extraction where it removes MyTimetable/Brightspace duplication.
- Narrow closed runtime dispatch keyed by provider plus authentication type.
- Truthful Connections setup/management UI and safe validation metadata.
- Deterministic Rust and frontend regression tests.
- Required `.ai/` completion records and draft PR publication.

## Allowed paths

- `src-tauri/src/brightspace.rs`
- `src-tauri/src/subscribed_calendar_provider.rs`
- `src-tauri/src/my_timetable.rs`
- `src-tauri/src/integration_sync.rs`
- `src-tauri/src/db/repositories/integrations.rs`
- `src-tauri/src/lib.rs`
- `src/lib/db/tauri.ts`
- `src/lib/db/types.ts`
- `src/hooks/useConnections.ts`
- `src/lib/integrations/presentation.ts`
- `src/lib/integrations/presentation.test.ts`
- `src/components/connections/ConnectionsSettings.tsx`
- `src/components/connections/ConnectionsSettings.test.tsx`
- `.ai/HANDOFF.md`
- `.ai/TODO.md`
- `.ai/PROJECT_STATE.md`
- `.ai/SESSION_NOTES.md`
- `.ai/CHANGELOG.md`

## Out of scope

- Brightspace OAuth, username/password scraping, LMS APIs, Course, Assignment, Deadline, content, or material entities.
- Title/description/URL/name heuristics that infer richer school semantics.
- Pulse, AI tools, Safe Actions, School Space redesign, or unrelated provider work.
- An unrestricted plugin runtime, global sync semantic changes, or a new background service.

## Architecture constraints

- Preserve the existing React → typed invoke → Tauri command → native services/repositories → SQLite/DPAPI boundary.
- Reuse ADR-027 through ADR-030; provider code must add no independent fetcher, parser, scheduler, reconciliation engine, or secret store.
- The full URL is accepted only by dedicated native configuration commands and is never returned through IPC, logs, backups, normal diagnostics, Integration records, subscription records, or events.
- Runtime routing remains a closed allowlist and checks both provider ID and `ics_feed` auth type.
- Authoritative reconciliation occurs only from a complete valid CAL-ICS snapshot; failure and partial paths preserve cached events.

## Dependencies

- `INT-CORE-001`, `INT-CONN-001`, `CAL-CORE-001`, `CAL-ICS-001`, `INT-SYNC-001`, and `SCHOOL-MTT-001` are complete.
- `SCHOOL-BSP-CAP-001` product research is supplied by the approved implementation brief.
- No new package or crate dependency is expected.
- No database migration is required because all persistent records are already provider-neutral.

## Risks and safeguards

- **Bearer URL exposure:** retain URL solely in encrypted native secret storage; assert redaction/no public serialization.
- **SSRF/redirect abuse:** route all validation and sync traffic through CAL-ICS without provider transport overrides.
- **Replacement data loss:** validate first, use the existing atomic credential-store behavior, and do not mutate lifecycle/cache until persistence succeeds.
- **Semantic overclaiming:** advertise only calendar read/manual refresh and normalize Brightspace data as general events.
- **Runtime broadening:** add one explicit `(brightspace, ics_feed)` registry case only.
- **Regression to MyTimetable:** preserve provider-specific structured MyTimetable metadata and rerun its shared/provider tests.

## Rollback considerations

The change is additive and requires no migration. Reverting the task commit removes Brightspace commands/UI/runtime registration. Any locally created Brightspace Integration can be disconnected before downgrade; its existing generic foreign-key cascade removes subscription and event rows, while credential cleanup remains owned by the native disconnect flow.

## Required validation

- Focused Brightspace Rust tests.
- Relevant CAL-ICS, Integration Sync Runtime, MyTimetable, and Integration repository tests.
- `cargo test --manifest-path src-tauri/Cargo.toml`
- Focused Connections/presentation frontend tests.
- `pnpm test`
- `pnpm typecheck`
- `pnpm lint`
- `pnpm build`
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`
- `git diff --check`

## Independent review requirement

| Field | Value |
| --- | --- |
| Required | `No` |
| Reason | Repository workflow requires a distinct evidence-based Codex self-review; no separate reviewer was requested. |
| Reviewer scope | Final diff, security boundary, runtime dispatch, lifecycle rollback, semantic truthfulness, UI, tests, and scope. |

## Human decisions required

None. The implementation brief resolves the product/auth/capability boundary and authorizes normal draft-PR publication.

## Blocking decisions

None.

## Worktree / ownership gate

| Check | State |
| --- | --- |
| Correct branch/worktree confirmed | Pass — `agent/brightspace` at current `origin/master` |
| `git status` inspected | Pass — clean before task records |
| User-owned changes identified | None |
| Parallel task overlap checked | Pass — current worktree owns this task |
| Serialization points identified | Closed runtime registry, native command registry, Connections UI, Integration capability transition |

## Readiness review

Ready. The objective, security model, acceptance criteria, allowed paths, dependencies, rollback, validation, and publication requirements are bounded. Existing accepted ADRs already govern the design; no new durable architecture decision or migration is required.

## Implementation log

Implemented a shared closed subscribed-calendar provider lifecycle; added the thin Brightspace profile/commands; registered only `(brightspace, ics_feed)` in the native runtime; preserved MyTimetable structured metadata; activated effective capabilities only after validated configuration; and generalized Connections setup/management with safe validation metadata and calendar-only Brightspace copy.

## Verification evidence

Pass: focused Brightspace Rust tests (4), focused MyTimetable tests (17), closed runtime registry test, focused Integration repository tests (4), focused Connections/presentation tests (16), `pnpm typecheck`, `pnpm lint`, `pnpm test` (37 files / 131 tests), `pnpm build`, `cargo fmt --check`, strict Clippy, `cargo test` (163 tests), and `git diff --check`.

## Acceptance evidence

- Native lifecycle tests prove failed validation/secret writes create no connection, failed replacement retains the old credential and cache, successful replacement is runtime-visible before refresh, disconnect removes the credential/cascades data, and connected state precedes runtime work.
- CAL-ICS and shared runtime tests cover HTTPS/SSRF/DNS/redirect/size/error/304, stable identity, reconciliation, cancellation/removal/reappearance, failure preservation, and single-flight behavior for the reused path.
- Brightspace profile and normalization tests prove the exact capability set and that LMS-like text remains a `general` ExternalEvent without inferred entities.
- Frontend tests prove setup, validation metadata, calendar-only copy, connected controls, replacement, refresh, disconnect, validation state, and URL/error redaction.
- No schema migration and no new dependency were required.

## Self-review

Pass. Final diff is limited to the approved native provider/runtime/Integration seams, typed Connections boundary/UI/tests, and control records. The URL remains native-only and is never serialized or logged; dispatch is a closed provider/auth tuple; MyTimetable behavior remains covered; no OAuth, LMS entities, heuristics, Pulse, AI, or unrelated refactor entered scope. One write-only formatting command briefly touched unrelated frontend files; every formatter-only change was identified and reverted before validation and review.

## Publication state

| Field | Value |
| --- | --- |
| Commit | `a9347c80db5b1584ad785fd6a552e8826614b81e` |
| Remote branch | `origin/agent/brightspace` |
| Draft PR | `#59` — https://github.com/bimberlotDEV/Aether-Desktop/pull/59 |
| Exact-head CI | Windows quality gate started; completion not required to open the draft PR. |
