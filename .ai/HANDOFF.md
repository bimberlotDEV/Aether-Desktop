# Codex Task Contract

| Field             | Value                     |
| ----------------- | ------------------------- |
| Schema version    | 2                         |
| Task ID           | `STAB-001`                |
| Status            | `complete`                |
| Owner             | Codex                     |
| Last updated      | 2026-08-11                |
| Related milestone | Alpha 0.3.0 stabilization |
| Classification    | `planned_codex`           |

## Objective

Stress-test the complete Aether alpha across automated, browser, desktop, native, responsive, accessibility, data-lifecycle, and error-path behavior; repair reproducible in-scope defects and add regression coverage.

## Context

The Phase 10 release gates pass, but the owner reports remaining runtime and interface bugs. This task validates the product as an integrated application rather than relying only on unit and build gates.

## Acceptance criteria

- [x] Frontend and Rust quality gates pass from a clean task branch.
- [x] All routed product surfaces are exercised in light, dark, and system themes at desktop and compact window sizes.
- [x] Core create, edit, search/filter, archive/restore, and deletion flows are exercised with clearly prefixed test fixtures where supported.
- [x] Keyboard navigation, focus behavior, dialogs, empty/loading/error states, and rapid repeated interactions are checked.
- [x] The packaged or development desktop app is smoke-tested for startup, navigation, window behavior, and native integrations without altering unrelated user data.
- [x] Every reproducible in-scope defect is fixed, covered by the closest practical regression test, and reverified.
- [x] Remaining limitations are explicit and no P0/P1 data-loss, security, crash, or unusable-flow defect remains open.
- [x] The final diff passes self-review and is ready for commit, push, and GitHub pull request publication.

## Allowed paths

- `src/**`
- `src-tauri/**`
- `tests/**` when added for stabilization coverage
- `package.json`, `pnpm-lock.yaml`, and test/build configuration only when required for verification
- `.ai/**` and user-facing quality/release documentation

## Non-goals

- New product features or roadmap expansion.
- Public code signing, updater activation, or production credentials.
- Destructive testing against pre-existing user records or Vault files.
- Broad visual redesign; UI changes must correct verified usability, accessibility, layout, or consistency defects.

## Risks and safeguards

- Existing local data may be present: use browser-mode mocks or uniquely prefixed `STAB-001` fixtures and delete only task-owned fixtures after confirmation requirements are satisfied.
- Stress tests can expose timing-sensitive behavior: capture a deterministic reproduction before changing code.
- Native app close-to-tray behavior can leave a process running: target only the verified Aether process/window during cleanup.
- Rollback is the task branch; no database migration will be added unless a proven defect cannot be fixed safely without one.

## Required validation

```text
pnpm check
pnpm build
pnpm audit --audit-level high
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
cargo build --manifest-path src-tauri/Cargo.toml
git diff --check
manual browser route/theme/responsive/keyboard smoke matrix
manual Windows desktop/native smoke matrix
```

## Self-review record

- **Status:** Pass — no unresolved P0/P1 findings.
- **Findings repaired:** unknown-route blank screen; missing render recovery; stored Space icon mismatch; inaccessible Space/Note rows and action menus; incomplete dialog semantics and Escape behavior; browser-mode Space module/school hierarchy loss; dead browser-mode Note workflow; stale global Notes refresh; and multiple concurrent desktop instances.
- **Acceptance mapping:** 50/50 frontend tests and 61/61 Rust tests pass; TypeScript, lint, production frontend build, dependency audit, Rust format, strict Clippy, Rust build, Tauri release build, route/theme/responsive/keyboard matrices, desktop navigation, and single-instance release smoke all pass.
- **Data safety:** Existing Spaces, Notes, Vault files, database contents, and credentials were not modified or deleted.
- **Known release limitations:** Alpha remains unsigned and has no active auto-updater; live provider requests were not generated during stress testing to avoid using private credentials or incurring external usage.
