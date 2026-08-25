# Agent Implementation Changelog

> Append-only record of completed implementation and collaboration-system changes. It complements, but does not replace, a future product release changelog.

## Responsibility of this file

- Record completed work after verification.
- Capture task ID, scope, files, validation, and notable follow-up.
- Remain append-only except for correcting factual errors.
- Exclude plans, uncompleted work, and raw session transcripts.

## Entry format

Entries are newest first and use ISO dates. Each entry must reference a stable task ID.

```markdown
## YYYY-MM-DD — `TASK-ID` — Short outcome

- **Type:** Feature | Fix | Refactor | Test | Docs | Process
- **Implemented by:** Codex
- **Reviewed by:** Codex self-review | Human | Not required
- **Summary:** <observable result>
- **Files:** `<path>`, `<path>`
- **Verification:** `<command>` — Pass/Fail/Not run with reason
- **Decisions/deviations:** None | <concise note>
- **Follow-up:** None | `TASK-ID`
```

## 2026-08-25 — `AI-CHAT-004` — Completed AI responses render without Ctrl+R

- **Type:** Fix and test
- **Implemented by:** Codex
- **Reviewed by:** Codex self-review
- **Summary:** Made AI stream events tolerant of missing lifecycle messages and reconciled each successfully completed stream with the authoritative persisted conversation, so a WebView Channel timing issue cannot leave the answer hidden until a page reload.
- **Files:** `src/hooks/useAi.ts`, `src/hooks/useAi.test.ts`, `.ai/*`
- **Verification:** `pnpm check` passes with 55/55 tests across 24 files; production and Tauri builds pass; Rust formatting, strict Clippy, and 62/62 tests pass. The installed 0.3.1 executable embeds `index-uxuVmjVb.js`, remains responsive, and opens an integrity-clean database with zero foreign-key violations.
- **Decisions/deviations:** No live provider message was sent during verification because deterministic Channel-loss tests cover the state boundary without using private credentials or incurring external usage.
- **Follow-up:** None.

## 2026-08-11 — `AI-CHAT-003` — Stale loads no longer hide submitted prompts

- **Type:** Fix, test, and release
- **Implemented by:** Codex
- **Reviewed by:** Codex self-review
- **Summary:** Prevented an older in-flight conversation load from overwriting a newly submitted prompt and its streaming assistant placeholder. Added an explicit message revision guard, disabled sending while the selected conversation is loading, and released the fix as Alpha 0.3.1 so Windows has a real patch-upgrade boundary.
- **Files:** `src/hooks/useAi.ts`, `src/hooks/useAi.test.ts`, `src/components/ai/AiView.tsx`, application version metadata/UI, release documentation, `.ai/*`
- **Verification:** `pnpm check` passes with 53/53 tests across 24 files; `pnpm build`, Rust format, strict Clippy, 61/61 Rust tests, `pnpm tauri:build`, and `git diff --check` pass. The 0.3.1 NSIS upgrade replaced the installed executable, registered version 0.3.1, embedded the expected `index-DW1rFMl4.js` frontend bundle, and opened the AI route without the error boundary.
- **Decisions/deviations:** No live DeepSeek prompt was sent by Codex because that would be an external message using the owner's credentials. The deterministic race test forces the stale-load ordering that caused the refresh-only symptom.
- **Follow-up:** None.

## 2026-08-11 — `AI-CHAT-002` — AI streaming no longer crashes the view

- **Type:** Fix and test
- **Implemented by:** Codex
- **Reviewed by:** Codex self-review
- **Summary:** Prevented the AI message-scroll effect from returning WebView2's Promise to React as an effect cleanup, which crashed the view on the next streamed message update. Existing persisted prompts and responses remain intact.
- **Files:** `src/components/ai/AiView.tsx`, `src/components/ai/AiView.test.tsx`, `.ai/*`, `docs/release-artifacts-0.3.0.md`
- **Verification:** `pnpm check` passes with 52/52 tests across 24 files; `pnpm build`, `pnpm tauri:build`, and `git diff --check` pass; both debug and packaged release runtimes render the AI route without the error boundary.
- **Decisions/deviations:** The regression test deliberately makes `scrollIntoView()` return a Promise, matching WebView2 behavior, and verifies that rerendering does not invoke it as a cleanup. No live provider request or user-data mutation was required.
- **Follow-up:** None.

