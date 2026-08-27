# Session Notes

| Field          | Value         |
| -------------- | ------------- |
| Schema version | 2             |
| Session date   | 2026-08-27    |
| Active task    | `AI-EVOL-001` |
| Agent          | Codex         |
| State          | `complete`    |

## Current work

- Milestone F merged through PR #42 at `51fd6d9`; local `master` and `origin/master` were synchronized and clean.
- Classified Milestone G as `planned_codex` because it changes provider, credential, migration, privacy, routing, and Action consent boundaries.
- Inspected the existing provider trait, DeepSeek-specific creation/validation, DPAPI secret store, conversation persistence, explicit context resolver, Safe Actions, and current AI UI.
- Verified current official DeepSeek and OpenAI Chat Completions/SSE contracts. Arbitrary endpoints are excluded.
- Accepted ADR-021 and completed the `AI-EVOL-001` ready contract before production changes.
- Implemented and self-reviewed provider-isolated credentials, fixed-endpoint DeepSeek/OpenAI adapters, deterministic Auto routing, per-response provenance, and server-reconstructed Task/Note Safe Action drafts.
- All local release gates, package/startup smoke, responsive browser smoke, and exact-head GitHub Actions run `33101730042` pass; draft PR #43 is published.

## Exact resume point

Milestone G is complete. Inspect the next prioritized roadmap item before opening a new task contract; merge draft PR #43 only with explicit owner approval.
