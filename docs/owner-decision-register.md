# Owner decision register for Milestones J and K

Make these decisions in order. Record final decisions in owner-private/legal systems where appropriate; never put secrets, personal legal documents, credentials, private contracts or tester identities in Git.

| ID | Required owner decision | Recommended default until evidence exists | Evidence/input required | Blocks |
| --- | --- | --- | --- | --- |
| `OWN-01` | Legal entity, product owner and jurisdictions | Do not sell or promise an SLA before professional legal/tax review | Owner identity, target markets, counsel/accountant advice | Terms, payments, publisher identity, managed AI |
| `OWN-02` | Aether name/domain/trademark and Windows publisher identity | Verify availability and keep one consistent public identity | Brand/domain search, certificate/signing eligibility | Website, signing, contracts |
| `OWN-03` | Source license versus commercial product license | Keep current MIT unchanged until counsel/owner explicitly chooses; never silently revoke granted rights | Business model, contributor/IP ownership, legal review | Licensing, website/legal copy |
| `OWN-04` | Terms, privacy notice, acceptable use, data deletion and support promise | No commercial launch before approval; preserve no-hidden-telemetry/local-first claims | Data-flow inventory, providers/subprocessors, target regions | Accounts, managed AI, website, 1.0 |
| `OWN-05` | Public Beta cohort and success thresholds | Set numeric thresholds before reviewing final outcomes | Beta matrix/evidence, eligible cohort/window | J implementation, K |
| `OWN-06` | Free/BYOK/managed-AI plan capability boundaries | Keep all current local functionality available during beta; derive gates from demonstrated value | Activation/retention/replacement evidence and support cost | Entitlements, pricing |
| `OWN-07` | Pricing, trial, refund and annual-plan policy | Treat €5.99/€11.99 only as hypotheses; do not publish | Willingness-to-pay interviews, retention, AI unit economics, tax/refund advice | Website, billing |
| `OWN-08` | Account, entitlement and payment/tax authorities | Prefer proven managed services with export/recovery and least data collection | Vendor security/privacy/region/cost/lock-in review | Login, licenses, payments |
| `OWN-09` | Managed-AI provider/model/region/retention/training terms | Keep BYOK as working path until terms, caps and kill switches are approved | Provider DPA/terms, quality evals, unit cost, abuse/support plan | Managed AI |
| `OWN-10` | Managed-AI budgets, quotas and incident owner | Hard server-side ceilings and deny-by-default overage | Load/cost tests, plan economics, on-call capacity | Managed AI launch |
| `OWN-11` | Signing key custody and protected GitHub/Azure roles | Two independent updater-key recovery copies; least privilege; owner approval for publication | Runbook rehearsal, recovery test, publisher verification | Public release/update |
| `OWN-12` | Website domain/hosting/security and data collection | Static, tracker-free, no waitlist until a real privacy-owned backend exists | Domain/hosting ownership, threat model, policy approval | Website publication |
| `OWN-13` | Support channels, severity response and SLA | Public issues for privacy-safe bugs; private security reporting; no response-time SLA during beta | Owner capacity, escalation/incident plan | Paid pilot, 1.0 |
| `OWN-14` | 1.0 numeric retention/reliability thresholds and rollout size | Freeze before judging launch evidence; downloads/appearance do not count | Eligible D2/D7/D14/D30 cohort, reliability/support data | Version 1.0/public claim |

## Decision quality rules

- A decision is not complete because a vendor has a popular SDK; document data, cost, failure, recovery and exit paths.
- Any choice that changes local data access, privacy, license rights, pricing or provider processing requires explicit owner approval.
- When beta evidence is weak, the correct decision is to extend beta or repair product value—not relax thresholds after seeing results.
- Codex may implement a chosen reversible architecture under a new contract; it may not choose the owner's legal entity, accept provider/payment terms, buy services, set final prices or publish releases/sites.