## 2026-08-11 — `AI-CHAT-001` — Submitted prompts render immediately

- **Type:** Fix and test
- **Implemented by:** Codex
- **Reviewed by:** Codex self-review
- **Summary:** Added an optimistic user message before AI streaming begins and reconciled it with the persisted Rust message on the `started` event, preventing prompts from appearing only after refresh.
- **Files:** `src/hooks/useAi.ts`, `src/hooks/useAi.test.ts`, `.ai/*`
- **Verification:** `pnpm check` passes with 51/51 tests across 23 files; `pnpm build`, `pnpm tauri:build`, and `git diff --check` pass; a verified NSIS installer was copied to Downloads.
- **Decisions/deviations:** Retry reuses its existing user message and therefore does not create an optimistic duplicate. No live provider request was needed for verification, and the running installed app was not interrupted for a redundant startup smoke.
- **Follow-up:** None.

## 2026-08-11 — `STAB-001` — Integrated alpha stabilization complete

- **Type:** Fix, accessibility, test, native, and packaging
- **Implemented by:** Codex
- **Reviewed by:** Codex self-review
- **Summary:** Stress-tested the integrated browser and Windows desktop application; repaired blank navigation/render failures, Space icon mapping, inaccessible list/menu/dialog controls, incomplete browser-mode Spaces and Notes behavior, stale global Note refresh, and concurrent desktop instances.
- **Files:** App routing/error boundary, Space/Note hooks and routes, dialog/palette semantics, single-instance Tauri setup, regression tests, release/control documentation
- **Verification:** Frontend check/build pass with 50/50 tests; Rust format/strict Clippy/build/61 tests pass; dependency audit clean; browser/desktop matrices pass; MSI/NSIS build and two-launch single-instance release smoke pass.
- **Decisions/deviations:** Existing user data and credentials were left untouched. Public signing, updater activation, and live provider usage remain owner-controlled.
- **Follow-up:** None required for Alpha 0.3.0 stabilization.

## 2026-08-10 — `PHASE10-001` — Alpha release quality complete

- **Type:** Feature, security, accessibility, CI, packaging, and documentation
- **Implemented by:** Codex
- **Reviewed by:** Codex self-review
- **Summary:** Added sanitized workspace export, Windows CI and dependency monitoring, closed dependency advisories, improved dialog/theme accessibility, reconciled release documentation, and produced verified MSI/NSIS candidates.
- **Files:** Backup Rust/IPC/Settings/tests, `.github/*`, release/security/license/architecture docs, dependency manifests, accessibility tests, `.ai/*`
- **Verification:** Frontend check/build pass with 45/45 tests; Rust format/strict Clippy/61 tests pass; npm audit clean; MSI/NSIS build and executable startup pass; artifact hashes recorded.
- **Decisions/deviations:** Backup excludes API credentials and Vault file bytes. Signing, updater activation, and automatic restore remain gated on owner-controlled trust or separately designed semantics.
- **Follow-up:** None within the approved Alpha 0.3.0 roadmap.

## 2026-08-10 — `PHASE9-001` — Native Windows lifecycle and installers

- **Type:** Feature, packaging, and test
- **Implemented by:** Codex
- **Reviewed by:** Codex self-review
- **Summary:** Added hide-to-tray/Open/Quit lifecycle, non-fatal Ctrl+Shift+Space registration, native notifications, persisted window state, Settings readiness, installer metadata, and a signed-update activation gate.
- **Files:** Native Rust module/commands/plugins, Settings UI/tests, Tauri/Cargo/package metadata, ADR-013, native desktop docs, `.ai/*`
- **Verification:** Frontend check/build pass with 41/41 tests; Rust format, strict Clippy, and 59/59 tests pass; release executable starts; MSI and NSIS x64 installers build successfully.
- **Decisions/deviations:** Machine version is 0.3.0 for MSI compatibility; Alpha remains product maturity. Updater remains disabled until real signing trust exists.
- **Follow-up:** `PHASE10-001`

