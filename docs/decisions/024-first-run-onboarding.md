# ADR-024 — Upgrade-safe first-run onboarding

- **Status:** Accepted
- **Date:** 2026-08-29
- **Decision owner:** Codex

## Context

Aether has persisted `user_profile.onboarding_completed` since its first schema, but no application path uses it. New users therefore land in an empty Pulse without guidance. Existing installations often contain real workspace data and no profile, so treating a missing profile as a fresh install would interrupt upgrades and encourage duplicate starter Spaces.

The approved product flow asks briefly for a usage template, first Space, optional Source access, Ctrl+K education, optional AI, and entry into Pulse. It must retain local-first privacy and reuse existing trusted boundaries.

## Options considered

1. Always show a dismissible frontend modal and store completion in localStorage.
2. Create demo Spaces/content during migration and assume every missing profile is new.
3. Initialize one durable local profile in Rust, distinguish an empty database from a meaningful existing workspace, and gate the shell with a recoverable frontend flow that reuses Space, Source, and DPAPI provider commands.
4. Require accounts or cloud onboarding before the local app can open.

## Decision

Choose option 3.

- Rust repository code owns idempotent profile initialization and legacy-workspace detection.
- A missing profile in an empty workspace is created incomplete. A missing profile beside meaningful persisted domain rows is created completed, preserving the existing user's startup experience.
- The frontend gates only the normal shell; initialization failure offers retry and never falls through to a misleading empty workspace.
- Usage choices are local template presets for editable Space modules. They do not create distinct product modes or fake content.
- Required first-Space creation uses the existing transactional Space commands. An interrupted incomplete profile resumes with an already-created top-level Space rather than creating another.
- Optional folder authorization uses the existing native picker and Source command and performs no silent global scan. Optional provider setup reuses the existing DPAPI-backed settings component.
- Completion is stored only in the local profile after the required Space exists. Reopening onboarding from Settings is a non-destructive tour and does not reset completion.
- Browser development shows an honest preview path without pretending native persistence or permissions exist.

## Consequences

- No database migration is required, but startup gains one local repository read and, only when absent, one profile insert.
- Existing users are protected even though earlier releases never created a profile.
- Optional setup can be skipped and revisited in Sources or Settings; onboarding does not become an account or permissions wall.
- Real first-launch behavior requires packaged/native validation in addition to component tests.

## Evidence

- Task contract: `.ai/HANDOFF.md` (`ONBOARD-001`)
- Planned implementation: profile initialization repository/command, `src/components/Onboarding.tsx`, App shell gate, Settings tour entry, tests and onboarding documentation
