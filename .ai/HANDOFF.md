# Codex Task Contract

| Field             | Value                                                |
| ----------------- | ---------------------------------------------------- |
| Schema version    | 2                                                    |
| Task ID           | `COMM-LAUNCH-001`                                    |
| Status            | `in_review`                                          |
| Owner             | Codex                                                |
| Last updated      | 2026-08-29                                           |
| Related milestone | Canonical Milestones J and K — Commercial/1.0 gates |
| Classification    | `planned_codex`                                      |

## Objective

Take Milestones J and K to the furthest responsible repository-owned state while Milestone I awaits real external evidence: define a maintainable entitlement/managed-AI/signing/updater/website architecture, surface every legal/business/operations decision, and establish objective 1.0 retention/reliability gates. Do not ship unused entitlement code, finalize pricing, change legal licenses, create accounts/payments, publish a website, or claim launch readiness.

## Context

Signing/updater implementation already exists and repository beta operations are complete, but no signed beta cohort or retention evidence exists. The masterprompt explicitly forbids payments and final pricing before beta retention is validated and permits 1.0 only when retention/reliability justify it. Licensing, managed AI and website publication require owner identity, legal, provider, financial and infrastructure choices that cannot be inferred from code.

## Ordered checkpoints

1. Reconcile which J capabilities already exist and which are genuinely blocked.
2. Define entitlement and managed-AI trust boundaries without production placeholders or feature gating.
3. Produce a truthful public website/content brief with no fake availability, pricing, testimonials or data claims.
4. Produce an owner decision register with recommended sequencing, inputs and consequences.
5. Define measurable K launch gates and rollback rules tied to private beta evidence, then publish the verified readiness packet.

## Acceptance criteria

- [x] Commercial-readiness documentation inventories licensing, managed AI, signing, updater and website status without presenting plans as implemented.
- [x] Entitlement architecture keeps pricing/catalog data out of feature logic, uses a server-authoritative signed entitlement with bounded offline behavior, defaults safely, and does not lock existing local data on expiry/outage.
- [x] Managed-AI architecture keeps provider credentials server-side, requires accounts/consent/quotas/abuse controls/cost limits/retention policy/incident response, and preserves BYOK/local-first separation.
- [x] The signing/updater section points to the implemented protected workflow and enumerates only remaining owner trust and release operations.
- [x] A website brief contains approved target users/problem/solution/proof/privacy/download states and forbids fake testimonials, final pricing, waitlist collection or unsigned downloads.
- [x] The owner decision register identifies legal entity, source/product license, terms/privacy, publisher/domain identity, pricing evidence, entitlement authority, account/payment/tax vendors, managed-AI provider/data terms, support/SLA and metrics thresholds.
- [x] The 1.0 launch gate requires owner-approved numeric retention/reliability thresholds, sufficient eligible cohort/time windows, signed install/update/backup evidence, zero stop-ship issues, support readiness and a rollback plan.
- [x] Every unavailable capability remains absent from production UI/code; no dead buttons, dormant billing SDK, fake license state, hardcoded price or managed-AI endpoint is added.
- [ ] Documentation links and terminology validate, `git diff --check` passes, all existing product gates remain green, and exact-head GitHub CI passes.

## Allowed paths

- `docs/commercial-readiness.md`
- `docs/managed-ai-architecture.md`
- `docs/website-brief.md`
- `docs/owner-decision-register.md`
- `docs/launch-gate.md`
- `docs/decisions/026-commercial-and-launch-gates.md`
- `README.md`
- `.ai/*`

## Non-goals

- License/terms/privacy-policy changes with legal effect; company formation; trademark/domain/certificate purchases; signing/provider/payment/account infrastructure; tax/refund decisions.
- Pricing finalization, plan limits, paywalls, trial logic, license validation, accounts, payment SDKs/webhooks, managed-AI runtime, telemetry or cloud sync.
- Website implementation/hosting/publication, mailing-list collection, public download, release publication or a 1.0 version bump.
- Inventing beta, retention, revenue, reliability, support or conversion evidence.

## Risks and safeguards

- **Premature monetization:** J remains blocked until Milestone I evidence and explicit owner decisions exist; documents specify boundaries only.
- **Data hostage:** future entitlement loss may disable premium compute/service but never hide or delete local user data or block export.
- **Managed-AI exposure/cost:** desktop never receives service/provider master keys; quotas, budgets and abuse containment are server-side requirements.
- **Marketing misrepresentation:** website brief uses candidate/download states and prohibits fake social proof or unsigned releases.
- **Vanity launch:** K gates require numeric owner-approved evidence and eligible time windows rather than appearance feedback or download counts.

## Required validation

```text
pnpm check
pnpm build
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all-features
git diff --check
documentation link/status/forbidden-claim review
GitHub PR exact-head CI
```

## Blocking decisions

The readiness packet itself is not blocked. Actual J/K implementation is blocked by the owner decisions and Milestone I evidence enumerated in the decision register. Those are deliberately not inferred.

## Readiness review

- **Status:** Ready. This is a documentation/architecture checkpoint, not premature product implementation.
- **Architecture gate:** ADR-026 is accepted before readiness documents are written.
- **Legal/business gate:** no legally operative or commercial choice is made by Codex.
- **Worktree gate:** branch `codex/commercial-launch-gates` begins at CI-green beta-readiness head `a94e4dd`; content-identical line-ending/index noise remains unstaged.

## Implementation evidence and self-review

- `commercial-readiness.md` maps every J capability to implemented repository state versus owner/external gate and defines data-preserving signed-entitlement sequencing.
- `managed-ai-architecture.md` defines explicit request/consent, server-held credentials, quotas/budgets/abuse/retention/incident boundaries and honest local degradation while preserving BYOK.
- `website-brief.md` defines the approved audience/message, real proof, release-state actions, security/accessibility and prohibitions against unsigned downloads, final prices, fake proof or unowned data collection.
- `owner-decision-register.md` enumerates 14 ordered decisions; `launch-gate.md` requires pre-frozen numeric cohort, activation, D2/D7/D14/D30, differentiation, reliability, signing/recovery/legal/support evidence and zero stop-ship issues.
- No production source, dependency, capability, price, license, endpoint or version changed. All local links resolve, forbidden-claim review and `git diff --check` pass; 100/100 frontend and 108/108 Rust tests, build, fmt and strict Clippy remain green. Exact-head CI remains.
