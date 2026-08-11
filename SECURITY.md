# Security policy

## Supported version

Aether 0.3.1 is alpha software. Only the latest commit on `master` is supported; no public security SLA is offered yet.

## Reporting a vulnerability

Do not open a public issue containing credentials, private data, or exploit details. Use GitHub's private vulnerability reporting for this repository. Include affected versions, reproduction steps, impact, and a minimal proof of concept where safe.

## Security and privacy model

- Workspace records are stored locally in `%APPDATA%/Aether/aether.db`.
- A DeepSeek API key is protected with Windows DPAPI for the current Windows user.
- AI is opt-in. Only the prompt and context items explicitly attached to a conversation are sent to DeepSeek.
- Aether has no telemetry or account service.
- Workspace database exports omit API credentials. They include Vault metadata but not managed or externally linked file contents.
- Tauri capabilities and the content security policy intentionally expose only the native operations used by the product.

Official alpha installers are not code-signed yet. Verify hashes from the release record before running them. Auto-update remains disabled until an owner-controlled signing key and trusted endpoint are available.
