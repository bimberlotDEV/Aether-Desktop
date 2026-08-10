# Codex Task Contract

> Canonical execution contract for the single active `planned_codex` task. The filename is retained for repository compatibility; it is not an inter-agent handoff.

| Field | Value |
| --- | --- |
| Schema version | 2 |
| Task ID | `PHASE7-002` |
| Status | `self_review` |
| Owner | Codex |
| Prepared by | Codex |
| Last updated | 2026-08-10 |
| Related milestone | Phase 7 — Complete AI experience |

## Classification

```text
Classification: planned_codex
Reason: The task crosses credential UX, provider streaming, persistent conversations, private context, and transactional Task creation.
```

## Objective

Make the reviewed AI foundation usable through secure Settings, global and Space-scoped chat, explicit visible context, controllable streaming, response modes, and previewed Task proposals.

## Scope and plan

1. Add secure DeepSeek key configuration, connection testing, replacement, and removal.
2. Add synchronized conversation lifecycle and streamed message orchestration with cancel and retry.
3. Add global and Space AI surfaces with explicit Note, Task, and Vault context selection.
4. Add Ask, Summarize, Explain, Plan, Rewrite, and Propose Tasks modes.
5. Validate proposed Tasks, require user selection/confirmation, and create them transactionally.
6. Add navigation, IPC contracts, regression tests, and implementation documentation.

## Allowed files

- `src-tauri/src/commands.rs`, `src-tauri/src/lib.rs`
- `src/components/ai/*`, `src/hooks/useAi.ts`, `src/routes/AI.tsx`
- `src/App.tsx`, `src/components/Sidebar.tsx`, `src/components/CommandPalette.tsx`
- `src/routes/Settings.tsx`, `src/routes/SpaceDetail.tsx`
- `src/lib/aiProposal*`, `src/lib/db/types.ts`, `src/lib/db/tauri.ts`, `src/lib/db/tauri.test.ts`
- `.ai/*`

## Acceptance criteria

- [x] Settings can securely save/replace, test, and explicitly remove a DeepSeek API key.
- [x] Global and Space AI support create, select, rename, archive, and delete conversation flows.
- [x] Typed streamed responses expose progress, cancellation, terminal errors, and retry without duplicating the user turn.
- [x] Context is visibly attached and removable; Rust remains the final Space-isolation authority.
- [x] Vault context is described and transmitted as metadata-only.
- [x] Ask, Summarize, Explain, Plan, Rewrite, and Propose Tasks modes have explicit provider instructions.
- [x] Task proposals are strictly validated, previewed, selectable, and created atomically only after confirmation.
- [x] AI privacy disclosure names the external provider boundary; browser mode does not fabricate credentials or data.
- [x] Navigation and Space-module integration expose the complete experience.
- [x] Strict frontend and Rust gates pass.

## Verification result

- `pnpm check` — pass; 36/36 tests across 13 files.
- `pnpm build` — pass.
- `cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check` — pass.
- `cargo test --manifest-path src-tauri/Cargo.toml` — pass; 56/56 tests.
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` — pass.
- `git diff --check` — pass.

## Risks and controls

| Risk | Control |
| --- | --- |
| A retry duplicates the original user prompt. | Retry references and validates the existing user message ID. |
| AI silently sends unrelated local data. | Users explicitly attach visible context; Rust resolves and isolates it at send time. |
| Generated Tasks cause partial writes. | A maximum of 20 validated selections are inserted in one transaction. |
| Credentials leak to React or logs. | DPAPI storage and Rust-only retrieval remain unchanged; the input is cleared after save. |

## Codex self-review

| Field | Value |
| --- | --- |
| Decision | `approved_for_publication` |
| Reviewed at | 2026-08-10 |
| Acceptance evidence | All ten criteria and every required quality gate pass. |
| Corrections | Moved proposal parsing out of a React component, fixed active-stream message targeting, added transaction-only batch creation, and covered credential/proposal/IPC boundaries. |
| Residual risks | A live provider smoke test requires the owner's DeepSeek key and is deferred to the final desktop release audit. |
