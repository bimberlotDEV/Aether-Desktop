# Session Notes

> Temporary, replaceable working memory for the current or most recent engineering session.

| Field | Value |
| --- | --- |
| Schema version | 1 |
| Session date | 2026-07-31 |
| Active task | `TECH-001` (planning) |
| Agent | Hermes |
| Route | `hermes_codex` |
| State | `ready` |

## Session objective

Design TECH-001 credential hardening: fix recursive mutex deadlock and replace path-derived encryption key with Windows DPAPI.

## Work completed

### Deadlock analysis
- `store()` at credentials.rs:47 locks `db.conn`, calls `derive_key()` at :50 which re-locks at :31.
- `std::sync::Mutex` is not reentrant → guaranteed deadlock.
- `test_store_and_get` confirmed deadlocked (exceeds 30s). 4 credential tests affected.

### Architecture decision (ADR-006)
- Production: Windows DPAPI (`CryptProtectData`/`CryptUnprotectData`) via `windows-sys`.
- Tests: ring-based `TestCrypto` with fixed key — deterministic, no deadlock.
- `SecretCrypto` trait → `DpapiCrypto` (prod) + `TestCrypto` (test).
- `Database.crypto: Box<dyn SecretCrypto>` injected at construction.
- `derive_key()` removed → no key to derive = no re-locking = deadlock eliminated.

### Documentation updated
- `docs/decisions/006-dpapi-credential-storage.md` — full ADR
- `.ai/ARCHITECTURE.md` — ADR-006 accepted
- `.ai/PROJECT_STATE.md` — M-CRED-HARDEN active
- `.ai/TODO.md` — TECH-001 → ready_for_handoff
- `.ai/HANDOFF.md` — full `ready` handoff: 10 acceptance criteria, 12 allowed files, 5 verifications

## Exact resume point

Codex reads TECH-001 handoff, implements on `agent/tech-001-credential-hardening`, verifies with `cargo test`, publishes via draft PR.
