# Agent Implementation Changelog

> Append-only record of completed implementation and collaboration-system changes. It complements, but does not replace, a future product release changelog.

## Responsibility of this file

- Record completed work after verification.
- Capture task ID, scope, files, validation, and notable follow-up.
- Remain append-only except for correcting factual errors.
- Exclude plans, uncompleted work, and raw session transcripts.

## Entry format

Entries are newest first and use ISO dates. Each entry must reference a handoff task ID.

```markdown
## YYYY-MM-DD — `TASK-ID` — Short outcome

- **Type:** Feature | Fix | Refactor | Test | Docs | Process
- **Implemented by:** Codex
- **Reviewed by:** Hermes | Pending
- **Summary:** <observable result>
- **Files:** `<path>`, `<path>`
- **Verification:** `<command>` — Pass/Fail/Not run with reason
- **Decisions/deviations:** None | <concise note>
- **Follow-up:** None | `TASK-ID`
```

## 2026-07-31 — `PROC-002` — Automatic GitHub task publication

- **Type:** Process, tooling, and documentation
- **Implemented by:** Codex
- **Reviewed by:** Draft PR #1 pending merge
- **Summary:** Added a versioned post-commit auto-push hook, one-time hook installer, and guarded task publisher that validates scope, blocks common sensitive files, commits, pushes, and creates a draft PR.
- **Files:** `.gitattributes`, `.githooks/post-commit`, `scripts/install-git-hooks.ps1`, `scripts/publish-task.ps1`, `WORKFLOW.md`, `AGENTS.md`, `.ai/ARCHITECTURE.md`, `.ai/PROJECT_STATE.md`, `.ai/CHANGELOG.md`, `.ai/SESSION_NOTES.md`
- **Verification:** PowerShell and shell syntax passed; hook skip-path passed; local hook configuration passed; sensitive-path scan passed; `git diff --check` passed; `pnpm check` passed with 7/7 tests and four pre-existing warnings; automatic pushes passed; draft PR #1 created successfully.
- **Decisions/deviations:** Automatic commits occur only at verified task boundaries; unsafe file-save auto-commits are intentionally prohibited. Initial PR detection treated GitHub's empty JSON array incorrectly; corrected before final publication.
- **Follow-up:** Merge GitHub PR #1; install the hook in each new clone.

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
