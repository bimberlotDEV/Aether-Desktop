# Codex Task Contract

## Contract metadata

| Field | Value |
| --- | --- |
| Schema version | 3 |
| Task ID | `INT-CORE-001` |
| Status | `in_progress` |
| Owner | Codex |
| Last updated | 2026-09-21 |
| Related milestone | Connected Personal Workspace |
| Classification | `planned_codex` |
| Branch / worktree | `agent/integration-core` / `667b` |

## Objective

Deliver a provider-neutral, local-first Integration Core that persists connection and synchronization metadata behind Aether's existing typed Rust IPC boundary, without implementing a provider or exposing credentials to React.

## Context

- `INT-CORE-001` is the approved first prerequisite in `.ai/TODO.md`.
- SQLite repositories and typed Tauri commands are established; DPAPI-protected values live in Rust's `secrets` table.
- No generic integration model currently exists. Future providers must normalize their data before presentation domains consume it.

## Success criteria

- [ ] A local integration record persists a provider ID, enabled state, supported capabilities, authentication method, sync configuration, lifecycle timestamps, connection state, sync state, and a bounded error state.
- [ ] Credential material is never stored in an integration row or returned through IPC; the record exposes only a Rust-managed opaque credential reference and whether it is present.
- [ ] Rust repository operations and typed Tauri commands create, read, list, and update the generic records with validation.
- [ ] TypeScript exports strict provider-neutral schemas and invokes only typed integration commands.
- [ ] Fresh and upgrade migrations, repository behavior, and credential-boundary behavior have focused tests.
- [ ] The ADR, database documentation, and project state accurately describe the implemented boundary.

## In scope

- Integration domain model and validation
- append-only SQLite migration and repository
- minimal native credential-reference service
- typed Tauri commands and registration
- TypeScript schemas and invoke wrappers
- focused Rust and frontend wrapper tests
- ADR, database documentation, and `.ai` state

## Allowed paths

- `src-tauri/src/db/migrations.rs`
- `src-tauri/src/db/repositories/integrations.rs`
- `src-tauri/src/db/repositories/mod.rs`
- `src-tauri/src/integrations.rs`
- `src-tauri/src/commands.rs`
- `src-tauri/src/lib.rs`
- `src/lib/db/types.ts`
- `src/lib/db/tauri.ts`
- `src/lib/db/tauri.test.ts`
- `docs/database.md`
- `docs/decisions/027-integration-core.md`
- `.ai/ARCHITECTURE.md`, `.ai/PROJECT_STATE.md`, `.ai/TODO.md`, `.ai/CHANGELOG.md`, `.ai/SESSION_NOTES.md`, `.ai/HANDOFF.md`

## Out of scope

- All provider implementations, including MyTimetable, Brightspace, GitHub, calendar providers, and n8n
- OAuth, API-token entry flows, ICS parsing, polling execution, webhooks, or automation execution
- Connections, Calendar, School, Pulse, or Settings UI work
- Safe Actions and AI tool changes
- cloud infrastructure and unrelated refactors

## Architecture constraints

- Preserve React → hooks/stores → typed invoke wrappers → commands → Rust services/repositories → SQLite/native services.
- SQLite remains local source of truth; no frontend SQL or provider-specific payload in generic types.
- Credentials remain DPAPI-encrypted native secrets and must never cross IPC, logs, backup exports, or integration rows.
- Migrations are append-only; new rows must be safely ignored by existing product paths.

## Dependencies

- Existing Rust/SQLite/IPC architecture and ADR-006 credential storage.
- ADR-027 accepted before production implementation.
- No external account, credential, provider API, or owner decision is required.

## Risks and safeguards

- **Credential leakage:** use an opaque namespaced secret reference; return only presence status through IPC and test that no secret field exists in DTOs.
- **Provider lock-in:** restrict the core to stable provider IDs, capabilities, authentication and sync modes; retain opaque bounded JSON only for provider-neutral configuration.
- **Migration regression:** append one transactional migration and test fresh schema plus immediately preceding-schema upgrade.
- **Unbounded error/config storage:** validate JSON/config and cap retained sync error text.

## Rollback considerations

- Code can be reverted on this branch. The append-only schema migration must remain after release; older UI paths do not read the new table, and new records have no side effects or external writes.
- The credential reference is metadata only; deleting an integration is intentionally not part of this task, so no secret deletion lifecycle is introduced.

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
| Reason | The task uses established DPAPI and migration patterns; a formal self-review covers the new durable boundary. |
| Reviewer scope | N/A |

## Human decisions required

None.

## Blocking decisions

None.

## Worktree / ownership gate

| Check | State |
| --- | --- |
| Correct branch/worktree confirmed | `Pass` — rebased on `origin/master` at `3b37d43` |
| `git status` inspected | `Pass` — clean before task edits |
| User-owned changes identified | `Pass` — none |
| Parallel task overlap checked | `Pass` — no parallel work declared; integration migration and IPC are exclusively owned by this task |
| Serialization points identified | `Pass` — migrations, central IPC registration, credential namespace, architecture registry |

## Readiness review

All readiness requirements pass: the task ID, bounded scope, measurable criteria, allowed paths, constraints, risks, rollback, validation, ADR, and ownership boundary are documented. No unresolved human decision exists.

## Implementation log

2026-09-20

- Rebasing confirmed this worktree is based on current `origin/master` (`3b37d43`).
- ADR-027 records the provider-neutral persisted record and native-only credential-reference boundary.

## Verification evidence

Pending implementation.

## Acceptance evidence

Pending implementation.

## Self-review

Pending implementation.

## Independent review findings

Not required.

## Completion evidence

Pending implementation.

## Publication state

| Field | Value |
| --- | --- |
| Commit | Pending |
| Remote branch | Pending |
| Draft PR | Pending |
| Exact-head CI | Pending |

## Next task

`INT-CONN-001` may be planned only after this task is completed and published.
