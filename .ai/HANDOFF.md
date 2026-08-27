# Codex Task Contract

| Field             | Value                   |
| ----------------- | ----------------------- |
| Schema version    | 2                       |
| Task ID           | `PULSE-002`             |
| Status            | `self_review`           |
| Owner             | Codex                   |
| Last updated      | 2026-08-27              |
| Related milestone | Milestone E — Pulse 2.0 |
| Classification    | `planned_codex`         |

## Objective

Turn Pulse into a calm, immediately useful answer to “What needs my attention today?” using only bounded, explainable local state.

## Context

Milestones A–D are merged at `854d5b8`. Pulse currently composes separate frontend hooks for Spaces, Notes, and due Tasks; it has no single relevance model, Continue/New/Recent sections, or honest desktop boundary. ADR-019 selects one deterministic Rust read model over existing indexed data.

## Acceptance criteria

- [x] One typed Rust command returns bounded Today, Continue, New, Recent, and suggested-next-step data from active local records.
- [x] Today separates overdue, due-today, and upcoming open Tasks, orders deterministically, and never fabricates urgency.
- [x] Continue ranks recently worked active Spaces with a real reason and safe destination; archived and unrelated entities cannot leak.
- [x] New exposes only present metadata from explicitly authorized Sources, never contents or absolute child paths.
- [x] Recent uses the curated presentation-safe Activity model and contains no raw metadata JSON.
- [x] Suggested next step is derived by fixed documented priority and explains the supporting local fact; no AI call or mutation occurs.
- [x] Pulse provides a prominent Ask Aether entry point while explaining that context remains user-controlled.
- [x] Loading, error, empty, browser, keyboard, responsive, dark, and light states are honest and accessible.
- [x] Rust and frontend tests cover ordering, limits, inactive-row exclusions, Space isolation, safe Source metadata, typed IPC, navigation, and error/empty/browser behavior.
- [ ] Frontend/Rust gates, build, audit, packaging, diff/security review, browser smoke, packaged startup, and exact-head Windows CI pass.

## Allowed paths

- `src-tauri/src/db/repositories/pulse.rs`
- `src-tauri/src/db/repositories/mod.rs`
- `src-tauri/src/commands.rs`
- `src-tauri/src/lib.rs`
- `src/lib/db/types.ts`
- `src/lib/db/tauri.ts`
- `src/lib/db/tauri.test.ts`
- `src/hooks/usePulse.ts`
- `src/routes/Pulse.tsx`
- `src/routes/Pulse.test.tsx`
- `src/styles/index.css`
- `docs/decisions/019-deterministic-pulse.md`
- `docs/architecture.md`
- `README.md`
- `.ai/*`

## Non-goals

- AI-generated briefs, inferred emotion/importance, notifications, analytics, automatic Memory, background watchers, or autonomous actions.
- New persistence, migrations, dependencies, file-content extraction, or changes to Source authorization.
- Milestone F action execution.

## Risks and safeguards

- **False urgency:** local date/status predicates and visible factual explanations only.
- **Privacy:** Space-bound joins, active predicates, bounded metadata, and no provider call.
- **Noise:** small fixed limits and the existing curated Activity vocabulary.
- **Performance:** one command composed from indexed bounded queries.
- **Rollback:** remove the read model and restore the previous Pulse composition; no stored data changes.

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
browser Pulse smoke
packaged desktop startup smoke
GitHub PR CI
```

## Blocking decisions

None. The owner explicitly authorized milestone E. The deterministic, read-only design follows the approved roadmap and is reversible.

## Readiness review

- **Status:** Ready. Scope, relevance rules, privacy boundaries, rollback, and evidence requirements are explicit.
- **Architecture gate:** ADR-019 is accepted before production implementation.
- **Migration gate:** No migration is required.
- **Security gate:** The command is read-only and returns presentation-safe typed facts.

## Self-review

- **Outcome:** Local implementation is complete; exact-head GitHub Windows CI remains.
- **Acceptance evidence:** 72/72 frontend tests and 79/79 Rust tests cover typed IPC, factual grouping/priority, inactive exclusions, safe Source metadata, browser/error states, and navigation.
- **Architecture/security:** All queries remain in the Rust repository; the command is read-only, bounded, parameterized, and exposes no file contents, absolute Source roots, raw Activity metadata, provider call, or mutation.
- **Corrections made:** Activity filtering now propagates database failures rather than hiding them, Continue accepts only the curated event vocabulary, and never-opened Spaces use a factual changed-state reason.
- **Validation:** Frontend check/build, dependency audit, Rust format, strict Clippy, all-target/all-feature tests, MSI/NSIS packaging, browser 1024×640 smoke with zero console errors, diff/security review, and packaged startup pass on 2026-08-27.
