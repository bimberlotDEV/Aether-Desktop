# Trusted Windows release runbook

This runbook is the owner-controlled boundary between a verified Aether commit and a public Windows release. The workflow creates a **draft only**. Do not publish when any gate is missing, skipped, or ambiguous.

## Trust model

Aether uses two independent signatures:

1. **Windows Authenticode** identifies the publisher to Windows and covers the executable/installers. The workflow uses Azure Artifact Signing through `scripts/sign-windows-artifact.ps1`.
2. **Tauri updater signing** lets installed Aether clients reject modified or unauthorized update bundles. The public key ships in the generated release configuration; the private key exists only as a protected release secret.

GitHub hosting and TLS distribute metadata, but neither substitutes for either signature. React never chooses an endpoint or installer. Rust checks the fixed Stable feed, retains the pending update, verifies its bundle signature, and installs only after a separate user action.

## One-time owner provisioning

1. Create the GitHub Environment `public-release`.
   - Require an owner reviewer.
   - Restrict deployment branches to `master`.
   - Do not allow unreviewed administrators to bypass protection.
2. Generate the Tauri updater key outside the repository with `pnpm tauri signer generate`.
   - Protect it with a strong unique password.
   - Store the private key and password in two independent encrypted recovery locations.
   - Store `TAURI_SIGNING_PRIVATE_KEY`, `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`, and the matching complete `.pub` content as `AETHER_UPDATER_PUBLIC_KEY` Environment secrets.
3. Provision modern Azure Artifact Signing and grant the release service principal only the required signing roles.
   - Secrets: `AZURE_CLIENT_ID`, `AZURE_CLIENT_SECRET`, `AZURE_TENANT_ID`.
   - Environment variables: `AZURE_ARTIFACT_SIGNING_ENDPOINT`, `AZURE_ARTIFACT_SIGNING_ACCOUNT`, `AZURE_ARTIFACT_SIGNING_PROFILE`.
   - The endpoint must be HTTPS under `*.codesigning.azure.net`; account/profile names are strictly validated.
4. Record the expected Authenticode certificate subject, issuer, expiry, updater public-key fingerprint, and recovery custodians in an owner-private register. Do not put private values in Git, issues, logs, or release notes.

## Prepare a candidate

1. Update numeric SemVer consistently in `package.json`, `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml`, Cargo.lock, Settings, README, and CHANGELOG.
2. Merge the implementation PR and require green post-merge `master` CI.
3. Confirm the version has never been published. Never replace assets on an already published version.
4. Create and verify a complete `.aether-backup` of the upgrade-test workspace.
5. In GitHub Actions, select **Public Windows release**, choose `master`, enter the exact numeric version, and dispatch it. Approve the protected Environment only after confirming the commit SHA.

The workflow must fail when a trust input is absent, versions disagree, the tree is dirty, quality gates fail, signing fails, Authenticode is invalid, or updater signatures are absent. Its only external result is a draft `aether-v<version>` release.

## Inspect the draft

1. Confirm the draft tag/target is the intended `master` SHA and release notes match the candidate.
2. Confirm MSI, NSIS, v2 updater archives, `.sig` files, and `latest.json` exist; reject duplicate, unexpected, debug, portable, or cross-platform assets.
3. Download assets onto a separate Windows test account and record SHA-256 hashes.
4. Run `Get-AuthenticodeSignature` on every `.exe` and `.msi`; require `Status = Valid` and the exact private-register publisher identity.
5. Inspect `latest.json`: require the exact version, HTTPS GitHub asset URLs, Windows x86_64 target, non-empty signature text, and release notes with no secrets or private paths.
6. Verify the updater archive signature with the registered public key. Modify one byte in a copy and prove verification rejects it.

## Product acceptance matrix

Run all rows before publishing:

| Scenario | Required evidence |
| --- | --- |
| Clean per-user install | Installer identity is valid, no SmartScreen unknown-publisher result, launch succeeds, and a fresh database is created only in `%APPDATA%/com.aether.desktop`. |
| Upgrade from previous Stable | Complete backup exists; installer preserves database/Vault hashes and domain counts; migrations/integrity/FK checks pass; app starts responsive. |
| In-app Stable check | Previous Stable shows the intended version, date, notes, and explicit consequence; no download begins during check/review. |
| Dismiss | Dismissed pending update does not download/install and its token cannot later be reused. |
| Signed install | Progress is visible, signature verifies before installer launch, passive Windows install runs, and Aether restarts into the exact new version. |
| Tampered artifact | Modified archive/signature is rejected and the installed version/data remain unchanged. |
| Offline/server error | The user sees an actionable retry message; no raw stack, partial installer, or data mutation remains. |
| Backup/restore regression | Complete backup preview and an isolated restore round-trip preserve managed Vault bytes and omit credentials/linked bytes. |
| Uninstall | Behavior matches documented retention expectations; no unrelated user files are removed. |

Verify keyboard operation, visible focus, screen-reader names/live progress, and light/dark/system themes at 960×600, 1024×640, and a normal desktop size.

## Publish

Only the owner publishes the inspected draft. Immediately afterward:

1. Confirm the release is visible and `releases/latest/download/latest.json` returns the inspected metadata over HTTPS.
2. Check from the previous Stable client again and perform one final signed update on non-personal test data.
3. Record final asset hashes, certificate identity, updater-key fingerprint, workflow run, commit SHA, backup location, and acceptance evidence in the versioned release artifact document.
4. Keep the previous Stable installers and verified complete backup available for recovery.

## Rollback and incident response

- **Product defect:** do not replace or delete the published release. Publish a higher patch version signed by the same trust roots. Use the prior installer only for an explicitly tested manual rollback because the updater does not support downgrades.
- **Failed rollout before publication:** keep the draft private, record the failed gate, fix on a new commit, and rerun. Never publish a partially verified draft.
- **Windows signing credential compromise:** disable/revoke the Azure identity, stop releases, rotate credentials/certificate, and communicate the publisher transition before resuming.
- **Updater private key suspected compromised:** stop publication immediately. If the old key is still controlled, ship a signed intermediate client that trusts the new public key, then retire the old key. If the key is lost or cannot be trusted, in-app continuity is broken and users must manually install a newly Authenticode-signed bootstrap release.
- **GitHub compromise without updater-key compromise:** remove malicious assets/releases and rotate GitHub credentials; installed clients still reject artifacts lacking the valid updater signature, but metadata may be misleading, so communicate the incident.
- **Data regression:** stop rollout, preserve logs without secrets, use the complete pre-upgrade backup and recovery installer, and do not attempt automatic downgrade/restore on users' workspaces.
