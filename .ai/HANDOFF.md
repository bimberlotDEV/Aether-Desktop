# Codex Task Contract

| Field             | Value                                 |
| ----------------- | ------------------------------------- |
| Schema version    | 2                                     |
| Task ID           | `CONT-001`                            |
| Status            | `self_review`                         |
| Owner             | Codex                                 |
| Last updated      | 2026-08-27                            |
| Related milestone | Milestone D — Continuity and Activity |
| Classification    | `planned_codex`                       |

## Objective

Make every Space resumable from real local state and turn Activity into a useful, quiet timeline of meaningful actions, without AI-generated summaries, content extraction, hidden tracking, or cross-Space leakage.

## Context

Milestones A through C are merged on `master` at `36a6a67`. Aether already stores `activity_events` and records selected Task, Vault, and Memory mutations, but the global Activity route is a static empty state, Note and Source changes are incomplete, Space opens are not meaningful events, and the Space overview does not help the user resume work. ADR-018 selects one deterministic continuity read model over existing tables and a curated backend-owned event vocabulary.

## Acceptance criteria

- [x] One typed Rust continuity command returns a Space-scoped, bounded snapshot containing last-worked time, recent Notes, open Tasks, recent present Source files, latest active AI conversation, meaningful Activity, and one deterministic suggested next step.
- [x] Every continuity query enforces the requested active Space in SQL; archived Notes/Tasks/conversations, removed indexed files, unrelated/global Memory, and entities from other Spaces cannot leak into the snapshot.
- [x] Suggestions are derived from visible local records using a documented fixed priority; no DeepSeek call, prompt, embedding, fabricated narrative, or automatic mutation is involved.
- [x] Meaningful backend events cover Space opened, Note created/edited, Task created/completed, Vault file imported/updated/removed, Memory added/updated/deleted, Source scan changes, and successful AI conversation use.
- [x] Repeated Space opens are deduplicated in a fixed quiet window, Note autosave records only a bounded deduplicated edit signal, and Source scans with no changes do not create timeline noise.
- [x] Frontend callers cannot inject arbitrary event names or unvalidated metadata; events are emitted by domain commands and the public raw `record_activity` IPC surface is removed.
- [x] The global Activity route loads real local events with clear type, Space/provenance, time, accessible destinations, honest loading/error/empty states, and no raw JSON or absolute Source path disclosure.
- [x] Each active Space overview displays a calm resume surface with compact real sections and a useful next action, without turning into a generic widget dashboard.
- [x] Browser mode explicitly explains that continuity requires the installed desktop app and never fabricates persisted state.
- [x] Tests cover scope isolation, archive/removal exclusions, deterministic suggestion ordering, event allowlisting/deduplication, safe Source metadata, typed IPC arguments, Activity navigation, error/empty states, and browser behavior.
- [ ] Frontend/Rust gates, production and Tauri builds, dependency audit, diff/security review, browser smoke, packaged startup, and GitHub Windows CI pass.

## Allowed paths

- `src-tauri/src/db/repositories/activity.rs`
- `src-tauri/src/db/repositories/continuity.rs`
- `src-tauri/src/db/repositories/mod.rs`
- `src-tauri/src/commands.rs`
- `src-tauri/src/lib.rs`
- `src/lib/db/types.ts`
- `src/lib/db/tauri.ts`
- `src/lib/db/tauri.test.ts`
- `src/hooks/useContinuity.ts`
- `src/hooks/useContinuity.test.ts`
- `src/routes/Activity.tsx`
- `src/routes/Activity.test.tsx`
- `src/routes/SpaceDetail.tsx`
- `src/routes/SpaceDetail.test.tsx`
- `src/App.tsx`
- `src/components/ui/AetherUI.tsx`
- `src/styles/index.css`
- `docs/decisions/018-deterministic-continuity.md`
- `docs/architecture.md`
- `README.md`
- `.ai/*`

## Non-goals

- AI-generated daily briefs, automatic Memory, semantic embeddings, file-content extraction, notifications, telemetry, analytics, or background filesystem watchers.
- A complete audit log, undo log, version history, compliance ledger, or exposure of deleted content.
- New dashboards, fake metrics, charts, decorative feeds, or autonomous suggested actions.
- Backup/restore redesign, public release signing, or updater activation.

## Dependencies and evidence

- Merged Universal Search: PR #39 at `36a6a67`.
- Existing versioned `activity_events` schema, local-first repositories, active/archived predicates, Source metadata index, and typed Tauri bridge.
- ADR-011/012/016/017 privacy, Space-isolation, explicit-context, and bounded-metadata boundaries.
- Accepted ADR-018.

## Risks and safeguards

- **Cross-Space disclosure:** bind `space_id` at every query boundary and test adversarial mixed-Space fixtures.
- **Activity noise:** backend-owned allowlist plus per-event quiet windows; no per-keystroke or no-change scan events.
- **Misleading summaries:** expose structured facts and fixed local suggestions, never generated prose presented as fact.
- **Sensitive metadata:** normalize event labels in Rust and return only presentation-safe fields; never raw metadata JSON, file contents, credentials, or absolute Source child paths.
- **Autosave coupling:** event recording occurs in the same transaction as domain persistence and cannot make a successful edit appear failed after the fact.
- **Performance:** indexed, bounded queries with small per-section limits and one command per Space overview load.
- **Rollback:** no migration is planned; remove the command/UI and retain harmless existing events if rollback is required.

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
browser Activity and Space-overview smoke
packaged desktop startup smoke
GitHub PR CI
```

## Blocking decisions

None. The owner explicitly authorized Milestone D immediately after C. The continuity model is deterministic, local, reversible, and stays inside the approved roadmap.

## Readiness review

- **Status:** Ready. Scope, data boundaries, event vocabulary, privacy constraints, rollback, validation, and acceptance evidence are explicit.
- **Architecture gate:** ADR-018 is accepted before production changes.
- **Migration gate:** No schema migration is required; existing indexed tables and `activity_events` support the bounded read model and deduplication queries.
- **Security gate:** Arbitrary frontend event injection is removed; all new output is Space-bound and presentation-safe.

## Self-review

- **Outcome:** Local implementation is complete; GitHub Windows CI remains the final acceptance item.
- **Acceptance evidence:** 69/69 frontend tests and 77/77 Rust tests cover the typed bridge, UI states, event allowlisting/deduplication, deterministic ordering, exclusions, and adversarial Space isolation.
- **Architecture/security:** The final diff keeps SQL in Rust repositories, removes raw frontend event injection, exposes no raw metadata or absolute Source child paths, adds no migration/dependency/AI call, and performs no autonomous mutation.
- **Corrections made:** Route destinations now fall back when a Space module is disabled, Activity and Space Detail are lazy-loaded to keep the main bundle below the warning threshold, and semantic surface props preserve accessible live-region markup.
- **Validation:** `pnpm check`, `pnpm build`, high-severity dependency audit, Rust formatting, strict Clippy, all-target/all-feature Rust tests, `pnpm tauri:build`, `git diff --check`, browser smoke, and packaged startup smoke pass on 2026-08-27.
