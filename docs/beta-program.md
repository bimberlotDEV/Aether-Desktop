# Aether public beta operations

## Current status

The repository can produce a **beta candidate**. It is not a public beta release until the owner creates a signed draft through the protected workflow, completes the release runbook, deliberately publishes it, and enrolls real external testers. Ordinary local MSI/NSIS bundles are unsigned development artifacts.

Supported beta platform: Windows 10 or 11, x64. Aether has no account, cloud sync, remote support agent or telemetry.

## Who should test

Recruit a small, meaningful mix of students, developers, knowledge workers and local-first power users who will use real projects and can tolerate beta risk. Do not recruit people who cannot independently maintain a backup or who need regulated, shared, mobile or business-critical workflows.

Before participation, tell each tester:

- this is prerelease software with no availability or data-recovery guarantee;
- Aether stores workspace data locally and has no hidden analytics;
- AI is optional and beta users supply a supported DeepSeek or OpenAI key;
- only prompts and explicitly attached context go to the selected provider;
- the owner records only consented, privacy-minimal test outcomes under an anonymous tester code;
- testers must inspect anything they paste into GitHub and use private vulnerability reporting for security issues.

## Enrollment and release

1. Assign an anonymous tester code unrelated to name or email.
2. Record informed beta consent privately; do not commit the roster.
3. Create and verify a complete `.aether-backup` before an upgrade scenario.
4. Follow [the release runbook](release-runbook.md) to create a signed draft from `master`.
5. Complete every applicable row in [the beta test matrix](beta-test-matrix.md) on a clean Windows account and a protected previous-version workspace.
6. Publish only after signing identity, hashes, backup/restore, clean install, upgrade, in-app update, uninstall and rollback evidence pass.
7. Provide testers the signed release page, hash and known limitations. Never distribute a local unsigned bundle as the public beta.

## Feedback and diagnostics

Use the repository's **Beta bug report** for reproducible defects and **Beta product feedback** for workflow/value feedback. Settings → Beta support can create a sanitized report with only version, schema, integrity, generic platform and capability booleans. Aether displays it before copying and never sends it.

Never request or accept in a public issue:

- `.db`, `.aether-backup`, credential, key or environment files;
- notes, prompts, Memory, AI responses or document contents;
- private filenames, local paths, usernames, machine identifiers or full logs;
- exploit details or suspected secret exposure.

Security reports belong in GitHub private vulnerability reporting.

## Severity and response

| Severity | Meaning | Action |
| --- | --- | --- |
| Stop-ship | Data loss/corruption, secret/privacy exposure, signature/update bypass, unsafe file mutation, install/upgrade failure | Pause distribution, preserve the affected build and sanitized facts, use private reporting where applicable, repair and ship a higher signed patch. |
| High | A core journey is blocked with no safe workaround | Triage before the next tester wave; add a deterministic regression test. |
| Medium | Reliability, accessibility or significant UX defect with a workaround | Prioritize by frequency and impact. |
| Low | Cosmetic issue or narrow improvement | Batch only when it improves clarity or daily usefulness. |

Do not delete or replace a published release to hide a defect. Stop rollout, keep evidence, and publish a higher signed version after verification.

## Known limitations

- Windows x64 only; no macOS, Linux, mobile or browser product.
- No cloud sync, collaboration, accounts, managed AI, calendar integration or plugin marketplace.
- AI requires a tester-provided supported provider key; provider availability and cost are external.
- Sources index bounded file metadata only; Aether does not parse general file content.
- Vault has ownership-safe linked/managed storage but not a universal document previewer.
- Memory is explicitly user-controlled; Aether does not silently learn from activity.
- No automatic telemetry or crash upload. Reproduction depends on tester reports.
- Upgrades from an unsigned pre-0.5.0 build need a one-time signed installer before Stable updates can work.

## Exit criteria

Repository readiness is necessary but not sufficient. Milestone I completes only after signed beta distribution and meaningful external use demonstrate:

- clean install, protected upgrade, update, backup/restore and uninstall reliability;
- no unresolved stop-ship issue;
- multiple testers activate with a real Space and real Note/Task/File;
- voluntary D2 and D7 return signals, with D14/D30 tracked when time permits;
- actual use of Pulse, Spaces, Search/Continue and at least one differentiating context/AI workflow;
- candid evidence of what Aether replaces or fails to replace.

Use [the privacy-minimal evidence template](beta-evidence-template.md). Compliments about appearance alone are not an exit signal.
