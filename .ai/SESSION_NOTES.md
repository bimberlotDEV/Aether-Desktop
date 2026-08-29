# Session Notes

| Field          | Value          |
| -------------- | -------------- |
| Schema version | 2              |
| Session date   | 2026-08-29     |
| Active task    | `ONBOARD-001`  |
| Agent          | Codex          |
| State          | `in_review`    |

## Current work

- Implemented canonical Milestone H on `codex/onboarding-ux`; commit `db4a25b` is published in draft PR #47 and depends on draft PR #46.
- Fresh databases receive one incomplete profile; meaningful legacy workspaces receive a completed profile and bypass setup without domain-data mutation.
- Added the six-step local-first onboarding, resumable Space setup, optional Source/AI setup, non-destructive Settings tour, recovery states, tests and documentation.
- Local frontend, Rust, browser, production audit, release packaging and packaged startup checks are green; exact-head GitHub CI is the remaining H gate.

## Exact resume point

Push this publication-state update to trigger exact-head CI on draft PR #47, wait for it to pass, then record H closure and begin the Public Beta task contract.
