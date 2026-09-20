# Codex Task Contract

| Field             | Value                                   |
| ----------------- | --------------------------------------- |
| Schema version    | 2                                       |
| Task ID           | `ONBOARD-001`                          |
| Status            | `accepted`                             |
| Owner             | Codex                                   |
| Last updated      | 2026-08-29                              |
| Related milestone | Canonical Milestone H — Onboarding & UX |
| Classification    | `planned_codex`                        |

## Objective

Give genuinely new Aether users a calm, short, local-first first-run experience that creates a useful first Space, optionally authorizes one folder and configures AI through existing trusted boundaries, teaches Ctrl+K, and enters Pulse—without interrupting upgraded workspaces or introducing fake setup state.

## Context

The schema has supported one local profile and `onboarding_completed` since migration 1, but the application never reads that state and no onboarding UI exists. Existing 0.4.0 workspaces commonly have data but no profile, so naïvely showing onboarding whenever the profile is absent would disrupt upgrades and risk duplicate starter content. Existing Space templates, explicit Source authorization, DPAPI-backed provider settings, and the Aether interface system should be reused rather than duplicated.

## Ordered checkpoints

1. **First-run boundary:** add a Rust repository/command operation that creates the singleton profile and decides whether onboarding is required; any pre-existing meaningful workspace automatically bypasses onboarding without changing its domain data.
2. **Guided setup:** add a full-window, keyboard-accessible onboarding flow for welcome/privacy, usage template, first Space, optional folder authorization, optional existing AI settings, Ctrl+K education, and completion.
3. **Safe orchestration:** create the first Space through existing transactional repositories, authorize Sources through the existing canonical path boundary, store AI keys only through existing DPAPI commands, and mark completion only after required setup succeeds.
4. **Recovery and polish:** allow retry after every failure, avoid duplicate Spaces after an interrupted setup, provide honest browser fallback, preserve theme/responsive behavior, and let Settings restart onboarding intentionally without deleting data.
5. **Verification and publication:** cover backend first-run/upgrade semantics plus frontend success, skip, retry, resume, focus and privacy language; run full quality/package/UI/security gates, self-review, document, and publish a stacked draft PR.

## Acceptance criteria

- [x] A fresh desktop database gets exactly one incomplete local profile and sees onboarding; an existing workspace with no profile gets one completed profile and never sees onboarding solely because it upgraded.
- [x] Existing profile state is authoritative, initialization is idempotent, and no onboarding command deletes, rewrites, exports, scans, or sends existing user content.
- [x] The flow is concise and covers local-first privacy, Student/Developer/Professional/Personal/Blank usage, first Space naming, editable modules, optional Source access, optional AI setup, Ctrl+K, and entry into Pulse.
- [x] Templates configure ordinary editable Space modules rather than creating separate applications; no owner-specific, fake, or placeholder content is inserted.
- [x] Required first-Space creation uses existing transactional Rust repositories; retries/resume cannot create duplicate starter Spaces, and onboarding is not marked complete before the required Space exists.
- [x] Folder access remains optional, explicit, canonicalized and revocable through the existing Source boundary; onboarding never silently scans the computer or sends Source data to AI.
- [x] AI setup remains optional and reuses the existing provider/DPAPI boundary; API keys are never persisted in React, logs, profile/settings rows, URL state, or exports.
- [x] Every step supports keyboard navigation, visible focus, meaningful headings/labels, status or alert announcements, back/skip where safe, and responsive dark/light layouts at supported desktop sizes.
- [x] Browser development remains usable with an honest preview/bypass path, while packaged desktop startup exercises the real profile gate.
- [x] Settings can intentionally reopen onboarding as a tour without clearing `onboarding_completed`, creating duplicate Spaces, or changing data unless the user explicitly performs an onboarding action.
- [x] Documentation and canonical A–K status distinguish Milestone H from the already completed release/updater foundation; no milestone is claimed complete without evidence.
- [x] Frozen install, frontend typecheck/lint/tests/build/audit, Rust fmt/strict all-target/all-feature Clippy/tests/check/release build, Tauri MSI/NSIS packaging, responsive theme/keyboard/native startup smoke, diff/secret/capability/data-safety review, and exact-head GitHub CI pass.

## Allowed paths

