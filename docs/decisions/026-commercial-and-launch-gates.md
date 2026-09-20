# ADR-026: Commercial and 1.0 evidence gates

## Status

Accepted — 2026-08-29

## Context

The roadmap names licensing, managed AI, signing, updater and website as Commercial Readiness, followed by 1.0 only after retention and reliability justify it. Signing/updater code exists, but the other capabilities require beta evidence and owner-controlled legal, financial, infrastructure and product decisions. Adding dormant paywalls or managed-AI mocks would create fake functionality and lock the product into unvalidated assumptions.

## Decision

1. While Public Beta lacks external evidence, Aether records decision-ready commercial and launch architectures but does not implement entitlements, accounts, payments, managed AI or final pricing.
2. Future entitlements are server-authoritative and signed, with a bounded offline cache. Loss of entitlement may stop premium remote service but never hide/delete local data or block backup/export.
3. Managed AI is a separate server-mediated service. Provider master keys, budgets, abuse controls and service logs never enter the desktop; BYOK remains a distinct local credential path.
4. Signing and updater trust continue through the existing protected draft-first workflow. Owner provisioning and release verification cannot be replaced by application logic.
5. Website publication waits for owner identity/domain/legal/download decisions and a signed release. Copy must remain truthful and contain no invented testimonials, metrics or pricing.
6. 1.0 requires owner-approved numeric activation/retention/reliability thresholds, an eligible cohort observed for the required windows, verified signed delivery/backup recovery, no stop-ship issue and operational support/rollback readiness.

## Consequences

- Commercial implementation starts later but on validated product and legal assumptions.
- Current local-first users cannot be held hostage by future licensing outages.
- Aether may remain a strong personal beta longer instead of making an unsupported 1.0 claim.
- Any production entitlement, account, payment or managed-AI change requires a new planned contract, threat model and explicit owner decisions.
