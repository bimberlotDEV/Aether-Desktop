# Codex Task Contract

| Field             | Value                            |
| ----------------- | -------------------------------- |
| Schema version    | 2                                |
| Task ID           | `CTX-001`                        |
| Status            | `complete`                       |
| Owner             | Codex                            |
| Last updated      | 2026-08-26                       |
| Related milestone | Milestone B — Context Foundation |
| Classification    | `planned_codex`                  |

## Objective

Add the first privacy-preserving Context Engine slice: users can explicitly authorize a local directory as a Source, optionally associate it with a Space, run a bounded non-destructive metadata scan, inspect its status and indexed files, rescan for changes, and revoke access without Aether modifying any user file.

## Context

Alpha Hardening and release 0.3.2 are complete on merged PR #37. The product-evolution roadmap identifies Context Foundation as the next milestone and requires transparent, revocable directory authorization, local-first indexing, change detection, background work, and append-only database evolution. The current Vault supports individual linked/managed files, but there is no authorized-directory model or file index. ADR-016 defines the new trust boundary.

## Acceptance criteria

- [x] Migration `009_context_sources` creates constrained `sources` and `indexed_files` tables plus query indexes without changing existing migrations; fresh creation, upgrade, and idempotence tests pass.
- [x] Adding a Source requires an explicitly selected absolute directory, canonicalizes it in trusted Rust, rejects filesystem roots, regular files, duplicates, and the Aether application-data tree, and never grants broader access than the selected directory.
- [x] Scanning runs off the UI thread, never follows symlinks or Windows reparse points, skips known dependency/build internals, has deterministic depth/file limits, and records regular-file metadata using relative paths.
- [x] A completed rescan reports new, changed, renamed, removed, unchanged, skipped, error, and truncation counts; rename inference only occurs for an unambiguous metadata match.
- [x] Revoking a Source deletes only Aether's Source/index records and never moves, renames, writes, or deletes user files.
- [x] Source commands are narrow and validated; indexed-file responses expose relative paths and metadata but not reconstructed absolute child paths.
- [x] Users can add, rescan, associate, inspect, and revoke Sources from a dedicated accessible route with explicit privacy language, loading/progress feedback, empty state, error state, and destructive confirmation.
- [x] Browser mode presents an honest installed-app requirement and no fake persistence or scan results.
- [x] No Source metadata or indexed file content is sent to DeepSeek or attached to AI in this task.
- [x] Frontend and Rust quality gates, production build, migration tests, repository tests, filesystem safety tests, and final diff/security review pass locally; GitHub CI is pending publication.

## Allowed paths

- `src-tauri/src/context.rs`
- `src-tauri/src/db/migrations.rs`
- `src-tauri/src/db/repositories/mod.rs`
- `src-tauri/src/db/repositories/sources.rs`
- `src-tauri/src/commands.rs`
- `src-tauri/src/lib.rs`
- `src/lib/db/types.ts`
- `src/lib/db/tauri.ts`
- `src/lib/db/tauri.test.ts`
- `src/routes/Sources.tsx`
- `src/routes/Sources.test.tsx`
- `src/App.tsx`
- `src/App.test.tsx`
- `src/components/Sidebar.tsx`
- `src/components/Sidebar.test.tsx`
- `src/components/CommandPalette.tsx`
- `src/components/CommandPalette.test.tsx`
- `src/styles/index.css`
- `docs/decisions/016-context-sources-and-indexing.md`
- `docs/architecture.md`
- `README.md`
- `.ai/ARCHITECTURE.md`
- `.ai/CHANGELOG.md`
- `.ai/HANDOFF.md`
- `.ai/PROJECT_STATE.md`
- `.ai/SESSION_NOTES.md`
- `.ai/TODO.md`

## Non-goals

- Filesystem watchers, automatic startup/background schedules, continuous monitoring, or scanning an unselected directory.
- Universal search UI/ranking, FTS, embeddings, similarity, file recommendations, or AI context attachment.
- Reading or persisting file contents, PDF extraction, hashes, thumbnails, previews, or semantic classification.
- Moving, renaming, copying, opening, deleting, or reorganizing user files.
- Automatic Space assignment, assignment rules, or AI suggestions.
- Backup/restore or Vault managed-byte archive behavior.
- Cloud sync, collaboration, telemetry, licensing, signing, or updater activation.

