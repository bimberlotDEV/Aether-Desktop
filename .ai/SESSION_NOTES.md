# Session Notes

| Field          | Value         |
| -------------- | ------------- |
| Schema version | 2             |
| Session date   | 2026-08-28    |
| Active task    | `RELEASE-040` |
| Agent          | Codex         |
| State          | `self_review` |

## Current work

- PR #43 merged Milestone G into `master` at `da4ca98`; exact PR-head CI and post-merge `master` CI both pass.
- Local `master` and `origin/master` were synchronized with no content diff before branching to `codex/release-040`.
- Classified the integrated 0.4.0 release as `planned_codex` because it changes installer identity and upgrades the owner's live installation/data.
- Audited the 0.3.2 release checklist/artifact record, version locations, updater/signing constraints, and guarded publication workflow.
- Wrote a ready contract requiring full validation, an offline verified pre-install backup, data-preserving silent upgrade, installed startup smoke, traceable hashes, and exact-head CI.
- Passed the readiness gate and started checkpoint 1 on `codex/release-040`.
- Reconciled all current release identifiers to 0.4.0 and corrected the runtime app-data documentation to `%APPDATA%/com.aether.desktop`.
- Passed frozen install, 77 frontend tests, production build/audit, Rust format/strict Clippy, 95 Rust tests, release build, and MSI/NSIS packaging.
- Preserved and integrity-checked the complete live app-data/install backup, silently upgraded from installed 0.3.2 to 0.4.0 with exact database preservation, and started responsive installed Aether 0.4.0.

## Exact resume point

Complete diff/security self-review, publish the verified release patch as a draft PR, wait for exact-head Windows CI, then record final closure evidence without merging unless explicitly authorized.
