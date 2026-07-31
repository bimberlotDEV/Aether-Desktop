# ADR-006 — Windows DPAPI for credential encryption

- **Status:** Accepted
- **Date:** 2026-07-31
- **Decided by:** Hermes

## Context

Aether stores AI provider API keys in a local SQLite `secrets` table. The current implementation has two critical defects:

1. **Recursive mutex deadlock (RISK-001):** `store()` acquires `db.conn.lock()`, then calls `derive_key(db)` which attempts to acquire the same `std::sync::Mutex` again. Rust's `Mutex` is not reentrant, causing a guaranteed deadlock. `test_store_and_get` reproducibly exceeds 30 seconds.

2. **Weak key derivation (RISK-002):** The AES-256 encryption key is derived by SHA-256 hashing the database file path. This means anyone with filesystem access who knows the database path can trivially derive the key and decrypt all stored secrets. There is no salt, no KDF, and no user-bound secret material.

Aether is a Windows-only desktop application (Tauri 2, MSVC toolchain). This constrains the solution space to Windows-native APIs.

## Options considered

| Option | Description | Verdict |
|---|---|---|
| A. `ReentrantMutex` + keep path-based key | Swap `Mutex` for nightly-only `ReentrantMutex`; keep SHA-256(path) key derivation. | Rejected. Fixes deadlock but not the weak key; requires nightly Rust. |
| B. Pass `&Connection` to `derive_key` + salt/KDF | Fix deadlock by passing the already-locked connection; add PBKDF2 with random salt stored in DB. | Rejected. Better, but the key is still derivable from DB contents — no user binding. |
| C. Windows DPAPI (`CryptProtectData`/`CryptUnprotectData`) | Use the OS-level Data Protection API, which encrypts data with the user's login credentials. No key to derive or store. | **Accepted.** |
| D. `keyring` crate | Cross-platform credential store abstraction. | Rejected. Adds a large dependency tree; Aether is Windows-only. |

## Decision

**Use Windows DPAPI via the `windows-sys` crate for production credential encryption.**

DPAPI `CryptProtectData` encrypts data using a key derived from the current user's login credentials. The encrypted blob can only be decrypted by the same user on the same machine. This is the standard approach used by Chrome, Edge, 1Password, and other Windows desktop applications.

For in-memory tests (`cargo test`), a parallel `TestCrypto` implementation using the existing `ring` crate with a fixed test key will provide deterministic, non-deadlocking encryption. This keeps tests fast, portable, and independent of the Windows user session.

### Architecture

```
trait SecretCrypto: Send + Sync {
    fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, String>;
    fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, String>;
}

struct DpapiCrypto;       // Production: CryptProtectData / CryptUnprotectData
struct TestCrypto;        // Test: ring AES-256-GCM with fixed test key (no deadlock)

struct Database {
    pub conn: Mutex<Connection>,
    pub crypto: Box<dyn SecretCrypto>,  // NEW FIELD
}
```

The `store`, `get`, and `remove` functions take a `&Database` reference and use `db.crypto.encrypt()` / `db.crypto.decrypt()` instead of `derive_key(db)`. Since the crypto implementation does NOT need to lock `db.conn`, the deadlock is eliminated.

## Consequences

| Aspect | Impact |
|---|---|
| Deadlock | Eliminated. `test_store_and_get`, `test_remove`, `test_overwrite` pass. |
| Secret security | Production: DPAPI user-bound. Test: ring with fixed key (no security required). |
| Portability | Production crypto is Windows-only (acceptable — Aether is Windows-only). Tests remain cross-platform. |
| API surface | Unchanged. `store()`, `get()`, `remove()` signatures stay the same. |
| Database schema | Unchanged. `secrets` table structure remains identical. |
| Dependency | New: `windows-sys` with `Win32_Security_Cryptography` feature. |
| Migration | Existing secrets encrypted with the old path-based key become unreadable. Since Aether is pre-release alpha with no production users, no migration path is needed. |

## Evidence

- `test_store_and_get` deadlock confirmed: `cargo test` on 2026-07-31 showed the test exceeding 30 seconds.
- `derive_key()` analysis: `store()` at credentials.rs:47 locks `db.conn`, calls `derive_key()` at credentials.rs:50 which locks `db.conn` again at credentials.rs:31.
- DPAPI is the documented Microsoft recommendation for per-user secret storage on Windows.
