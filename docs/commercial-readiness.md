# Commercial readiness

## Status by capability

| Milestone J capability | Repository state | Remaining external/owner gate |
| --- | --- | --- |
| Licensing / entitlements | Architecture defined; no production gate or fake license state | Beta evidence, legal entity, source/product license, plan boundary, account/entitlement authority and offline policy approval |
| Managed AI | Trust/cost/privacy architecture defined; BYOK remains implemented | Account service, provider/data terms, backend, quotas, abuse controls, budgets, support and incident response |
| Windows signing | Protected draft-first workflow, fixed signing wrapper and runbook implemented | Provision Azure Artifact Signing identity/credentials, protected reviewers, publisher verification and signed clean-install evidence |
| Updater | Rust-owned fixed Stable channel, review/approval and signed install implemented | Updater key custody/public key, signed 0.5.0 bootstrap, draft verification and deliberate publication |
| Website | Truthful content/design/release brief defined; no fake site shipped | Domain/trademark/legal identity, policies, signed download URL, hosting/security ownership and publication approval |
| Pricing/payments | Hypotheses remain outside code | Retention/value evidence, plan design, tax/refund/vendor/legal decisions; payments remain prohibited until approved |

This packet means the next decisions are explicit. It does not make Aether commercially ready by itself.

## Entitlement boundary

Future entitlement logic should follow this boundary only after approval:

```text
Owner-controlled catalog and billing
  -> account / entitlement service
  -> signed entitlement document
  -> Rust verification and bounded offline cache
  -> typed capability decisions
  -> UI explains current access and recovery
```

Rules:

1. Product code asks capabilities such as `managed_ai` or a validated limit; it never compares plan names or hardcoded prices throughout components.
2. The server is authoritative for paid remote service. The desktop verifies a signed, versioned, expiry-bounded entitlement and fails safely when authenticity cannot be established.
3. Offline behavior must be declared per capability before implementation. A temporary service outage must not immediately disable ordinary local work.
4. Expiry, cancellation, account loss or entitlement outage may stop premium remote compute, but must never hide/delete local records, block backup/export, prevent restore, or make a workspace unreadable.
5. Downgrades show a clear preview and preserve local user-created data. A future plan may limit new premium operations, never destructively rewrite existing work.
6. No payment card data, provider master key, billing webhook secret or server signing key enters the desktop, SQLite, logs, diagnostics or backup.
7. License/account state needs explicit loading, offline, expired, revoked, error and recovery behavior plus tests. Until that exists, no product surface mentions an active paid plan.

## Commercial implementation order

Do not parallelize these dependencies into speculative code:

1. Complete meaningful Public Beta and freeze owner-approved success/reliability evidence.
2. Decide legal entity, source/product license, privacy/terms, publisher/domain identity and support scope.
3. Decide target plans and capabilities from observed value; approve pricing separately.
4. Select account, entitlement, payment/tax and hosting authorities; threat-model their integration.
5. Implement account and signed entitlement verification behind tests without gating local data.
6. Implement managed AI as a separately consented server service using [its architecture](managed-ai-architecture.md).
7. Implement and security-review the website from [the website brief](website-brief.md).
8. Run a paid/private pilot before broad commercial or 1.0 claims.

Each step requires a new `planned_codex` contract. No step may infer the next owner's legal or financial choice.

## Signing and updater owner work

The code path is already present. Follow [the release runbook](release-runbook.md) rather than adding another delivery system:

- establish protected `public-release` environment/reviewer rules on `master`;
- provision and recoverably custody the Tauri updater key;
- provision least-privilege Azure Artifact Signing credentials and verify publisher identity;
- dispatch the exact committed version, inspect the draft and run clean install/upgrade/in-app update/rollback checks;
- publish deliberately only after all signatures, hashes and data-preservation results pass.

Do not embed trust material, bypass SmartScreen/signature failures, replace published assets, or allow the frontend to choose release endpoints.

## Commercial stop conditions

Stop implementation for unresolved data-loss/privacy/security defects, weak beta retention, unclear product value, unaffordable managed-AI economics, unsigned delivery, unapproved legal documents, ambiguous data processing, missing refund/tax ownership or any design that can hold local data hostage.
