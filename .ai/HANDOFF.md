# Codex Task Contract

| Field             | Value                               |
| ----------------- | ----------------------------------- |
| Schema version    | 2                                   |
| Task ID           | `BETA-001`                          |
| Status            | `in_review`                         |
| Owner             | Codex                               |
| Last updated      | 2026-08-29                          |
| Related milestone | Canonical Milestone I — Public Beta |
| Classification    | `planned_codex`                     |

## Objective

Make Aether operationally ready for meaningful external Windows testing: testers can understand the risk and privacy model, exercise the product, capture a strictly sanitized local diagnostic summary, report defects without exposing private content, recover data, and follow a repeatable beta test matrix. Do not claim the beta has happened until signed distribution and real external evidence exist.

## Context

Milestones A–H and the owner-gated signing/updater foundation are implemented. The repository lacks beta-specific support and feedback intake, production-safe diagnostics, a maintained known-limitations/test matrix, and current security documentation. Aether intentionally has no telemetry or account service, so beta evidence must be explicitly user-reported rather than silently collected.

## Ordered checkpoints

1. **Truthful beta surface:** reconcile version/security/support/privacy documentation and distinguish an unsigned candidate from a signed public beta.
2. **Sanitized diagnostics:** expose a Rust-owned, bounded report containing only app/schema/platform/integrity/native-capability state; never content, paths, identifiers, credentials, logs, counts, prompts, filenames, or provider keys.
3. **User-controlled sharing:** show the report in Settings with explicit privacy language and a copy action; generation and copying remain local and never submit data automatically.
4. **External test operations:** add issue forms, beta handbook, test matrix, known limitations, rollback/support/severity guidance, and a manual evidence ledger for activation/retention/core-behavior signals.
5. **Verification/publication:** test the Rust/IPC/UI privacy boundary, run full gates and package smoke, self-review, publish a draft PR, and record which external gates remain.

## Acceptance criteria

- [x] A diagnostic report is constructed in Rust from a closed schema and contains only Aether version, database schema version, SQLite integrity result, Windows platform label, updater configured state, and native capability booleans.
- [x] Diagnostic generation does not query domain tables, enumerate rows/files, read logs, expose paths, persist identifiers, contact a network, or include user content/secrets.
- [x] Settings displays the exact report before a user-initiated copy; browser mode is honest, copy success/failure is announced, and no report is automatically transmitted or stored.
- [x] Frontend and Rust tests reject accidental field expansion and verify the invoke boundary, loading, error, copy and unsupported-browser states.
- [x] GitHub issue forms request reproducible technical details while warning against credentials, databases, notes, prompts, filenames, personal paths and security disclosures.
- [x] A beta handbook defines supported Windows versions, candidate trust/signing status, install/upgrade/backup/restore/uninstall checks, severity triage, feedback route, rollback, known limitations and owner-controlled release gates.
- [x] The beta evidence template records tester consent, anonymous tester code, build/version, scenario outcomes, activation, D2/D7/D14/D30 return signals and qualitative replacement behavior without telemetry or private content.
- [x] `SECURITY.md`, `README.md`, release checklist and changelog accurately describe 0.5.0, both AI providers, complete backups, signing/updater state, correct app-data path and vulnerability reporting.
- [x] No payments, accounts, telemetry, cloud backend, hidden analytics, automatic issue submission, crash upload, pricing decision or public-release claim is introduced.
- [ ] Frozen install, frontend check/build/audit, Rust fmt/strict Clippy/check/tests/release build, MSI/NSIS packaging, browser/native smoke, diff/secret/capability/privacy review and exact-head CI pass.

## Allowed paths

