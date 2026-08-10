# Codex Task Contract

| Field | Value |
| --- | --- |
| Schema version | 2 |
| Task ID | `PHASE10-001` |
| Status | `complete` |
| Owner | Codex |
| Last updated | 2026-08-10 |
| Related milestone | Phase 10 — Quality and release preparation |

## Objective

Close the alpha with an evidence-backed accessibility, security, performance, error/empty-state, Windows, installer, backup/export, versioning, documentation, and CI audit.

## Acceptance criteria

- [x] Windows CI enforces frontend and Rust gates on pushes and PRs.
- [x] A sanitized, consistent workspace database export is available from Settings and tested.
- [x] Accessibility and keyboard/focus behavior receive automated and manual-code review coverage.
- [x] Dependency, secret, CSP/capability, path, credential, and destructive-flow security checks pass or have explicit release blockers.
- [x] Bundle size, database bounds, streaming, loading/error/empty states, and startup are reviewed.
- [x] README, license, architecture, database, native, backup, release, version, and changelog documentation match reality.
- [x] Final frontend/Rust gates, installer builds, hashes, executable startup, and GitHub state are verified.
- [x] No fake content, secrets, databases, build artifacts, or unresolved P0/P1 work is published.

## Out of scope

- Public code signing and auto-update activation require owner-controlled signing secrets and a release endpoint.
- Automatic backup restore and managed-file archive import require a later conflict/version design.
