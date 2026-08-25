# Codex Task Contract

| Field             | Value                         |
| ----------------- | ----------------------------- |
| Schema version    | 2                             |
| Task ID           | `HARD-001`                    |
| Status            | `ready`                       |
| Owner             | Codex                         |
| Last updated      | 2026-08-25                    |
| Related milestone | Milestone A — Alpha Hardening |
| Classification    | `planned_codex`               |

## Objective

Make Alpha 0.3.1 start safely on databases created by the earlier personal-beta schema line, preserving valid Tasks, Memory, Vault metadata, Space relationships, and any discoverable managed-file ownership without weakening current path protections.

## Context

The current upstream migration sequence reuses different names and table shapes than an earlier personal-beta build. On an affected database, migration `005_tasks` runs against an existing legacy `tasks` table and fails while creating `idx_tasks_parent`, causing the release executable to exit during startup. A local emergency patch proves the schema mismatch and restores startup for the owner's database, but it must be reviewed as a formal data-lifecycle change before publication.

## Acceptance criteria

- [ ] Fresh database creation still produces the current Tasks, Memory, and Vault schemas.
- [ ] Re-running migrations remains idempotent.
- [ ] A representative prior-schema database upgrades in one transaction without losing valid Task, Memory, Vault, Space, or Note rows.
- [ ] Legacy Task status, priority, due date, completion, archive, and Space ownership map deterministically to supported current values.
- [ ] Legacy Memory scope and content remain explicit and do not gain broader AI access as a side effect.
- [ ] Legacy Vault metadata never resolves outside Aether-controlled storage, never overwrites a file, and either preserves discoverable managed bytes or reports an explicit recoverable limitation.
- [ ] Failed upgrades roll back without leaving renamed, partial, or duplicate tables.
- [ ] Current repositories can read and mutate upgraded rows under their normal validation rules.
- [ ] The owner's database is backed up before installer replacement and passes SQLite integrity plus foreign-key checks afterward.
- [ ] All frontend, Rust, build, packaging, and startup gates pass.
- [ ] Self-review finds no unresolved data-loss, path-safety, secret, or scope defect.

## Allowed paths

- `src-tauri/src/db/migrations.rs`
- `src-tauri/src/db/mod.rs`
- `src-tauri/src/vault.rs`
- Closely related Rust tests or a bounded migration fixture under `src-tauri/`
- `.ai/CHANGELOG.md`
- `.ai/PROJECT_STATE.md`
- `.ai/HANDOFF.md`
- `.ai/SESSION_NOTES.md`
- `.ai/TODO.md`
- Relevant database/release documentation when verified behavior changes

## Non-goals

- Context Engine, universal search, Pulse 2.0, onboarding, or other post-hardening features.
- Replacing SQLite, the repository pattern, or the existing Tauri trust boundary.
- Weakening Vault containment or converting managed files into arbitrary linked paths.
- Deleting invalid legacy records without preserving evidence and an explicit recovery path.
- Changing public product positioning beyond recording the new master roadmap.

## Dependencies and evidence to inspect

- Current and historical migration sequences and Git history.
- Current Tasks, Memory, and Vault repository invariants.
- Historical managed-file storage behavior, if it existed in a shipped build.
- The backed-up owner database, read-only except during the final verified upgrade.
- Current backup/export limitations and release packaging behavior.

## Risks and safeguards

- **Data loss:** all schema replacement must occur inside a single SQLite transaction and be covered by populated fixtures.
- **Managed-file orphaning:** do not fabricate a current managed path unless the corresponding byte location is known and safely migrated.
- **Scope leakage:** global/Space Memory mapping must preserve original scope and remain explicit-only AI context.
- **Path traversal/symlinks:** filesystem repair, if required, stays in trusted Rust and uses canonical containment checks.
- **User worktree overlap:** only the existing `src-tauri/src/db/migrations.rs` prototype overlaps this task; all other task paths are clean at readiness.
- **Rollback:** preserve the pre-upgrade database directory backup and keep schema changes transactional; installation can be replaced with the prior installer if startup validation fails.

## Required validation

```text
pnpm check
pnpm build
pnpm audit --audit-level high
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
cargo build --manifest-path src-tauri/Cargo.toml
pnpm tauri:build
git diff --check
SQLite fresh-schema, populated legacy-upgrade, rollback, idempotence, integrity, and foreign-key checks
Installed Windows startup smoke against a backed-up legacy database
```

## Blocking decisions

None. The safest behavior can be derived from existing architecture and historical repository evidence. If legacy managed-file bytes cannot be located deterministically, the task must preserve metadata and expose a recoverable limitation rather than guessing or deleting.

## Self-review record

- **Status:** Pending.
- **Acceptance mapping:** Pending implementation and verification.
- **Data safety:** No production database or managed file may be modified before a verified backup exists.
- **Known limitations:** To be determined from historical implementation evidence.
