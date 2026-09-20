# Public beta test matrix

Run with neutral test data. Destructive and restore scenarios require an isolated Windows account or disposable VM plus a verified pre-test `.aether-backup`. Record build hash, Windows version, result and issue URL—never private content or paths.

| Area | Required scenario | Pass condition |
| --- | --- | --- |
| Trust | Download signed installer and inspect Authenticode | Expected publisher, valid chain, no unsigned/unknown-publisher result. |
| Clean install | Install MSI and NSIS separately on clean Windows 10/11 x64 accounts | Start menu/app identity correct; first launch creates only Aether app data and shows onboarding. |
| Onboarding | Complete each persona across the tester set; skip optional Source/AI | Exactly one editable Space, no fake content, Pulse entry works, optional steps remain optional. |
| Existing upgrade | Back up a populated supported build, install candidate | Existing workspace bypasses onboarding; notes/tasks/Vault/Memory/AI history remain valid. |
| Stable update | Manually check, review, approve and install a newer signed candidate | Fixed feed only; signature verified; visible progress; restart into exact version; data preserved. |
| Spaces/Notes/Tasks | Create, edit, search, archive/restore and restart | State persists; Space isolation and autosave/revision behavior remain correct. |
| Vault | Exercise linked and managed imports, open/reveal, removal and restart | Linked originals never deleted; managed ownership remains contained. |
| Sources/Search/Continue | Authorize one test folder, scan metadata, search, revoke | No whole-PC scan/content extraction; results are scoped and revoked data disappears. |
| AI/Memory | Configure one provider, attach explicit context, stream/cancel/retry, approve/reject drafts | Key stays hidden; provenance/scope visible; no mutation before separate approval. |
| Actions | Preview, cancel and approve safe database/file actions | Reviewed arguments are immutable; traversal/overwrite/delete remain rejected; audit is factual. |
| Backup/restore | Create complete archive, inspect preview, cancel, then restore isolated data | Credentials/linked bytes excluded; managed bytes/counts preserved; safety backup and restart succeed. |
| Native | Tray, global shortcut, notification, window state and single instance | Each reports honest availability and behaves consistently across two launches. |
| Diagnostics | Generate and inspect sanitized report | Only eight documented fields; no content, paths, logs, identifiers, counts or keys. |
| Accessibility | Keyboard-only core flows, focus, zoom/contrast, light/dark | No keyboard trap; focus visible; names/states announced; supported minimum window remains usable. |
| Failure/recovery | Cancel dialogs, deny notification, provider offline, invalid backup/update | Clear bounded error, retry/cancel path, no partial mutation or data loss. |
| Uninstall | Uninstall after backup and document Windows prompt choices | No unrelated or linked files removed; retained local data behavior matches the documented owner decision. |

## Stop conditions

Immediately stop the tester wave for suspected data loss/corruption, secret exposure, path escape, signature/update failure, unrecoverable startup failure or destructive restore/upgrade behavior. Preserve the affected signed installer and hashes, do not collect private workspace files, and follow the severity process in [beta-program.md](beta-program.md).