- `src/components/BetaDiagnostics.tsx`
- `src/components/BetaDiagnostics.test.tsx`
- `src/routes/Settings.tsx`
- `src/routes/Settings.test.tsx`
- `src/lib/db/types.ts`
- `src/lib/db/tauri.ts`
- `src/lib/db/tauri.test.ts`
- `src-tauri/src/diagnostics.rs`
- `src-tauri/src/commands.rs`
- `src-tauri/src/lib.rs`
- `.github/ISSUE_TEMPLATE/*`
- `README.md`
- `SECURITY.md`
- `CHANGELOG.md`
- `docs/beta-program.md`
- `docs/beta-test-matrix.md`
- `docs/beta-evidence-template.md`
- `docs/release-checklist.md`
- `docs/architecture.md`
- `docs/decisions/025-public-beta-operations.md`
- `.ai/*`

## Non-goals

- Publishing/distributing an installer, enrolling testers, signing binaries, provisioning secrets, accepting legal terms, offering an SLA, collecting actual retention data, or declaring Public Beta complete.
- Automatic telemetry, crash upload, full log export, database attachment, remote support access, analytics identifiers, accounts, payments, licensing or managed AI.
- Product feature expansion unrelated to external testing and reliability.

## Risks and safeguards

- **Privacy leakage:** diagnostics use an allowlisted serialized struct and a structural test that fails if fields expand; no generic environment/log/database dump is used.
- **False release claim:** docs call the output a beta candidate until owner signing, runbook and tester evidence pass.
- **Unsafe support intake:** issue forms prohibit sensitive attachments and route vulnerabilities to private reporting.
- **Silent analytics drift:** evidence collection is an owner-maintained local template, not application instrumentation.
- **Data loss during testing:** the matrix requires complete backup validation before upgrade/restore/uninstall scenarios and preserves rollback artifacts.

## Required validation

```text
pnpm install --frozen-lockfile
pnpm check
pnpm build
pnpm audit --prod
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
cargo check --manifest-path src-tauri/Cargo.toml --all-targets --all-features
cargo test --manifest-path src-tauri/Cargo.toml --all-features
pnpm tauri:build
browser light/dark/keyboard/copy/error diagnostic smoke
packaged startup smoke
git diff --check and secret/capability/privacy review
GitHub PR exact-head CI
```

## Blocking decisions

Repository implementation has no blocking decision. Actual beta launch remains blocked by owner-provisioned signing trust, protected workflow approval, signed clean-install/update testing, deliberate release publication, tester recruitment/consent and real evidence.

## Readiness review

- **Status:** Ready. Repository beta scope, privacy boundary, external evidence gap, recovery expectations and non-goals are explicit.
- **Architecture gate:** ADR-025 is accepted before production changes.
- **Data gate:** diagnostics read integrity/schema metadata only; no migration or domain query is required.
- **Security gate:** no new Tauri capability, network endpoint, file write or secret flow is introduced.
- **Worktree gate:** branch `codex/public-beta-readiness` starts at CI-green H head `228a345`; unrelated status entries remain content-identical line-ending/index noise and will not be staged.

## Implementation evidence and self-review

- Rust diagnostics serialize exactly eight primitive allowlisted fields; 108/108 Rust tests include structural privacy, schema/integrity and no-nested-value checks. The module performs only `_migrations` metadata and `PRAGMA quick_check` queries.
- TypeScript validates the native object with a strict Zod schema and rejects extra fields. Five component tests cover browser honesty, generation, loading, exact preview-before-copy, generation failure and clipboard failure; the complete frontend suite passes 100/100 tests.
- Settings light/dark browser smoke at 1280×800 and the supported 960×600 minimum shows readable, keyboard-discoverable beta support without generic dashboards or new styling primitives.
- Issue forms, handbook, test matrix and evidence template enforce privacy-safe support, severity/stop conditions, real tester signals and the distinction between repository readiness and a real beta.
- Frozen install, production build/audit, Rust fmt/strict Clippy/check/tests, MSI/NSIS packaging, packaged startup, YAML/format/diff/capability/privacy review pass locally. Exact-head GitHub CI remains.
