# First-run onboarding

Aether onboarding is a local desktop bootstrap, not account creation.

## When it appears

- A genuinely empty desktop database receives one incomplete local profile and opens onboarding before the normal shell.
- A workspace with existing meaningful data and no profile is recognized as an upgrade, receives a completed profile, and opens normally.
- An existing profile is authoritative and initialization is idempotent.
- Browser development opens the normal shell; Settings can launch an honest non-persistent tour.

## Flow

1. Explain the local-first privacy model.
2. Choose Student, Developer, Professional, Personal, or Blank.
3. Confirm the name and editable modules for the first Space.
4. Optionally authorize one local folder through the existing Source picker.
5. Optionally configure DeepSeek or OpenAI through existing DPAPI-protected provider settings.
6. Learn Ctrl+K and enter Pulse.

The presets create ordinary Spaces. They do not create separate app modes, accounts, demo records, or owner-specific content.

## Failure and recovery

- The required Space is created transactionally before onboarding can complete.
- If setup is interrupted after that creation, Aether resumes against the existing active top-level Space and does not create another.
- Optional folder or AI setup can fail, be retried, or be skipped without blocking Notes and Tasks.
- Profile initialization and completion errors remain visible with retry controls.
- The Settings tour never resets completion or deletes data.

## Privacy boundaries

- No folder is chosen automatically and Aether never scans the whole computer.
- Authorizing a Source records the canonical folder through the existing Rust boundary; onboarding does not start a scan or read file contents.
- API keys use existing provider commands and Windows DPAPI. They are never stored in profile/settings rows, frontend persistence, URLs, logs, or backups.
- No telemetry, account identity, email, demographics, employer/school, payment data, or analytics consent is requested.

See ADR-024 for the binding decision.
