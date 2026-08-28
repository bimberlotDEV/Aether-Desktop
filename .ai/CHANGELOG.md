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

## 2026-08-28 — `BACKUP-RESTORE-001` — Complete portable backup and safe restore

- **Type:** Feature, security, data lifecycle, accessibility, test, and docs
- **Implemented by:** Codex
- **Reviewed by:** Codex self-review
- **Summary:** Added verified portable workspace archives with managed Vault bytes, strict untrusted-archive preview, one-time explicit restore approval, device-local credential preservation, a complete safety archive, and restart-bound database/Vault replacement with interrupted-swap recovery.
- **Files:** `src-tauri/src/backup.rs`, native commands/startup, typed TS bridge, `BackupSettings`, ADR-022, backup/database/release docs, `.ai/*`
- **Verification:** 80 frontend tests; production build/audit; Rust fmt/strict Clippy; 99 Rust tests; release build; MSI/NSIS packaging; 1024×640 and 720×640 light/dark UI smoke; archive/path/secret diff review; exact-head GitHub Actions run `33194814850` — Pass
- **Decisions/deviations:** Restore replaces rather than merges; linked files and credentials are never archived. Startup semantics are exercised with isolated native temporary app data because Windows known-folder resolution and single-instance enforcement prevent a second packaged executable from using an environment-overridden `%APPDATA%` while preserving the owner's live workspace.
- **Follow-up:** Public signing and updater activation require owner-controlled trust infrastructure.

## 2026-08-28 — `RELEASE-040` — Alpha 0.4.0 installed and published

- **Type:** Build, release, data safety, docs, and process
- **Implemented by:** Codex
- **Reviewed by:** Codex self-review
- **Summary:** Cut merged Milestones A–G as Alpha 0.4.0, produced traceable unsigned Windows artifacts, protected the complete existing installation/data, silently upgraded 0.3.2 with exact database preservation, and started the installed release.
- **Files:** Version metadata/UI labels, `README.md`, runtime data-path documentation, `docs/release-artifacts-0.4.0.md`, `.ai/*`
- **Verification:** Frozen install; 77/77 frontend tests; production build/audit; Rust format, strict all-target/all-feature Clippy, 95/95 tests and release build; MSI/NSIS hashes; backup/database integrity and domain counts; silent install; installed 0.4.0 startup; exact-head GitHub Actions run `33122769813` — Pass
- **Decisions/deviations:** Corrected the runtime data path to `%APPDATA%/com.aether.desktop`; release remains unsigned with updater artifacts disabled because owner-controlled trust infrastructure does not exist.
- **Follow-up:** Design restore plus Vault-byte archives as a separately reviewed milestone; signing/updater activation remains owner-controlled.

## 2026-08-27 — `AI-EVOL-001` — Transparent multi-provider AI Evolution

- **Type:** Feature, security, data, migration, accessibility, test, docs
- **Implemented by:** Codex
- **Reviewed by:** Codex self-review
- **Summary:** Added fixed-endpoint DeepSeek/OpenAI support, provider-isolated DPAPI credentials, deterministic transparent Auto routing, per-response provenance, and strictly approved Task/Note AI drafts.
- **Files:** `src-tauri/src/ai/`, `src-tauri/src/db/`, `src-tauri/src/actions.rs`, `src/components/ai/`, `src/routes/AI.tsx`, `src/routes/Settings.tsx`, `docs/decisions/021-ai-provider-routing.md`, `.ai/`
- **Verification:** 77 frontend tests, production build/audit, Rust fmt/strict Clippy, 95 Rust tests, MSI/NSIS packaging, 1024×640 light/dark browser smoke, packaged startup, and exact-head GitHub Actions run `33101730042` — Pass
- **Decisions/deviations:** Fixed official endpoints only; no silent provider fallback, general autonomy, model-owned approval, or weaker direct Task batch mutation route.
- **Follow-up:** None

## 2026-08-27 — `ACTION-001` — Typed user-approved Safe Actions

- **Type:** Feature, security, accessibility, test, and docs
- **Implemented by:** Codex
- **Reviewed by:** Codex self-review
- **Summary:** Added eight closed Action types for Task/Note creation, contained single-folder/file operations, and opening authorized Source targets through a visible preview → approve/cancel → result flow.
- **Files:** `src-tauri/src/actions.rs`, Safe Action commands and Activity integration, typed frontend bridge/hook, `src/routes/Actions.tsx`, tests, ADR-020, architecture/readme, `.ai/*`
- **Verification:** 77/77 frontend tests, production build, clean dependency audit, Rust format, strict Clippy, 85/85 Rust tests, Windows symlink escape and audit-rollback coverage, MSI/NSIS packaging, 1024×640 browser smoke, packaged startup, security/diff review, and GitHub Actions run `33092882774` pass.
- **Decisions/deviations:** No shell, delete, recursive operation, overwrite, arbitrary executable, cross-Source transfer, network action, or model-owned approval exists. Source metadata requires an explicit rescan after file changes.
- **Follow-up:** Milestone G may propose only this existing closed vocabulary under a separately reviewed AI consent boundary.

