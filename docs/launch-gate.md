# Aether 1.0 launch gate

## Rule

Do not call Aether 1.0 until retention and reliability justify it. A version bump, green CI, polished interface, signed installer or download count is not enough.

Before evaluating results, the owner records numeric thresholds for every item marked **owner threshold** below. Freeze the cohort definition and observation window so the decision cannot be reverse-engineered around favorable outcomes.

## Eligible evidence

- Real external testers using their own meaningful, non-demo work after informed beta consent.
- Anonymous tester codes and privacy-minimal outcomes from `beta-evidence-template.md`.
- Only testers eligible for the whole D2/D7/D14/D30 window enter that retention denominator.
- Duplicate installations, owner/developer use, automated tests and appearance-only reviewers do not count.
- Manual evidence is acceptable because Aether has no telemetry, but unknown/missing outcomes are not converted to success.

## Required gates

### Product value

- **Activation threshold (owner threshold):** proportion creating a real Space plus real Note/Task/File and returning to it.
- **Retention thresholds (owner thresholds):** D2, D7, D14 and D30 among eligible testers.
- **Core behavior threshold (owner threshold):** meaningful use of Pulse, Spaces and Search/Continue.
- **Differentiation threshold (owner threshold):** meaningful Source/Context/AI/Memory/Action use with an explicit reason it was better than opening the prior tool alone.
- **Replacement signal (owner threshold):** users voluntarily report reduced reliance on ChatGPT, Notion, Obsidian, Todoist, Explorer or another workflow.
- Qualitative evidence includes trust with real projects, voluntary return, recommendation, noticeable absence and requests for more access. “Looks nice” is insufficient.

### Reliability and safety

- Zero unresolved stop-ship defect: data loss/corruption, secret/privacy exposure, path escape, unsafe file mutation, signature/update bypass or unrecoverable install/upgrade/restore failure.
- **Crash/startup-free threshold (owner threshold):** defined from reproducible tester sessions/reports with unknown outcomes handled explicitly.
- All supported clean-install, protected-upgrade, in-app-update, backup/restore, rollback and uninstall scenarios pass on signed artifacts.
- Database integrity, Space isolation, Memory/context scope, AI confirmation, Vault ownership and Source containment regressions have deterministic automated coverage.
- No unresolved high-severity core-journey defect beyond the owner-approved maximum (recommended maximum: zero at launch candidate freeze).

### Delivery, legal and operations

- Release comes from protected `master`, has verified Authenticode and updater signatures, matching publisher/version/hash and a tested rollback/higher-patch path.
- Legal entity/product owner, source/product license, privacy, terms, acceptable use, provider/subprocessor and support decisions are approved.
- Website download points only to the exact signed release and shows requirements, privacy, limitations and backup guidance.
- Support/security channels, severity owner, credential/key recovery, domain/hosting recovery and incident communication are rehearsed.
- Managed AI, if offered, passes entitlement, quota, cost, abuse, privacy, deletion, provider-outage and billing-reconciliation tests. Otherwise it is absent, not labelled “coming soon” inside the app.

## Decision outcomes

| Outcome | Meaning | Action |
| --- | --- | --- |
| Pass | Every gate and frozen numeric threshold passes with an eligible cohort | Create a separately reviewed 1.0 release contract; rerun all gates on the exact version. |
| Extend beta | Safety is sound but value/retention evidence is insufficient or cohort window incomplete | Continue testing/targeted product repair; do not lower thresholds post hoc. |
| Stop launch | Any stop-ship, signing, recovery, legal, managed-AI cost/privacy or support blocker exists | Halt distribution/implementation as appropriate and repair the blocker first. |

## 1.0 release checklist after gate pass

1. Freeze scope; no opportunistic feature additions.
2. Resolve every launch-blocking issue and map it to regression evidence.
3. Reconcile version manifests, changelog, website, policies and supported-version statements.
4. Create a complete owner-held recovery backup and rehearse signing/update/domain key recovery.
5. Run local/CI/package/security/accessibility/performance/install/upgrade/update/restore/uninstall gates on the exact commit.
6. Generate a signed draft, verify clean machines and previous Stable, then obtain deliberate owner approval.
7. Publish gradually, monitor privacy-safe support signals, retain rollback/higher-patch capacity and stop rollout on a stop-ship signal.

Nothing in this document authorizes the 1.0 claim today. Its purpose is to make the remaining proof falsifiable and difficult to waive casually.
