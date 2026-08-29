# Security policy

## Supported version

Aether is prerelease software. Only the latest published signed candidate is supported for external testing; until one exists, only the latest commit on `master` receives fixes. No public security or response-time SLA is offered.

## Reporting a vulnerability

Do not open a public issue containing exploit details, credentials, private data, database/backup files, paths or logs. Use [GitHub private vulnerability reporting](https://github.com/bimberlotDEV/Aether-Desktop/security/advisories/new). Include affected version, impact and minimal reproduction steps using neutral test data.

## Security and privacy model

- Workspace records are stored locally in `%APPDATA%/com.aether.desktop/aether.db`.
- DeepSeek and OpenAI API keys are separately encrypted with Windows DPAPI for the current Windows user and are never returned to React.
- AI is opt-in. Only the prompt, conversation history and context items explicitly attached by the user are sent to the provider shown on the response.
- Aether has no account service, cloud sync, hidden telemetry or automatic crash upload.
- Sources require explicit folder authorization and index bounded metadata only. Revoking a Source removes its index without modifying user files.
- Linked Vault files remain user-owned; managed files are contained below Aether's app-data directory and use ownership-aware removal.
- Complete `.aether-backup` archives include a sanitized SQLite snapshot and Aether-managed Vault bytes. They exclude credentials and externally linked file bytes; restore uses preview, separate approval, validation, a safety backup and restart-time swap recovery.
- Safe Actions and AI drafts require a visible preview and separate one-time approval. React cannot replace reviewed arguments, run shell commands, traverse outside authorized Sources, overwrite or delete user files.
- The beta diagnostic report contains only app/schema/platform/integrity and native-capability state. It contains no content, paths, logs, identifiers, counts or keys and is never submitted automatically.
- Tauri capabilities and CSP expose only the native operations used by the product. Ordinary development builds contain no updater endpoint or trust key and make no update request.

## Release trust

Local `pnpm tauri:build` MSI/NSIS artifacts are unsigned development builds. Public candidates must be created from `master` by the protected manual workflow with independent Authenticode and Tauri updater trust. The workflow emits a draft release; an owner must complete `docs/release-runbook.md` before publication. Never bypass signing or distribute an unsigned local bundle as the public beta.

If a signed release has a security or data defect, stop rollout, preserve the affected artifact and hashes, and ship a higher signed version after repair. Do not silently replace a published artifact.
