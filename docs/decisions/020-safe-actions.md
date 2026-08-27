# ADR-020 — Typed preview-and-approval Safe Actions

- **Status:** Accepted
- **Date:** 2026-08-27
- **Task:** `ACTION-001`

## Context

Aether can read explicitly permitted local context, but must not become a general autonomous computer agent. Useful writes and native operations need one narrow authority model before AI evolution can propose them safely.

## Decision

Rust owns a closed Action enum, validation, preview, pending proposal state, execution, and audit. Preview returns a presentation-safe summary plus an opaque random token stored only in a process-local pending map. Approval calls execution with that token only; it is removed before work and expires after ten minutes.

The initial vocabulary covers Task/Note creation; single-folder creation; single-file copy, move, or rename within the same explicitly authorized Source; and opening a Source file/folder. Relative path components, canonical roots, existing sources, destination parents, symlink containment, file type, and no-overwrite are validated in Rust. There is no delete, recursive operation, shell, arbitrary executable, cross-Source transfer, network action, background automation, or model-owned approval.

Successful writes emit a curated `action_executed` Activity record. Database writes and audit are transactional. Reversible filesystem writes roll back if audit persistence fails.

## Consequences

- Review and execution cannot diverge because execution accepts only the stored proposal token.
- Restarting Aether safely discards unapproved proposals.
- The webview has a useful but deliberately narrow capability surface.
- Later AI work may propose this same vocabulary but cannot bypass human approval.
- Removing the runtime/route/commands rolls back the feature without schema cleanup.