## 2026-08-10 — `PHASE8-001` — Explicit scoped Memory MVP

- **Type:** Feature, privacy, architecture, and test
- **Implemented by:** Codex
- **Reviewed by:** Codex self-review
- **Summary:** Added user-authored global and Space Memory with visible reason/category attribution, search, edit/delete, optional Space module, Activity events, and explicit AI context attachment.
- **Files:** Memory migration/repository/commands/types/wrappers/hooks/UI/tests, AI context integration, navigation, ADR-012, database docs, `.ai/*`
- **Verification:** Frontend check/build pass with 39/39 tests; Rust format, strict Clippy, and 58/58 tests pass; `git diff --check` pass.
- **Decisions/deviations:** No automatic conversation extraction; Space deletion cascades Memory and cleans polymorphic AI attachments.
- **Follow-up:** `PHASE9-001`

## 2026-08-10 — `PHASE7-002` — Complete AI experience

- **Type:** Feature, security, and test
- **Implemented by:** Codex
- **Reviewed by:** Codex self-review
- **Summary:** Added secure DeepSeek Settings, synchronized global and Space chat, model selection, streamed progress/cancel/retry, explicit visible context, six response modes, and strictly validated transactional Task proposals.
- **Files:** `src/components/ai/*`, `src/hooks/useAi.ts`, `src/routes/AI.tsx`, navigation and Space integration, AI commands, TypeScript IPC/types/tests, `.ai/*`
- **Verification:** Frontend check/build pass with 36/36 tests; Rust format, strict Clippy, and 56/56 tests pass; `git diff --check` pass.
- **Decisions/deviations:** Vault stays metadata-only; retry reuses an existing user turn; proposal creation is capped at 20 and atomic.
- **Follow-up:** `PHASE8-001`

## 2026-08-10 — `PHASE7-001` — Cancellable AI streaming foundation

- **Type:** Security, feature, refactor, and test
- **Implemented by:** Codex
- **Reviewed by:** Codex self-review
- **Summary:** Replaced retired DeepSeek aliases and blocking chat with current V4 models, async Rustls SSE streaming, typed Tauri channels, network-racing cancellation, persistent terminal messages, classified errors, and explicit bounded Space-isolated context.
- **Files:** `src-tauri/src/ai/*`, `src-tauri/src/commands.rs`, `src-tauri/src/lib.rs`, AI migration/repository, Rust dependencies, `src/lib/db/types.ts`, `src/lib/db/tauri.ts`, `src/lib/db/tauri.test.ts`, ADR-011, `.ai/*`
- **Verification:** Frontend check/build pass with 31/31 tests; Rust format, strict Clippy, test, and build pass with 56/56 tests; `git diff --check` pass.
- **Decisions/deviations:** `deepseek-v4-flash` is the default; Vault context is metadata-only; raw provider bodies and credentials never cross IPC.
- **Follow-up:** `PHASE7-002`

## 2026-08-10 — `PHASE6-002` — Vault MVP complete

- **Type:** Feature, accessibility, and test
- **Implemented by:** Codex
- **Reviewed by:** Codex self-review
- **Summary:** Replaced the Vault placeholder with synchronized global and Space-scoped workflows for native import, linked/managed ownership, search and filters, metadata editing, open/reveal, and ownership-specific safe removal.
- **Files:** `src/hooks/useVault.ts`, `src/components/vault/VaultEditor.tsx`, `src/components/vault/VaultView.tsx`, `src/routes/Vault.tsx`, `src/routes/Vault.test.tsx`, `src/routes/SpaceDetail.tsx`, `.ai/*`
- **Verification:** Frontend check/build pass with 30/30 tests; Rust format, strict Clippy, and 48/48 tests pass; `git diff --check` pass.
- **Decisions/deviations:** Browser mode remains read-only rather than fabricating user files; native actions stay behind the PHASE6-001 trusted command boundary.
- **Follow-up:** `AI-UI-EPIC`

