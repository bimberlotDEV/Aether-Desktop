# Codex Task Contract

> Canonical execution contract for the single active `planned_codex` task. The filename is retained for repository compatibility; it is not an inter-agent handoff.

| Field | Value |
| --- | --- |
| Schema version | 2 |
| Task ID | `PHASE7-001` |
| Status | `self_review` |
| Owner | Codex |
| Prepared by | Codex |
| Last updated | 2026-08-10 |
| Related milestone | Phase 7 — AI streaming and context foundation |

## Responsibility of this file

- Hold exactly one active, bounded task contract for complex or risky work.
- Define objective, scope, allowed paths, acceptance criteria, validation, and risks before implementation.
- Record implementation evidence, deviations, self-review findings, and final outcome.
- Remain `idle` for `direct_codex` work.
- Never serve as the general backlog or architecture diary.

## Status values

- `idle`: no planned task is active.
- `draft`: Codex is analysing and writing the contract.
- `ready`: the readiness gate passes; implementation may begin.
- `in_progress`: implementation is underway.
- `self_review`: implementation is complete and undergoing an independent Codex review pass.
- `changes_required`: self-review found corrections that must be implemented.
- `blocked`: a human decision or external dependency is required.
- `complete`: acceptance criteria, verification, self-review, and publication are complete.
- `superseded`: replaced by another task with a recorded reason.

## Classification

```text
Classification: planned_codex
Reason: AI networking, credential boundaries, cancellation, persistent partial output, and private context isolation are security-sensitive cross-layer concerns.
```

## Current task

### Objective

Replace the AI prototype transport with a current, cancellable, typed streaming foundation and enforce explicit Space-isolated context before building the chat UI.

### Context

- Phase 6 is merged through PR #10 at `c352e67`.
- The AI prototype still used retired DeepSeek aliases and blocking/non-streaming transport.
- Existing DPAPI credential storage is retained; credentials must never cross IPC.
- ADR-011 defines channel streaming, async cancellation, message terminal states, and explicit context isolation.

### Implementation plan

1. Replace the blocking provider transport with async Rustls streaming and current DeepSeek V4 models.
2. Add a typed Tauri channel protocol and request registry with network-racing cancellation.
3. Persist user and partial assistant turns with complete/cancelled/error terminal states.
4. Resolve explicit Note, Task, and Vault context in Rust with Space isolation and bounded payloads.
5. Harden conversation validation, history limits, context deduplication, migration, TypeScript contracts, and tests.

### Allowed files

- `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`
- `src-tauri/src/ai/*`, `src-tauri/src/commands.rs`, `src-tauri/src/lib.rs`
- `src-tauri/src/db/migrations.rs`, `src-tauri/src/db/repositories/conversations.rs`
- `src/lib/db/types.ts`, `src/lib/db/tauri.ts`, `src/lib/db/tauri.test.ts`
- `docs/decisions/011-ai-streaming-and-context.md`
- `.ai/ARCHITECTURE.md`, `.ai/HANDOFF.md`, `.ai/PROJECT_STATE.md`, `.ai/TODO.md`, `.ai/CHANGELOG.md`, `.ai/SESSION_NOTES.md`

### Out of scope

- Chat, Settings, context picker, summary, and task-proposal UI (`PHASE7-002`).
- Sending Vault file bytes or content; only explicit metadata is in scope.
- Additional providers, autonomous tools, web search, or implicit database access.

### Acceptance criteria

- [x] Provider advertises only current `deepseek-v4-flash` and `deepseek-v4-pro` models.
- [x] Streaming uses ordered typed Tauri channels and handles split UTF-8/SSE events and keep-alives.
- [x] Cancellation races connection and stream reads and always clears request registration.
- [x] User and assistant turns persist with terminal complete, cancelled, or classified error status.
- [x] History selects the latest bounded turns in chronological order.
- [x] Note/Task/Vault context is explicit, bounded, deduplicated, and Space-isolated.
- [x] Vault context excludes stored paths and file content.
- [x] Provider errors are actionable and never include API keys or request bodies.
- [x] TypeScript contracts cover credentials, conversations, models, channels, cancellation, and context.
- [x] Strict frontend and Rust gates pass.

### Required verification

- `pnpm check` — pass.
- `pnpm build` — pass.
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` — pass.
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` — pass.
- `cargo test --manifest-path src-tauri/Cargo.toml` — all tests pass, including temporary-directory ownership tests.
- `cargo build --manifest-path src-tauri/Cargo.toml` — pass.
- `git diff --check` — pass.

### Risks and rollback

| Risk | Mitigation or rollback |
| --- | --- |
| Deprecated provider contract stops working. | Use current V4 model IDs and cover them with regression tests. |
| Cancellation leaves endless loading. | Race a cancellation token against connect/read and persist a terminal state. |
| Context leaks across Spaces. | Resolve every entity in Rust against conversation Space immediately before sending. |
| Stream chunks split UTF-8 or SSE frames. | Buffer raw bytes until a complete event boundary; test multibyte splits. |
| Provider errors expose sensitive payloads. | Classify status/network failures without returning raw response or request bodies. |

## Implementation result

### Summary

Implemented the current DeepSeek V4 provider contract, async Rustls streaming, typed channel events, cancellable request registry, persistent terminal messages, explicit context resolution, migration hardening, and full TypeScript IPC surface.

### Files changed

- Added ADR-011, `src-tauri/src/ai/context.rs`, and `src-tauri/src/ai/runtime.rs`.
- Rebuilt `provider.rs`; updated AI commands, state registration, migration/repository behavior, TypeScript schemas/wrappers/tests, dependencies, and control documents.

### Verification result

- `pnpm check` - pass; 31/31 tests across ten files.
- `pnpm build` - pass.
- Rust formatting, strict Clippy, 56/56 tests, and build - pass.
- `git diff --check` - pass.

### Deviations

- `deepseek-v4-flash` is the cost-conscious default with thinking explicitly disabled; UI selection can opt into Pro.
- Vault file content remains excluded until a separately reviewed parsing/consent design exists.

## Codex self-review

| Field | Value |
| --- | --- |
| Decision | `approved_for_publication` |
| Reviewed at | 2026-08-10 |
| Acceptance evidence | All ten criteria pass; 56 Rust tests and 31 frontend tests pass with strict lint/build gates. |
| Findings | Prototype used retired model aliases, blocking transport, lossy per-chunk UTF-8 decoding, unbounded/oldest-first history, and unvalidated context. |
| Corrections | Replaced transport and model IDs; buffered raw SSE bytes; bounded latest history; added cancellation, terminal state, deduplication, and Space isolation. |
| Residual risks | Live DeepSeek behavior requires a user-provided key and is verified through Settings/UI and final desktop smoke testing. |