## Dependencies and evidence

- Merged 0.3.2 baseline: `3459e0f` on `master`.
- Product-evolution masterprompt sections 3, 6–8, 30–32, and Milestone B.
- Existing Rust repository → Tauri command → typed wrapper → React boundary.
- Existing directory dialog capability from `tauri-plugin-dialog`.
- ADR-010 for filesystem ownership safety and ADR-014 for local data-export disclosure patterns.
- Accepted ADR-016 for Source authorization and snapshot-index semantics.

## Risks and safeguards

- **Scope escape:** canonicalize the authorized root, derive only relative child records, skip symlinks/reparse points, and never accept a child path from the frontend.
- **Resource exhaustion:** iterative bounded traversal, maximum depth and file count, excluded generated/dependency directories, metadata-only indexing, and blocking work outside the async UI path.
- **Concurrent scans:** a process-local runtime guard permits one scan per Source and releases on every success/error path.
- **Stale or missing files:** snapshot application is transactional; unseen rows become `removed` only after a complete non-truncated scan.
- **Rename false positives:** preserve identity only when one removed and one new file share an otherwise unique size/modified-time signature; ambiguous cases remain remove+add.
- **User-data loss:** scanner uses metadata/read-directory operations only; revoke deletes database rows through cascade and has explicit UI confirmation.
- **Privacy leakage:** the frontend sees an authorized root only for Source transparency and sees relative indexed paths; no AI integration is introduced.
- **Rollback:** reverting code leaves append-only tables unused; removing a Source is safe and does not touch the selected directory.

## Required validation

```text
pnpm typecheck
pnpm lint
pnpm test
pnpm build
cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all-targets --all-features
git diff --check
fresh migration, upgrade, and idempotence tests
filesystem root/file/duplicate/app-data/symlink/reparse/depth/count safety tests
new/changed/removed/unambiguous-rename/truncated snapshot tests
browser-mode route and accessibility smoke
installed Tauri directory-selection and scan smoke on a temporary owner-created fixture
GitHub PR CI
```

## Blocking decisions

None. The owner authorized the next roadmap phase. ADR-016 selects a reversible metadata-only snapshot foundation; cloud use, automatic monitoring, actions, restore, and broader indexing remain excluded.

## Self-review record

- **Status:** Complete. Local implementation, self-review, draft PR publication, clean Windows CI, packaged startup, live migration inspection, and consistent database backup pass.
- **Acceptance mapping:** Migration 009 and migration tests cover append-only persistence; `context.rs` covers canonical authorization and bounded traversal; the Source repository covers transactional snapshot reconciliation and safe revocation; narrow Tauri commands and typed wrappers enforce the IPC boundary; the lazy-loaded Sources route covers transparent authorization, inspection, rescan, Space association, and confirmed revocation.
- **Automated evidence:** `pnpm check` passes 60/60 tests across 26 files; `pnpm build` produces a 493.59 kB main chunk and 7.26 kB lazy Sources chunk without a size warning; `pnpm audit --audit-level high`, Rust formatting, strict Clippy, 70/70 Rust tests, `pnpm tauri:build`, and `git diff --check` pass. GitHub Actions run 32972614553 repeats frontend quality/build, Rust formatting, strict lint, and Rust tests successfully on a clean Windows runner.
- **Interactive evidence:** Browser-mode `/sources` renders the installed-app disclosure, privacy boundary, accessible navigation, and a disabled Add folder action without console errors. Packaged candidates startup-smoke as responsive. The live owner database applies migration 009 and remains with zero authorized Sources. Native Windows picker interaction was not programmatically driven because doing so required enabling a prohibited WebView debug boundary; picker invocation is covered at the React boundary, while canonical authorization, real temporary-directory scanning, revocation, and non-mutation are covered in Rust.
- **Findings corrected:** Removed two React hook dependency warnings; lazy-loaded Sources to remove the main-chunk warning; fixed reappearing-file identity collisions with regression coverage; clarified large-index count disclosure. The final security review found no Source code path that writes, copies, renames, deletes, or attaches data to AI.