- `src/App.tsx`
- `src/App.test.tsx`
- `src/components/Onboarding.tsx`
- `src/components/Onboarding.test.tsx`
- `src/components/ai/AiSettings.tsx`
- `src/components/spaceOptions.ts`
- `src/routes/Settings.tsx`
- `src/routes/Settings.test.tsx`
- `src/styles/index.css`
- `src/lib/db/types.ts`
- `src/lib/db/tauri.ts`
- `src/lib/db/tauri.test.ts`
- `src-tauri/src/db/repositories/profile.rs`
- `src-tauri/src/commands.rs`
- `src-tauri/src/lib.rs`
- `README.md`
- `IDEA.md`
- `docs/architecture.md`
- `docs/database.md`
- `docs/onboarding.md`
- `docs/decisions/024-first-run-onboarding.md`
- `.ai/*`

## Non-goals

- Accounts, cloud profiles, sync, telemetry, remote templates, a template marketplace, automatically reading files, managed AI subscriptions, payments, licensing, public release, or 1.0 claims.
- Replacing the existing Space creation modal, Source management, provider settings, theme system, shell, or Pulse after onboarding.
- Collecting legal name, email, school/employer, precise location, demographics, analytics consent, payment data, or other unnecessary personal information.
- Deleting or resetting data when the tour is reopened.

## Risks and safeguards

- **Upgrade interruption:** backend initialization checks meaningful persisted rows before creating the profile and marks legacy workspaces complete atomically.
- **Duplicate setup:** an incomplete profile with an existing active top-level Space resumes against that Space; required creation is never repeated blindly.
- **Partial optional setup:** the required Space is created first; optional Source/AI failures remain retryable or skippable and completion is explicit.
- **Secret exposure:** AI uses the existing credential commands only; no new credential storage or transport is introduced.
- **Filesystem overreach:** the existing picker and Source command remain the only authorization path; no default folders or background full-PC scans are added.
- **UI lockout:** initialization errors show a retry surface; browser mode bypasses native persistence honestly; reopened tours are non-destructive.
- **Scope drift:** H prepares external users only; public beta operations, commercial entitlements, website, pricing, and launch evidence remain later contracts.

## Required validation

```text
pnpm install --frozen-lockfile
pnpm check
pnpm build
pnpm audit --audit-level high
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all-targets --all-features
cargo check --manifest-path src-tauri/Cargo.toml --all-targets --all-features
cargo build --manifest-path src-tauri/Cargo.toml --release
pnpm tauri:build
fresh/legacy/interrupted profile initialization tests
responsive dark/light and keyboard onboarding smoke
packaged first-start and existing-workspace bypass smoke
git diff --check and secret/capability/data-safety review
GitHub PR exact-head CI
```

## Blocking decisions

None. The five usage choices, module presets, optional permissions, no-telemetry default, existing-user bypass, and non-destructive tour behavior follow the owner-approved masterprompt and existing architecture.

## Readiness review

- **Status:** Ready. Required outcomes, legacy behavior, privacy/security boundaries, recovery semantics, test evidence, allowed scope, and non-goals are explicit.
- **Architecture gate:** ADR-024 is accepted before production implementation.
- **Data gate:** no migration is required; the existing singleton profile column is used and legacy data is only detected, never rewritten.
- **Security gate:** Sources and AI credentials reuse existing native boundaries; React gains no raw SQL, broad permission, filesystem scan, or secret persistence.
- **Worktree gate:** the branch is stacked on the pushed, CI-green release/updater draft; unrelated frontend status entries are line-ending/index noise with no content diff and will not be staged.

## Implementation evidence and self-review

- Rust profile initialization is idempotent and uses existing domain rows only to decide whether a legacy workspace should bypass onboarding; 105/105 Rust tests pass, including fresh and populated database cases.
- The React gate, six-step setup, interrupted-resume handling, non-destructive tour, optional Source authorization, optional DPAPI-backed AI settings, retry surfaces and focus behavior are covered by 94/94 frontend tests.
- Browser smoke covered every step plus light/dark and 1280×800/720×640 layouts. The packaged release executable remained running against the existing workspace and MSI/NSIS packaging completed.
- Frozen install, typecheck, lint, production build, production audit, Rust fmt, strict Clippy, Cargo check/test/release build, Tauri packaging and `git diff --check` pass. No capability expansion, raw SQL in React, secret persistence, background scan, or destructive data operation was introduced.
- Self-review mapped each acceptance criterion to tests or direct inspection. GitHub Actions run `33254872286` passed on exact published head `2019c27`.