## 2026-08-10 — `PHASE6-001` — Safe Vault foundation

- **Type:** Feature, security, and test
- **Implemented by:** Codex
- **Reviewed by:** Codex self-review
- **Summary:** Added linked and managed Vault persistence, canonical path containment, atomic managed imports, quarantined deletion with rollback, metadata/search filters, native dialog/open/reveal integration, and Rust-private storage paths.
- **Files:** `src-tauri/src/vault.rs`, `src-tauri/src/db/repositories/vault.rs`, `src-tauri/src/db/migrations.rs`, `src-tauri/src/commands.rs`, `src-tauri/src/lib.rs`, `src-tauri/capabilities/default.json`, `src/lib/db/types.ts`, `src/lib/db/tauri.ts`, `src/lib/db/tauri.test.ts`, dependency manifests, ADR-010, database docs, `.ai/*`
- **Verification:** Frontend check/build pass with 25/25 tests; Rust format, strict Clippy, test, and build pass with 48/48 tests; `git diff --check` pass.
- **Decisions/deviations:** Open/reveal stay behind ID-only Rust commands; frontend opener authority and serialized storage paths were intentionally excluded.
- **Follow-up:** `PHASE6-002`

## 2026-08-10 — `PHASE5-002` — Tasks and Pulse MVP complete

- **Type:** Feature, fix, and test
- **Implemented by:** Codex
- **Reviewed by:** Codex self-review
- **Summary:** Added a global Task Inbox, full Space Tasks module, synchronized mutations, quick capture, full metadata editing, subtasks, search/filtering, one-click completion, archive recovery and deletion, and calm overdue/upcoming Task attention on Pulse. Added Tasks navigation and repaired command-palette routing for BrowserRouter.
- **Files:** `src/hooks/useTasks.ts`, `src/hooks/useTasks.test.ts`, `src/components/tasks/*`, `src/routes/Tasks.tsx`, `src/routes/Tasks.test.tsx`, `src/routes/Pulse.tsx`, `src/routes/Pulse.test.tsx`, `src/routes/SpaceDetail.tsx`, `src/App.tsx`, `src/components/Sidebar.tsx`, `src/components/CommandPalette.tsx`, `src/components/CommandPalette.test.tsx`, `.ai/*`
- **Verification:** `pnpm check` — pass with 24/24 tests; `pnpm build` — pass; Rust format, strict Clippy, 39/39 tests, and build — pass; `git diff --check` — pass; desktop launch and accessibility discovery — pass.
- **Decisions/deviations:** The active desktop minimized Aether during UI automation, so the remaining manual clicks were replaced with expanded automated interaction coverage; no feature or architecture scope changed.
- **Follow-up:** `VAULT-EPIC`.

## 2026-08-10 — `PHASE5-001` — Task persistence and IPC foundation

- **Type:** Feature, architecture, and test
- **Implemented by:** Codex
- **Reviewed by:** Codex self-review
- **Summary:** Added the append-only Task migration, validated Rust repository with global and Space-owned task trees, CRUD and lifecycle commands, due-attention queries, Activity events, strict TypeScript contracts, and IPC wrappers.
- **Files:** `docs/decisions/009-task-domain-model.md`, `docs/database.md`, `src-tauri/src/db/migrations.rs`, `src-tauri/src/db/repositories/tasks.rs`, `src-tauri/src/commands.rs`, `src-tauri/src/lib.rs`, `src/lib/db/types.ts`, `src/lib/db/tauri.ts`, `src/lib/db/tauri.test.ts`, `.ai/*`
- **Verification:** `pnpm check` — pass with 16/16 tests; `pnpm build` — pass; Rust format, strict Clippy, 39/39 tests, and build — pass; `git diff --check` — pass.
- **Decisions/deviations:** ADR-009 governs the model. Runtime schema parsing was omitted from the low-level invoke wrapper after it increased the main bundle by approximately 60 kB; Rust validation remains authoritative and Zod schemas remain available at the UI boundary.
- **Follow-up:** `PHASE5-002`.