## 2026-08-27 — `PULSE-002` — Explainable daily Pulse relevance

- **Type:** Feature, privacy, accessibility, test, and docs
- **Implemented by:** Codex
- **Reviewed by:** Codex self-review
- **Summary:** Rebuilt Pulse around one bounded local snapshot with factual Today, Continue, New, Recent, a deterministic suggested next step, and a prominent user-controlled Ask Aether entry point.
- **Files:** `src-tauri/src/db/repositories/pulse.rs`, Pulse command and typed bridge, `src/hooks/usePulse.ts`, `src/routes/Pulse.tsx`, tests, ADR-019, architecture/readme, `.ai/*`
- **Verification:** 72/72 frontend tests, production build, clean dependency audit, Rust format, strict Clippy, 79/79 Rust tests, MSI/NSIS packaging, 1024×640 browser smoke, packaged startup, diff/security review, and GitHub Actions run 33083523023 pass.
- **Decisions/deviations:** Relevance is local, read-only, bounded, factual, and deterministic; archived scopes, removed files, absolute Source roots, raw Activity metadata, hidden AI calls, and mutations are excluded.
- **Follow-up:** `ACTION-001` — build narrow typed Safe Actions with explicit review, approval, one-time execution, and auditability.

## 2026-08-27 — `CONT-001` — Deterministic Space continuity and meaningful Activity

- **Type:** Feature, privacy, accessibility, test, and docs
- **Implemented by:** Codex
- **Reviewed by:** Codex self-review
- **Summary:** Replaced placeholder Activity with a curated local timeline and added a calm Space resume surface composed deterministically from bounded active Notes, Tasks, Source metadata, AI conversations, and meaningful events.
- **Files:** `src-tauri/src/db/repositories/activity.rs`, `src-tauri/src/db/repositories/continuity.rs`, domain commands and typed bridge, `src/routes/Activity.tsx`, `src/routes/SpaceDetail.tsx`, tests, ADR-018, architecture/readme, `.ai/*`
- **Verification:** 69/69 frontend tests, production build, clean dependency audit, Rust format, strict Clippy, 77/77 Rust tests, MSI/NSIS packaging, browser and packaged-startup smoke, diff/security review, and GitHub Actions run 33071798193 attempt 2 pass.
- **Decisions/deviations:** Continuity remains local, deterministic, Space-isolated, metadata-bounded, and module-aware; it performs no DeepSeek call, content extraction, telemetry, or autonomous mutation. The first CI attempt only hit the unchanged Vault test's known fixed-timeout flake.
- **Follow-up:** Run the integrated Milestones A–D audit; restore, Vault-byte archives, public signing, and updater activation remain separately governed release work.

## 2026-08-26 — `SEARCH-001` — Private local Universal Search

- **Type:** Feature, privacy, accessibility, test, and docs
- **Implemented by:** Codex
- **Reviewed by:** Codex self-review
- **Summary:** Turned Ctrl+K into a bounded Universal Search across commands and permitted local Spaces, Notes, Tasks, Vault metadata, explicit Memory, AI conversation titles, meaningful Activity, and present Source-file metadata with deterministic ranking and provenance.
- **Files:** `src-tauri/src/db/repositories/search.rs`, search command and typed bridge, `src/components/CommandPalette.tsx`, tests, ADR-017, architecture/readme, `.ai/*`
- **Verification:** 64/64 frontend tests, production build, dependency audit, Rust format, strict Clippy, 73/73 Rust tests, MSI/NSIS packaging, browser-mode and packaged-startup smoke, final diff/security review, and GitHub Actions run 32983855580 attempt 2 pass.
- **Decisions/deviations:** Search is conventional and entirely local: Notes use FTS5, other domains use bounded parameterized metadata matching, Source results never expose absolute child paths or content, and no query invokes AI or mutates data.
- **Follow-up:** `CONT-001` — build deterministic Space continuity and a meaningful Activity timeline.

## 2026-08-26 — `CTX-001` — Explicit local Sources and safe metadata indexing

