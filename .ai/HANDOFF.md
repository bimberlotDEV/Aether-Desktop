# Codex Task Contract

> Active implementation contract for the current planned Codex task.
> Completed task history belongs in `.ai/CHANGELOG.md`.

## Contract metadata

| Field | Value |
| --- | --- |
| Schema version | 3 |
| Task ID | `CAL-ICS-SEC-001` + `CAL-ICS-REDIRECT-001` |
| Status | `complete` |
| Owner | Codex |
| Last updated | 2026-09-24 |
| Related milestone | Aether 24 — shared Calendar ICS security hardening |
| Classification | `planned_codex` |
| Branch / worktree | `agent/cal-ics-security` |

## Objective

Bound aggregate iCalendar parsing/recurrence work before excessive occurrence allocation and ensure HTTP conditional validators never cross request origins during redirects.

## Context

The Aether 22 security review found that CAL-ICS limits each recurrence but checks the feed-wide result only after expansion, and that ETag/Last-Modified values are currently sent on every validated redirect hop. MyTimetable and the draft Brightspace connector both reuse this shared native layer, so the correction belongs in CAL-ICS rather than provider code.

## Success criteria

- A centralized feed budget rejects before appending occurrence 2,001 and bounds recurrence expansion using the remaining aggregate allowance.
- Ordinary events, RRULE/RDATE occurrences, and recurrence overrides all consume the same aggregate budget while the existing ±366-day horizon and per-series ceiling remain intact.
- RDATE, EXDATE, repeated recurrence properties, per-component property count, and copied untrusted field lengths have explicit deterministic limits with sanitized error codes.
- Parse/budget failure produces no authoritative reconciliation, preserves cached events, and records a sanitized failed sync.
- Conditional validators have a persisted private origin association; legacy unassociated validators are cleared on upgrade.
- Same-origin redirects may retain validators; after an origin change the chain sends neither conditional header, including if a later hop returns to the original origin.
- Redirect DNS/SSRF, HTTPS, pinned-IP, userinfo, and hop-limit protections remain intact.
- Same-origin/direct 304 behavior remains successful; an unconditioned cross-origin 304 cannot be mistaken for a valid no-change response.
- Final-response validators replace prior validators together with the normalized final response origin.
- Focused adversarial and regression tests plus the required repository validation pass.

## In scope

- Shared CAL-ICS parsing, recurrence expansion, transport, and tests.
- Native Integration Sync plumbing needed to supply and persist validator origin.
- Append-only SQLite migration and repository behavior for private validator-origin metadata.
- Existing database and ADR-029 documentation.
- Current task state, verification, and completion records.

## Allowed paths

- `src-tauri/src/calendar_ics.rs`
- `src-tauri/src/integration_sync.rs`
- `src-tauri/src/db/migrations.rs`
- `src-tauri/src/db/repositories/integrations.rs`
- `src-tauri/src/diagnostics.rs` only for the latest-schema test expectation
- `src-tauri/src/my_timetable.rs` only if shared signature/regression tests require mechanical updates
- `docs/database.md`
- `docs/decisions/029-ics-subscription-ingestion.md`
- `.ai/HANDOFF.md`
- `.ai/TODO.md`
- `.ai/PROJECT_STATE.md`
- `.ai/SESSION_NOTES.md`
- `.ai/CHANGELOG.md`

## Out of scope

- Generation-safe feed replacement / `CAL-SUB-ROTATE-001`.
- Integration Sync scheduling / `INT-SYNC-002`.
- School Space source association / `SCHOOL-SCOPE-002`.
- Changes to the merged Brightspace provider behavior beyond compatibility with the shared hardened calendar layer.
- Pulse, AI, frontend, IPC, or provider-specific limit/redirect behavior.
- Broad async-runtime or threading redesign.

## Architecture constraints

- Preserve the native trust boundary and provider-neutral CAL-ICS ownership.
- Preserve authoritative reconciliation as an atomic post-parse commit only.
- Use one occurrence-budget helper and one redirect/request-policy path.
- Do not expose validator origin through public Integration serialization or IPC.
- Keep migrations append-only and existing cached events intact.

## Dependencies

- Merged `CAL-ICS-001`, `INT-SYNC-001`, `SCHOOL-MTT-001`, and `SCHOOL-SPACE-001` on `origin/master`.
- Existing `ical`, `rrule`, `reqwest`, SQLite, and Calendar Core infrastructure.

No new dependency is required.

## Risks and safeguards

- **False rejection of normal feeds:** limits align with downstream Calendar Core field limits and existing 2,000-occurrence policy; real-world MyTimetable fixtures remain regression-tested.
- **Legacy validator ambiguity:** migration clears validators whose origin cannot be proven, forcing one safe complete refresh without deleting cached events.
- **Incorrect 304:** accept 304 only when at least one validator was actually attached to that exact request.
- **Data loss on parser failure:** normalization completes before the transaction that reconciles events; failure follows the existing sanitized failure path.
- **SSRF regression:** keep per-hop URL validation, DNS resolution, public-address checks, pinned addresses, and redirect bounds unchanged and covered.