## 2026-08-10 — `PHASE34-CLOSEOUT` — Spaces and Notes MVP complete

- **Type:** Feature, fix, and test
- **Implemented by:** Codex
- **Reviewed by:** Codex self-review
- **Summary:** Completed Space editing, module selection, accessible reordering, cross-view mutation refresh, and correct archive restoration. Added recoverable archived Notes, full-content current-Space search, and serialized autosave that flushes pending edits on navigation. Corrected stale UI version labels.
- **Files:** `src/components/CreateSpaceModal.tsx`, `src/components/EditSpaceModal.tsx`, `src/components/Sidebar.tsx`, `src/components/spaceOptions.ts`, `src/hooks/useNotes.ts`, `src/hooks/useSpaces.ts`, `src/hooks/useSpaces.test.ts`, `src/lib/autosave.ts`, `src/lib/autosave.test.ts`, `src/lib/db/tauri.ts`, `src/lib/db/tauri.test.ts`, `src/routes/Notes.tsx`, `src/routes/Settings.tsx`, `src/routes/SpaceDetail.tsx`, `src/routes/Spaces.tsx`, `.ai/HANDOFF.md`, `.ai/PROJECT_STATE.md`, `.ai/TODO.md`, `.ai/CHANGELOG.md`, `.ai/SESSION_NOTES.md`
- **Verification:** `pnpm check` — pass with 14/14 tests; `pnpm build` — pass; Rust format, strict Clippy, 33/33 tests, and build — pass; local browser create/edit/archive/restore smoke test — pass with no console errors; `git diff --check` — pass.
- **Decisions/deviations:** Cleared optional Space display fields persist as empty strings because the existing optional Tauri command payload cannot distinguish omitted from explicit SQL `NULL`; behavior and schemas remain correct.
- **Follow-up:** `PHASE5-EPIC`.

## 2026-08-10 — `DEBT-005/DEBT-006` — Rust quality gates restored

- **Type:** Fix and refactor
- **Implemented by:** Codex
- **Reviewed by:** Codex self-review
- **Summary:** Formatted the existing Rust backend, resolved all strict Clippy findings, removed unnecessary test bindings and mutability, simplified result propagation, and documented intentionally staged Phase 7 APIs with localized lint allowances.
- **Files:** `src-tauri/src/ai/provider.rs`, `src-tauri/src/commands.rs`, `src-tauri/src/db/repositories/conversations.rs`, `src-tauri/src/db/repositories/mod.rs`, `src-tauri/src/db/repositories/notes.rs`, `src-tauri/src/db/repositories/spaces.rs`, `.ai/TODO.md`, `.ai/PROJECT_STATE.md`, `.ai/CHANGELOG.md`, `.ai/SESSION_NOTES.md`
- **Verification:** `cargo fmt --check` — pass; strict Clippy — pass; Rust tests 33/33 — pass; `cargo build` — pass; `pnpm check` — pass with 7/7 tests; `pnpm build` — pass; `git diff --check` — pass.
- **Decisions/deviations:** Kept unfinished-but-planned AI streaming and message lifecycle APIs with narrow `dead_code` annotations because Phase 7 will expose them; no global lint suppression was added.
- **Follow-up:** Audit Phase 3 Spaces and Phase 4 Notes acceptance and regression coverage.

## 2026-08-10 — `PROC-003` — Codex-only engineering workflow

- **Type:** Process and documentation
- **Implemented by:** Codex
- **Reviewed by:** Codex self-review; authorized by project owner
- **Summary:** Replaced the Hermes/Codex dependency with a single-agent Codex workflow. Complex work now uses `planned_codex` task contracts and a distinct evidence-based self-review; small work remains `direct_codex`.
- **Files:** `AGENTS.md`, `WORKFLOW.md`, `docs/decisions/008-codex-only-workflow.md`, `.ai/ARCHITECTURE.md`, `.ai/HANDOFF.md`, `.ai/PROJECT_STATE.md`, `.ai/TODO.md`, `.ai/CHANGELOG.md`, `.ai/SESSION_NOTES.md`
- **Verification:** Active-policy scan contains no Hermes requirements; required control files and Codex-only status vocabulary are present; `git diff --check` passes; changed-path audit contains documentation only.
- **Decisions/deviations:** ADR-008 supersedes ADR-004. Historical Hermes references remain in older changelog entries and ADR-006 as factual history.
- **Follow-up:** Codex selects the next roadmap slice.