- **Type:** Feature, privacy, data, test, and docs
- **Implemented by:** Codex
- **Reviewed by:** Codex self-review
- **Summary:** Added explicit, revocable local-directory Sources with optional Space ownership, bounded metadata-only scanning, transactional change and rename reconciliation, inspectable relative-path indexes, and a dedicated accessible desktop route.
- **Files:** `src-tauri/src/context.rs`, Source migration/repository/commands, typed frontend bridge, `src/routes/Sources.tsx`, tests, ADR-016, architecture/readme, `.ai/*`
- **Verification:** 60/60 frontend tests, production build, dependency audit, Rust format, strict Clippy, 70/70 Rust tests, MSI/NSIS packaging, browser-mode smoke, packaged startup smoke, SQLite migration/integrity inspection, and GitHub Actions runs 32972614553 and 32973076347 pass.
- **Decisions/deviations:** Native Windows picker interaction could not be programmatically driven without enabling a prohibited WebView debug boundary. The supported picker integration is covered at the React boundary, while canonical authorization, real temporary-directory scanning, revocation, and filesystem non-mutation are covered in Rust. A consistent pre-test SQLite backup was created with SQLite's backup API; no Source remained authorized in the owner's database.
- **Follow-up:** `SEARCH-001` — design Universal Search over explicit local domains without expanding AI access.

## 2026-08-25 — `RELEASE-032` — Alpha 0.3.2 candidate installed and published

- **Type:** Build, release, docs, and process
- **Implemented by:** Codex
- **Reviewed by:** Codex self-review
- **Summary:** Consolidated merged AI reconciliation, personal-beta migration safety, and the interface redesign into version 0.3.2; produced reproducible Windows artifacts, protected the existing local data with an offline backup, installed the upgrade, and published draft PR #37.
- **Files:** Version metadata and labels, `CHANGELOG.md`, `README.md`, `docs/native-desktop.md`, `docs/release-artifacts-0.3.2.md`, `.ai/*`
- **Verification:** Frozen installation, 56/56 frontend tests, production build, clean high-severity dependency audit, Rust format, strict Clippy, 63/63 Rust tests, release build, MSI/NSIS packaging, artifact hashes, byte-for-byte database preservation, installed-process smoke, and GitHub Actions run 32894361551 pass.
- **Decisions/deviations:** Artifacts remain explicitly unsigned, automatic updating remains disabled, and no GitHub Release was published. Draft PR #37 remains open for owner-controlled merge.
- **Follow-up:** Design full restore and Vault-byte archive semantics as a separate reviewed task; configure signing/updating only with owner-controlled trust infrastructure.

## 2026-08-25 — `UI-001` — Aether interface system reimagined

- **Type:** Feature, design system, accessibility, and test
- **Implemented by:** Codex
- **Reviewed by:** Codex self-review
- **Summary:** Rebuilt Aether's complete frontend into a calm, layered, distinctly futuristic Windows workspace with a stronger shell, meaningful Pulse cockpit, coherent route compositions, richer real empty/populated states, and consistent dark/light interactions.
- **Files:** `src/styles/index.css`, `src/components/ui/AetherUI.tsx`, shell, product routes, dialogs, component tests, ADR-015, `.ai/*`
- **Verification:** Standalone frontend gates pass with 54/54 tests. The combined local candidate with PRs #34/#35 passes 56/56 tests, production build, dependency audit, Windows MSI/NSIS build, backup, silent installation, and responsive installed-app startup. Browser visual QA covers all core routes, empty/populated states, dark/light themes, dialogs, command palette, and 1024×640 through 1600×900.
- **Decisions/deviations:** No new dependencies, backend behavior, fake data, gradient/glow/glass decoration, or persistence changes were introduced. The combined build was installed locally while each PR remains independently reviewable.
- **Follow-up:** None.

## 2026-08-25 — `HARD-001` — Personal-beta databases upgrade without losing managed files

- **Type:** Fix, migration hardening, and test
- **Implemented by:** Codex
- **Reviewed by:** Codex self-review
- **Summary:** Added transactional compatibility upgrades for the earlier Tasks, Memory, and Vault schemas. Legacy managed Vault bytes are canonically validated, copied through a crash-repeatable no-overwrite path into current ownership storage, and retained at their original location for recovery.
- **Files:** `src-tauri/src/db/migrations.rs`, `.ai/*`
- **Verification:** 53/53 standalone and 55/55 combined frontend tests pass; 63/63 Rust tests, strict Clippy, formatting, production builds, dependency audit, MSI/NSIS packaging, combined installed startup, SQLite integrity, and foreign-key checks pass.
- **Decisions/deviations:** Corrupt, missing, size-mismatched, path-unsafe, or conflicting legacy Vault blobs block the upgrade rather than being silently discarded or associated with another file. Original valid blobs remain as recovery copies, trading temporary disk usage for rollback safety.
- **Follow-up:** Continue Milestone A with the next evidence-backed stability defect; begin Context Foundation only after Alpha-hardening exit criteria are defined and satisfied.

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
