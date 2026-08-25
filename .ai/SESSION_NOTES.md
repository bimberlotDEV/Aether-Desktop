# Session Notes

| Field          | Value      |
| -------------- | ---------- |
| Schema version | 2          |
| Session date   | 2026-08-25 |
| Active task    | `HARD-001` |
| Agent          | Codex      |
| State          | `ready`    |

## Current work

- Adopted the owner's Aether Product Evolution Masterprompt as the ordered product roadmap.
- Classified the next work as `planned_codex` and selected the proven personal-beta database incompatibility as the first Milestone A hardening checkpoint.
- Moved the existing local migration prototype onto `codex/alpha-legacy-upgrade`; no additional production code changed before the readiness gate.
- Created a complete task contract covering populated schema upgrades, rollback, Memory scope, Vault bytes/path safety, installer replacement, and publication.

## Safety boundary

- Preserve the owner's existing database backup and current application data.
- Do not guess legacy Vault byte locations or weaken current managed-path containment.
- Do not send live provider messages or expose credentials during this migration task.
- Keep the already published AI response-sync work isolated in PR #34.

## Exact resume point

The `HARD-001` contract is `ready`. Inspect historical migrations, repository invariants, and legacy managed-file behavior before revising the existing migration prototype.
