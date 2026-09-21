# Codex Task Contract

## Contract metadata

| Field | Value |
| --- | --- |
| Schema version | 3 |
| Task ID | `CAL-CORE-001` |
| Status | `in_progress` |
| Owner | Codex |
| Last updated | 2026-09-21 |
| Related milestone | Connected Personal Workspace |
| Classification | `planned_codex` |
| Branch / worktree | `agent/calendar-core` / `a27c` |

## Objective

Add the minimum provider-neutral, local-first Calendar Core that normalizes, persists, reconciles, and boundedly reads external calendar occurrences without implementing a provider or calendar UI.

## Context

- `INT-CORE-001` is merged at `origin/master` and owns provider-neutral connection records in migration `011_integrations`.
- Calendar occurrences reference `integrations.id`; provider payloads, credentials, raw feed URLs, and secret metadata remain outside this domain and IPC.
- ADR-028 records the approved calendar normalization, identity, time, and reconciliation semantics.

## Success criteria

- [ ] Normalized external events persist with the required identity, lifecycle, provenance, content, and time fields.
- [ ] Identity is exactly `(connection_id, external_id, occurrence_id)`; repeated snapshots are idempotent and occurrence identity is independent of title/time.
- [ ] Native reconciliation only tombstones missing occurrences from a complete authoritative window; reappearance reactivates and cancellation remains distinct.
- [ ] Timed and all-day semantics are validated, including UTC instants, timezone context, exclusive all-day end dates, and half-open bounded range reads.
- [ ] React can only make strict, typed, bounded read IPC calls; no provider-owned write command or secret/provider payload leaks exist.
- [ ] Migration, repository/service behavior, IPC schemas, documentation, and required quality gates are verified.

## In scope

- ExternalEvent Rust domain, repository, reconciliation service, tests, and append-only migration.
- Bounded read-only Tauri commands and strict TypeScript/Zod read models/wrappers/tests.
- ADR-028, database/architecture documentation, and relevant `.ai` state records.

## Allowed paths

- `src-tauri/src/db/migrations.rs`
- `src-tauri/src/db/repositories/external_events.rs`
- `src-tauri/src/db/repositories/mod.rs`
- `src-tauri/src/calendar.rs`
- `src-tauri/src/diagnostics.rs`
- `src-tauri/src/commands.rs`
- `src-tauri/src/lib.rs`
- `src/lib/db/types.ts`
- `src/lib/db/tauri.ts`
- `src/lib/db/tauri.test.ts`
- `docs/decisions/028-calendar-core.md`
- `docs/database.md`
- `.ai/ARCHITECTURE.md`, `.ai/PROJECT_STATE.md`, `.ai/TODO.md`, `.ai/CHANGELOG.md`, `.ai/SESSION_NOTES.md`, `.ai/HANDOFF.md`

## Out of scope

- ICS/RFC5545 parsing, MyTimetable, Brightspace, OAuth, credentials, sync scheduling/polling/webhooks, provider connectors, Calendar/School/Pulse UI, course/assignment/deadline persistence, user-created events, and cross-provider deduplication.

## Architecture constraints

- Preserve React → typed invoke wrapper → Tauri command → Rust service/repository → SQLite.
- Persist only normalized provider-neutral data; no provider payloads, raw feed URLs, tokens, credentials, or secret metadata.
- Migrations are append-only; no frontend provider-owned event mutation APIs.
- Read IPC must be bounded; all time validation and reconciliation mutation remains native-only.

## Dependencies

- `INT-CORE-001` / migration `011_integrations` (merged at `origin/master`).
- Accepted ADR-028 for the Calendar Core boundary.

## Risks and safeguards

- **False removals from incomplete provider results** — only `Authoritative` snapshots may tombstone, in one native transaction.
- **Duplicate or moved recurring events** — unique identity uses connection/external/occurrence IDs only; upsert tests cover idempotency and moved occurrences.
- **Time corruption around all-day/DST values** — strict native validation distinguishes UTC instants from date-only values and rejects unresolved floating values.
- **Privacy leakage** — database/read model contain bounded normalized fields only; no credentials, feed URLs, or raw provider payload columns.

## Rollback considerations

- Code is reversible on the task branch. Migration `012_external_events` is append-only after release; earlier product paths safely ignore its data. Tombstones preserve history and do not hard-delete user-local records.

## Required validation

```text
pnpm typecheck
pnpm lint
pnpm test
pnpm build
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
git diff --check
```

## Independent review requirement

| Field | Value |
| --- | --- |
| Required | `No` |
| Reason | Bounded additive domain with formal self-review and complete validation. |
| Reviewer scope | None |

## Human decisions required

None — the user supplied the approved Calendar Core design.

## Blocking decisions

None.

## Worktree / ownership gate

| Check | State |
| --- | --- |
| Correct branch/worktree confirmed | `agent/calendar-core` fast-forwarded to `origin/master` (`3bd9d58`) |
| `git status` inspected | Clean |
| User-owned changes identified | None |
| Parallel task overlap checked | None; Calendar Core owns its new migration/repository/service/read IPC paths |
| Serialization points identified | Migration `012`, repository module registry, command registry, `lib.rs`, TypeScript DB contracts, ADR registry |

## Readiness review

Status: `ready`

- [x] Stable task ID, observable objective, measurable success criteria, bounded scope and paths.
- [x] Dependencies, risks, rollback, validation, and explicit non-goals recorded.
- [x] Worktree is current and clean; migration/shared IPC ownership is clear.
- [x] ADR-028 is approved by the supplied task design and will be committed with implementation.

## Implementation log

2026-09-21

- Worktree fast-forwarded from `3b37d43` to current `origin/master` `3bd9d58` before production edits.
- Readiness gate passed; implementation started.

## Verification evidence

Pending implementation.

## Acceptance evidence

Pending self-review.

## Self-review

Pending implementation.

## Publication state

| Field | Value |
| --- | --- |
| Commit | `None` |
| Remote branch | `None` |
| Draft PR | `None` |
| Exact-head CI | `None` |

## Next task

None — stop after CAL-CORE-001.
