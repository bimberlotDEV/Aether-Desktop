# Codex Task Contract

| Field             | Value                          |
| ----------------- | ------------------------------ |
| Schema version    | 2                              |
| Task ID           | `SEARCH-001`                   |
| Status            | `complete`                     |
| Owner             | Codex                          |
| Last updated      | 2026-08-26                     |
| Related milestone | Milestone C — Universal Search |
| Classification    | `planned_codex`                |

## Objective

Turn Ctrl+K into a fast, accessible Universal Search over Aether commands and permitted local domains with transparent deterministic ranking, useful navigation, honest browser behavior, and no AI or Source-content disclosure.

## Context

Milestones A and B are complete on merged `master` at `1d11ac0`. The existing palette searches commands and loaded Spaces only. Notes already have FTS5; other domains have bounded repositories and indexes. ADR-017 selects a hybrid local search repository and a single typed IPC boundary.

## Acceptance criteria

- [x] One validated Rust command searches active Spaces, Notes, non-archived Tasks, Vault metadata, explicit Memory, non-archived AI conversation titles, meaningful Activity metadata, and present indexed-file metadata with strict query and result limits.
- [x] Notes content uses existing SQLite FTS5; all other queries are parameterized and escape wildcard/syntax input without raw SQL construction from user text.
- [x] Ranking is deterministic and test-covered using text quality, current Space, favourite/pinned, open Task, and recency signals with stable tie-breaking.
- [x] File results expose only Source identity, Source display name, and relative indexed paths; no file content, reconstructed absolute child path, removed row, or unapproved directory is returned.
- [x] Search never invokes DeepSeek, changes Memory scope, attaches context to AI, records noisy per-keystroke Activity, or mutates domain data.
- [x] Ctrl+K debounces desktop search, merges commands and domain results into one keyboard-operable accessible list, shows type/provenance/loading/error/empty states, and opens the most specific supported destination.
- [x] Browser mode remains an honest command/navigation search and does not fabricate database results.
- [x] Tests cover every result domain, archived/removed exclusions, current-Space ranking, special-character queries, limits, IPC argument shape, keyboard navigation, stale-response suppression, and browser behavior.
- [x] Frontend/Rust gates, production and Tauri builds, dependency audit, diff/security review, packaged startup smoke, and GitHub Windows CI pass.

## Allowed paths

- `src-tauri/src/db/repositories/search.rs`
- `src-tauri/src/db/repositories/mod.rs`
- `src-tauri/src/commands.rs`
- `src-tauri/src/lib.rs`
- `src/lib/db/types.ts`
- `src/lib/db/tauri.ts`
- `src/lib/db/tauri.test.ts`
- `src/components/CommandPalette.tsx`
- `src/components/CommandPalette.test.tsx`
- `src/components/Sidebar.tsx`
- `src/stores/commandStore.ts`
- `docs/decisions/017-universal-search-ranking.md`
- `docs/architecture.md`
- `README.md`
- `.ai/*`

## Non-goals

- Source file-content extraction, PDF parsing, OCR, embeddings, vector search, semantic ranking, or AI reranking.
- Search-driven mutations, autonomous actions, filesystem writes, telemetry, cloud search, or new external dependencies.
- Milestone D continuity summaries, Pulse changes, or expanding the Activity event model.
- Perfect entity highlighting where the destination route does not yet expose a stable selection contract.

## Dependencies and evidence

- Merged Context Foundation: PR #38 at `1d11ac0`.
- Existing `notes_fts`, domain repositories, typed invoke wrappers, and Ctrl+K store.
- ADR-011/012/016 privacy and Space-scope boundaries.
- Accepted ADR-017.

## Risks and safeguards

- **Query injection/syntax errors:** validate length, parameterize every value, escape LIKE wildcards, and construct FTS expressions only from quoted tokens.
- **Latency:** cap per-domain candidates, query off the React layer, debounce input, suppress stale responses, and cap final results.
- **Privacy:** metadata-only Source results, no absolute child paths or AI calls, explicit provenance on every result.
- **Ranking surprises:** fixed documented weights, stable tie-breaking, and current-Space boost only when explicitly supplied.
- **Archived leakage:** each domain query owns explicit active/present predicates.
- **Rollback:** no schema change; remove the command and restore command-only filtering.

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
browser-mode Ctrl+K smoke
packaged desktop startup smoke
GitHub PR CI
```

## Blocking decisions

None. The owner explicitly authorized Milestone C followed by D. ADR-017 chooses the reversible conventional-search foundation required by the roadmap.

## Self-review record

- **Status:** Complete. Draft PR #39 is published and exact-head GitHub CI run 32983855580 attempt 2 passes every Windows quality gate.
- **Acceptance mapping:** `search.rs` owns validated cross-domain retrieval, FTS/LIKE safety, exclusions, privacy, scoring, limits, and stable ordering; one command and typed wrapper own IPC; the Ctrl+K component owns command merging, debounce, stale suppression, provenance, keyboard behavior, and honest browser disclosure.
- **Evidence:** `pnpm check` passes 64/64 tests across 26 files; production build passes at 496.41 kB main JS / 139.94 kB gzip without a size warning; dependency audit, Rust formatting, strict Clippy, 73/73 Rust tests, final diff check, MSI/NSIS packaging, browser-mode Ctrl+K smoke, keyboard open/close, zero console errors, packaged startup, and GitHub Actions run 32983855580 attempt 2 pass.
- **Findings corrected:** Bounded metadata candidates now have deterministic recency/id ordering and a larger diversity cap before global ranking; Activity matches humanized event names; empty-result ArrowDown cannot produce index `-1`; the inherited forbidden `glass-surface` class was removed. The first Tauri build failed only because the running prior candidate locked `aether.exe`; after stopping that exact process, the unchanged build completed both bundles.