## 2026-07-31 — `TECH-001` — Windows DPAPI credential hardening

- **Type:** Security fix and refactor
- **Implemented by:** Codex
- **Reviewed by:** Accepted under the project owner's Codex-only transition; merged via PR #3 on 2026-08-10
- **Summary:** Replaced database-path-derived credential encryption with current-user-bound Windows DPAPI, injected a testable crypto boundary into `Database`, and eliminated the recursive SQLite mutex deadlock.
- **Files:** `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, `src-tauri/src/ai/credentials.rs`, `src-tauri/src/db/mod.rs`, `src-tauri/src/lib.rs`, `.ai/HANDOFF.md`, `.ai/PROJECT_STATE.md`, `.ai/TODO.md`, `.ai/CHANGELOG.md`, `.ai/SESSION_NOTES.md`
- **Verification:** Rust tests 33/33 pass; production Cargo build passes; frontend typecheck/lint pass; Vitest 7/7 pass; changed Rust files are formatted; no new Clippy warnings in changed modules.
- **Decisions/deviations:** Followed ADR-006. Included generated `Cargo.lock` as a necessary consequence of the approved dependency addition. Existing repository-wide formatting and Clippy debt remain out of scope as `DEBT-005` and `DEBT-006`.
- **Follow-up:** None.

## 2026-07-31 — `ENV-001` — Rust MSVC verification restored

- **Type:** Tooling and documentation
- **Implemented by:** Codex
- **Reviewed by:** Not required (`direct_codex`)
- **Summary:** Verified Rust 1.97.1 on the stable MSVC toolchain with Visual Studio 2022 C++ build tools, built both test binaries, ran the Rust quality gates, and replaced the environment blocker with precise code findings.
- **Files:** `.ai/TODO.md`, `.ai/PROJECT_STATE.md`, `.ai/CHANGELOG.md`, `.ai/SESSION_NOTES.md`
- **Verification:** `cargo test --no-run` — pass; 29 non-credential tests — pass; `test_missing_key` — pass; `test_store_and_get` — timeout after 30 seconds; `cargo fmt --check` — fail on pre-existing formatting; strict Clippy — fail on 11 library/12 test-build findings.
- **Decisions/deviations:** Application code was intentionally unchanged. `ENV-001` is complete because local Rust verification is operational; the observed deadlock belongs to the now-unblocked security task `TECH-001`.
- **Follow-up:** `TECH-001`, `DEBT-005`, `DEBT-006`.

## 2026-07-31 — `PROC-002` — Automatic GitHub task publication

- **Type:** Process, tooling, and documentation
- **Implemented by:** Codex
- **Reviewed by:** Merged via PR #1
- **Summary:** Added a versioned post-commit auto-push hook, one-time hook installer, and guarded task publisher that validates scope, blocks common sensitive files, commits, pushes, and creates a draft PR.
- **Files:** `.gitattributes`, `.githooks/post-commit`, `scripts/install-git-hooks.ps1`, `scripts/publish-task.ps1`, `WORKFLOW.md`, `AGENTS.md`, `.ai/ARCHITECTURE.md`, `.ai/PROJECT_STATE.md`, `.ai/CHANGELOG.md`, `.ai/SESSION_NOTES.md`
- **Verification:** PowerShell and shell syntax passed; hook skip-path passed; local hook configuration passed; sensitive-path scan passed; `git diff --check` passed; `pnpm check` passed with 7/7 tests and four pre-existing warnings; automatic pushes passed; draft PR #1 created successfully.
- **Decisions/deviations:** Automatic commits occur only at verified task boundaries; unsafe file-save auto-commits are intentionally prohibited. Initial PR detection treated GitHub's empty JSON array incorrectly; corrected before final publication.
- **Follow-up:** None.

## 2026-07-31 — `PROC-001` — Hybrid Hermes/Codex task routing

- **Type:** Process and documentation
- **Implemented by:** Codex
- **Reviewed by:** Not required (`direct_codex`)
- **Summary:** Added a mandatory routing gate that sends small, clear, low-risk work directly to Codex and reserves the full Hermes planning and review cycle for architectural, security-sensitive, cross-cutting, ambiguous, or product-shaping work.
- **Files:** `WORKFLOW.md`, `AGENTS.md`, `.ai/ARCHITECTURE.md`, `.ai/PROJECT_STATE.md`, `.ai/CHANGELOG.md`, `.ai/SESSION_NOTES.md`
- **Verification:** Nine policy assertions passed; `git diff --check` passed; changed-file audit confirmed exactly the six intended collaboration documents and no application code.
- **Decisions/deviations:** `ADR-004` now defines hybrid rather than universal two-agent routing.
- **Follow-up:** None.

## 2026-07-30 — `AIWF-001` — Hermes/Codex collaboration control plane

- **Type:** Process and documentation
- **Implemented by:** Codex
- **Reviewed by:** Hermes — accepted 2026-07-30
- **Summary:** Added canonical project state, backlog, architecture, handoff, changelog, session memory, and a strict two-agent delivery workflow.
- **Files:** `.ai/HANDOFF.md`, `.ai/PROJECT_STATE.md`, `.ai/TODO.md`, `.ai/ARCHITECTURE.md`, `.ai/CHANGELOG.md`, `.ai/SESSION_NOTES.md`, `WORKFLOW.md`, `AGENTS.md`
- **Verification:** Required files/headings passed; referenced paths passed; `git diff --check` passed; changed-path audit confirmed no application-code changes.
- **Decisions/deviations:** Established one active implementation handoff at a time and explicit source-of-truth ownership.
- **Follow-up:** `DOC-001` promoted as next handoff.

## 2026-07-30 — `DOC-001` — Corrections after Codex review

- **Type:** Documentation
- **Implemented by:** Hermes (corrections); verified by: Codex
- **Reviewed by:** Hermes — accepted 2026-07-30 (re-review after corrections)
- **Summary:** Applied all 6 Codex review findings: bumped Cargo.lock aether version to 0.3.0-alpha (manually — Cargo unavailable), removed trailing whitespace from CHANGELOG.md, changed README Notes to "Markdown notes", updated SESSION_NOTES.md with pnpm check/build results, made HANDOFF.md status/review consistent. Codex independently re-verified: git diff --check clean, pnpm check pass, pnpm build pass.
- **Files:** `src-tauri/Cargo.lock`, `.ai/CHANGELOG.md`, `.ai/HANDOFF.md`, `.ai/SESSION_NOTES.md`, `README.md`
- **Verification:** `git diff --check` — pass. `pnpm check` — pass (typecheck 0 errors, lint 0 errors, test 7/7). `pnpm build` — pass. No application code changed.
- **Decisions/deviations:** Cargo.lock manually corrected — Cargo not available to regenerate. Only aether entry changed; dependency versions untouched.
- **Follow-up:** None — milestone complete.

## 2026-07-30 — `DOC-001` — Version and metadata reconciliation

- **Type:** Documentation
- **Implemented by:** Hermes (per explicit user request)
- **Reviewed by:** Hermes — accepted 2026-07-30
- **Summary:** Reconciled all version fields to `0.3.0-alpha`. Updated README status line and database schema list. Populated `Cargo.toml` repository URL. Resolved `RISK-003` and `DEBT-004`.
- **Files:** `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `README.md`, `.ai/PROJECT_STATE.md`, `.ai/TODO.md`, `.ai/HANDOFF.md`, `.ai/SESSION_NOTES.md`
- **Verification:** `pnpm check` passed (typecheck clean, lint 0 errors, test 7/7). Version consistency confirmed across all manifests. No application code changed.
- **Decisions/deviations:** None.
- **Follow-up:** None — milestone complete.