## Rollback considerations

Code can be reverted, but the additive validator-origin column remains harmless. Cleared legacy validators cause only a future full refresh. No cached event rows or credentials are migrated or deleted.

## Required validation

- Focused CAL-ICS parser/recurrence/security tests.
- Focused redirect transport tests.
- Relevant Integration Sync, subscribed-calendar, migration, and MyTimetable regression tests.
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`
- `pnpm typecheck`
- `pnpm lint`
- `pnpm build`
- `git diff --check`

Full frontend tests are required only if frontend contracts unexpectedly change; no frontend change is planned.

## Independent review requirement

| Field | Value |
| --- | --- |
| Required | No |
| Reason | The repository workflow requires a distinct evidence-based Codex self-review; no separate reviewer was requested. |
| Reviewer scope | Parser budgets, redirect origin isolation, persistence semantics, cache preservation, and scope discipline. |

## Human decisions required

None. The requested limits and security behavior are bounded by the existing Calendar Core and CAL-ICS architecture.

## Blocking decisions

None.

## Worktree / ownership gate

| Check | State |
| --- | --- |
| Correct branch/worktree confirmed | Pass — `agent/cal-ics-security` |
| `git status` inspected | Pass — clean before contract updates |
| User-owned changes identified | None |
| Parallel task overlap checked | Pass — merged Brightspace behavior is preserved |
| Serialization points identified | CAL-ICS, Integration validator persistence, migration ordering |

## Readiness review

Ready. The objective, boundaries, risks, persistence requirement, exact implementation surface, validation, and stop condition are explicit. Production implementation may begin.

## Implementation log

- Added a single incremental `OccurrenceBudget` shared by ordinary, recurring, RDATE, and override output; recurrence collection receives only a one-item sentinel beyond the remaining allowance.
- Added pre-parse logical-line bounds plus per-event/property, recurrence-property, RDATE, EXDATE, category, and Calendar Core-aligned field limits.
- Added origin-associated conditional validators, explicit follow-redirect statuses, cross-origin sticky header stripping, conditioned-304 enforcement, and bounded response-validator parsing.
- Added migration `016_ics_validator_origin`, private runtime repository plumbing, atomic validator replacement/preservation semantics, and legacy ICS validator clearing without cache deletion.
- Updated ADR-029, database documentation, diagnostics schema evidence, and provider/shared regression coverage. No frontend source or dependency changed.

## Verification evidence

- Focused CAL-ICS tests: 21 passed.
- Focused migration tests: 14 passed.
- Focused Integration Sync tests: 5 passed.
- Focused Integration repository tests: 5 passed.
- Focused MyTimetable tests: 18 passed.
- `cargo test --manifest-path src-tauri/Cargo.toml`: 178 passed after reconciling the merged Brightspace connector.
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`: passed.
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`: passed.
- `pnpm typecheck`: passed.
- `pnpm lint`: passed.
- `pnpm test`: 131 passed across 37 files after merge reconciliation.
- `pnpm build`: passed.
- `git diff --check`: passed.

## Acceptance evidence

- AC1: boundary/adversarial tests prove exactly 2,000 occurrences succeed, 2,001 and combined-series/RDATE overflow fail, and the output vector never grows past the limit.
- AC2: RDATE/EXDATE counts, repeated recurrence properties, 128 properties/event, 80,000-byte logical properties, and downstream-aligned field lengths fail with fixed sanitized codes.
- AC3: MyTimetable normalization and cancellation/removal/reappearance tests pass; an adversarial budget failure leaves the cached active event intact.
- AC4: transport fixtures prove same-origin preservation, cross-origin stripping of both headers, final-origin validator capture, per-hop DNS rejection, multi-origin non-reappearance, direct fetch, and conditioned 304 behavior.
- AC5: migration/repository tests prove legacy validators are cleared without cached-event loss, final validators and origin replace atomically, 304 preserves them, and origin is absent from serialized Integration IPC state.

## Self-review

Passed. The security implementation stays inside the approved shared CAL-ICS/native Integration scope plus the migration-required diagnostics expectation. No provider-specific policy, scheduler redesign, School association, frontend, Pulse, or AI work was introduced; the later merged Brightspace provider continues to reuse the same hardened shared lifecycle. Parser failures remain pre-reconciliation; validator origin remains native-only; existing HTTPS, userinfo, redirect-hop, DNS/public-address, and pinned-IP protections remain enforced. No secrets, URLs, payloads, or attacker-controlled values enter error messages.

## Publication state

| Field | Value |
| --- | --- |
| Commit | `ed0aafb` |
| Remote branch | `origin/agent/cal-ics-security` |
| Draft PR | [#60](https://github.com/bimberlotDEV/Aether-Desktop/pull/60) |
| Exact-head CI | `None` |

## Stop condition

Stop after all acceptance criteria are evidenced, required checks pass, the conflict-resolution merge commit is pushed, and the Aether 24 draft PR is ready for review. Do not begin any named follow-up task.
