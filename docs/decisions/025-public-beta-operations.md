# ADR-025: Privacy-safe public beta operations

## Status

Accepted — 2026-08-29

## Context

Aether needs meaningful external Windows testing, but has no account or telemetry service and contains deeply private local work. A generic diagnostic dump, automatic crash upload, database attachment or hidden analytics would conflict with its local-first promise. Signing, publishing and tester participation also occur outside the repository and cannot be inferred from a green build.

## Decision

1. The application exposes one Rust-owned diagnostic structure with a closed allowlist: app version, database schema version, SQLite integrity result, generic Windows platform label, updater configuration and native capability booleans.
2. Diagnostics never include user records or counts, file/database/log paths, machine/user identifiers, environment variables, filenames, prompts, API/provider keys, logs or timestamps. Structural tests fail on field expansion.
3. Aether displays the exact report locally before an explicit user copy action. It has no submission endpoint and performs no automatic upload.
4. External beta evidence is collected manually in a privacy-minimal template using anonymous tester codes and scenario outcomes. Product telemetry remains absent.
5. Repository readiness and actual Public Beta are separate states. Actual completion requires signed distribution, clean install/upgrade evidence and meaningful real tester use.

## Consequences

- Support receives less ambient data, but users keep control and reports are easier to inspect.
- Reproduction steps and explicit test matrices matter more than automated dashboards.
- Adding any diagnostic field, telemetry, remote submission or crash upload requires a new privacy/security review.
- A green beta-candidate PR cannot by itself complete canonical Milestone I.

## Alternatives rejected

- Full log/database bundles: likely to contain private content, paths or secrets.
- Automatic crash reporting or analytics: creates identifiers, consent and backend obligations before they are justified.
- Frontend-composed environment dumps: weakens the Rust trust boundary and encourages accidental expansion.
