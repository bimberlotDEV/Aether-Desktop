# Codex Task Contract

| Field             | Value                      |
| ----------------- | -------------------------- |
| Schema version    | 2                          |
| Task ID           | `ACTION-001`               |
| Status            | `self_review`              |
| Owner             | Codex                      |
| Last updated      | 2026-08-27                 |
| Related milestone | Milestone F — Safe Actions |
| Classification    | `planned_codex`            |

## Objective

Add a narrow, useful Action subsystem in which every native or persistent mutation is typed, previewed, explicitly approved, validated in Rust, executed once, and recorded without exposing unrestricted computer control.

## Acceptance criteria

- [ ] Rust defines a closed Action request vocabulary for create Task, create Note, create folder, copy/move/rename file inside one authorized Source, open Source file, and open Source folder.
- [ ] Preview validates and normalizes the request, returns a presentation-safe consequence summary, and issues only a short-lived opaque one-time token; absolute Source roots never cross IPC.
- [ ] Execution accepts only an unexpired pending token, consumes it before work, and cannot be replayed or replaced with arbitrary arguments.
- [ ] Filesystem writes remain inside the canonical authorized Source root, reject traversal/rooted paths, symlink escape, missing parents, directory-as-file input, and existing destinations, and never delete or overwrite user data.
- [ ] Create Task/Note validates active Space ownership and persists transactionally; every successful write receives a curated Activity record in the same transaction.
- [ ] Filesystem failures are surfaced clearly; if audit recording fails after a reversible filesystem write, the operation is rolled back where safe.
- [ ] No command exposes shell execution, arbitrary executable launch, raw path access, delete, automation, model-triggered approval, or unrestricted external application control.
- [ ] The Actions route provides an accessible proposal → review → approve/cancel → result flow and an honest browser-mode disclosure.
- [ ] Tests cover every action type, one-time/expiry behavior, validation, traversal, containment, symlink-safe canonicalization, no-overwrite, rollback/audit semantics, typed IPC, confirmation, cancellation, errors, and browser behavior.
- [ ] Frontend/Rust gates, build, audit, packaging, diff/security review, browser smoke, packaged startup, and exact-head Windows CI pass.

## Allowed paths

- `src-tauri/src/actions.rs`
- `src-tauri/src/commands.rs`
- `src-tauri/src/lib.rs`
- `src-tauri/src/db/repositories/activity.rs`
- `src/lib/db/types.ts`
- `src/lib/db/tauri.ts`
- `src/lib/db/tauri.test.ts`
- `src/hooks/useActions.ts`
- `src/routes/Actions.tsx`
- `src/routes/Actions.test.tsx`
- `src/App.tsx`
- `src/components/Sidebar.tsx`
- `src/components/CommandPalette.tsx`
- `src/components/CommandPalette.test.tsx`
- `src/styles/index.css`
- `docs/decisions/020-safe-actions.md`
- `docs/architecture.md`
- `README.md`
- `.ai/*`

## Non-goals

- Delete actions, recursive directory operations, overwrite, arbitrary shell/terminal commands, scripts, downloads, network calls, arbitrary application launch, background automation, or model-owned approval.
- Cross-Source moves/copies, unmanaged filesystem paths, rollback across process crashes, or generalized plugin execution.
- AI proposal generation; Milestone G may propose only these existing types through a separately reviewed boundary.

## Risks and safeguards

- **Path escape/spoofing:** component validation plus canonical root/source/parent containment in Rust.
- **Replay:** pending tokens are random, short-lived, process-local, and removed before execution.
- **Data loss:** no delete/overwrite; file writes are single-item and rollback when post-write audit fails.
- **Confused deputy:** frontend never supplies an absolute path to execution and cannot mutate a reviewed proposal.
- **Audit privacy:** curated Activity exposes type/entity/provenance only, never absolute paths or file contents.
- **Rollback:** remove the route/commands/runtime and retain harmless curated Activity rows; no migration is added.

## Required validation

```text
pnpm check
pnpm build
pnpm audit --audit-level high
cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all-targets --all-features
pnpm tauri:build
git diff --check
browser Actions smoke
packaged desktop startup smoke
GitHub PR CI
```

## Blocking decisions

None. The owner explicitly authorized Milestone F. The initial vocabulary is narrow, reversible where practical, and intentionally excludes destructive/general autonomy.

## Readiness review

- **Status:** Ready. Action types, authority boundaries, confirmation semantics, rollback, privacy, and validation are explicit.
- **Architecture gate:** ADR-020 is accepted before production implementation.
- **Migration gate:** No migration is required; curated Activity supplies the durable audit signal.
- **Security gate:** Rust owns proposal state, validation, execution, containment, and audit recording.

## Codex self-review

**Verdict:** Implementation satisfies the ready contract locally; exact-head Windows CI remains the publication gate.

| Acceptance criterion                   | Evidence                                                                                                                                                                                                          |
| -------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Closed Action vocabulary               | `ActionRequest` contains only the eight contracted Task, Note, contained file/folder, and open variants; typed TS IPC mirrors it.                                                                                 |
| Preview and one-time execution         | Rust returns only a presentation-safe preview and opaque ten-minute token; execution removes the token before revalidation; replay, cancellation, expiry, and the 128-proposal bound are tested.                  |
| Filesystem containment and data safety | Canonical Source/path checks reject traversal, outside symlinks, unindexed files, missing parents, non-files, and existing destinations. Copy uses `create_new`; no delete/overwrite/cross-Source command exists. |
| Transaction and rollback               | Task/Note plus Activity commit in one transaction. A forced Activity failure proves a copied file is removed and its original remains.                                                                            |
| Explicit confirmation UX               | The desktop route separates proposal from review and invokes execution only from “Approve and execute”; cancellation and honest browser mode are tested.                                                          |
| Privacy and authority                  | Preview/result never contain Source roots or file contents. Security diff scan found no shell/process/recursive-delete/network/general-executable capability.                                                     |
| Validation                             | 77/77 frontend tests, 85/85 Rust tests, typecheck, lint, production build, high audit, fmt, strict Clippy, diff check, MSI/NSIS build, 1024×640 browser smoke, and packaged startup pass.                         |

No unresolved P0/P1 findings remain. File mutations intentionally require a later Source rescan to refresh indexed metadata; the route discloses this rather than silently expanding Action authority.
