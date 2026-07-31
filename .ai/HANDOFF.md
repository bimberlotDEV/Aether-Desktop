# Agent Handoff

> Canonical contract for the single task currently moving from planning to implementation or from implementation to review.

| Field | Value |
| --- | --- |
| Schema version | 1 |
| Task ID | `TECH-001` |
| Status | `ready_for_review` |
| Owner | Codex |
| Prepared by | Hermes |
| Last updated | 2026-07-31 |
| Related milestone | `M-CRED-HARDEN` |

## Responsibility of this file

- Hold exactly one active, implementation-ready assignment.
- Define scope, editable files, acceptance criteria, verification, and risks.
- Record Codex's implementation result and Hermes's review decision.
- Never act as a general backlog, architecture diary, or changelog.

## Classification

```
Classification: hermes_codex
Reason: Security-sensitive credential storage redesign with new native dependency, database struct change, and cross-cutting impact on tests.
```

## Current task

### Objective

Replace the path-derived AES key with Windows DPAPI encryption and fix the recursive mutex deadlock in credential operations. All 4 credential tests must pass; 0 production code deadlocks.

### Context

Two critical defects in `src-tauri/src/ai/credentials.rs`:

**RISK-001 — Recursive mutex deadlock:** `store()` locks `db.conn` (line 47), then calls `derive_key(db)` (line 50) which locks `db.conn` again (line 31). `std::sync::Mutex` is not reentrant — guaranteed deadlock. Also affects `get()` (lines 82 + 110). `test_store_and_get` reproducibly times out.

**RISK-002 — Weak key derivation:** `derive_key()` hashes the database path with SHA-256 to produce the AES-256 key. Anyone with filesystem access who knows the DB path can trivially decrypt all secrets. No salt, no KDF, no user binding.

**ADR-006** (accepted, see `docs/decisions/006-dpapi-credential-storage.md`) mandates:
- Production: Windows DPAPI (`CryptProtectData`/`CryptUnprotectData`) via `windows-sys`
- Tests: ring-based `TestCrypto` with fixed key (no deadlock, deterministic)

### Implementation plan

1. Add `windows-sys` to `Cargo.toml` with `Win32_Security_Cryptography` feature.
2. Define `SecretCrypto` trait in `credentials.rs`.
3. Implement `DpapiCrypto` using `CryptProtectData`/`CryptUnprotectData`.
4. Implement `TestCrypto` using `ring` AES-256-GCM with a fixed test key.
5. Add `crypto: Box<dyn SecretCrypto>` field to `Database` struct in `src-tauri/src/db/mod.rs`.
6. Update `Database::open()` to accept and inject the crypto implementation.
7. Update `Database` construction in `src-tauri/src/lib.rs` (and `main.rs` if needed) to pass `DpapiCrypto`.
8. Rewrite `store()`, `get()`, `remove()` to use `db.crypto.encrypt()`/`decrypt()` instead of `derive_key(db)`.
9. Remove `derive_key()` and the `ring` dependency from production path (ring stays for `TestCrypto`).
10. Run `cargo test`, `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`.

### Allowed files

- `src-tauri/Cargo.toml` (new dependency)
- `src-tauri/src/ai/credentials.rs` (complete rewrite of crypto layer)
- `src-tauri/src/ai/mod.rs` (if new module structure needed)
- `src-tauri/src/db/mod.rs` (new `crypto` field on `Database`)
- `src-tauri/src/lib.rs` (inject `DpapiCrypto` into `Database`)
- `src-tauri/src/main.rs` (if separate `Database` construction exists)
- `docs/decisions/006-dpapi-credential-storage.md` (already created, may need minor updates)
- `.ai/HANDOFF.md` (implementation result)
- `.ai/PROJECT_STATE.md` (resolve RISK-001, RISK-002)
- `.ai/TODO.md` (mark TECH-001 done)
- `.ai/CHANGELOG.md` (append completion entry)
- `.ai/SESSION_NOTES.md` (session record)

### Out of scope

- UI changes or frontend code.
- Database migrations or schema changes (`secrets` table stays identical).
- Provider/API changes (DeepSeek remains the only provider).
- Adding support for non-Windows platforms.
- Fixing Clippy warnings outside the credential module (tracked as DEBT-005, DEBT-006).
- Migration of existing secrets (pre-release alpha, no production data).

### Acceptance criteria

- [x] `test_store_and_get` passes (no deadlock).
- [x] `test_remove` passes.
- [x] `test_overwrite` passes.
- [x] `test_missing_key` passes.
- [x] `store()` and `get()` use DPAPI in production builds, not path-derived key.
- [x] `SecretCrypto` trait exists with `DpapiCrypto` and `TestCrypto` implementations.
- [x] `Database` struct has a `crypto` field injected at construction time.
- [x] `derive_key()` function is removed.
- [x] No unsafe code beyond the `windows-sys` FFI bindings.
- [x] `cargo clippy` produces 0 new warnings in the credential module.

### Required verification

- `cargo test --manifest-path src-tauri/Cargo.toml` — all 33 tests pass.
- `cargo build --manifest-path src-tauri/Cargo.toml` — compiles with DPAPI.
- `pnpm check` — frontend still passes.

### Risks and constraints

| ID | Risk | Mitigation |
| --- | --- | --- |
| `RISK-TECH-001` | DPAPI FFI is unsafe and could panic on malformed data. | Wrap FFI in safe functions; return `Result::Err` on failure, never panic. |
| `RISK-TECH-002` | `TestCrypto` uses a fixed key — don't accidentally use it in production. | `TestCrypto` only constructed in `#[cfg(test)]` code paths. |
| `RISK-TECH-003` | `Database` struct change breaks callers in `lib.rs`/`main.rs`. | Explicitly listed as allowed files; inspect all `Database::open()` / `Database { ... }` call sites. |
| `RISK-TECH-004` | DPAPI blobs are larger than plaintext — base64 size may increase. | DPAPI adds ~200 bytes overhead; API key strings are small enough that this is irrelevant. |

## Implementation result

### Summary

Implemented ADR-006 with a `SecretCrypto` boundary, Windows DPAPI production encryption, ring-based test encryption, and constructor injection into `Database`. Credential encryption and decryption no longer acquire the SQLite mutex, eliminating the recursive lock.

### Files changed

- `src-tauri/Cargo.toml`
- `src-tauri/Cargo.lock`
- `src-tauri/src/ai/credentials.rs`
- `src-tauri/src/db/mod.rs`
- `src-tauri/src/lib.rs`
- `.ai/HANDOFF.md`
- `.ai/PROJECT_STATE.md`
- `.ai/TODO.md`
- `.ai/CHANGELOG.md`
- `.ai/SESSION_NOTES.md`

### Verification result

- `cargo test --manifest-path src-tauri/Cargo.toml` — pass, 33/33.
- `cargo build --manifest-path src-tauri/Cargo.toml` — pass with production `DpapiCrypto`.
- Frontend typecheck and lint — pass; Vitest — pass, 7/7 (two files run sequentially because Windows workers exceeded the pool startup timeout).
- Changed-file `rustfmt --check` — pass.
- Strict Clippy — no findings in changed modules; repository command remains red on pre-existing `DEBT-006` findings outside scope.
- Repository-wide `cargo fmt --check` — remains red on pre-existing `DEBT-005` files outside scope.

### Deviations

`src-tauri/Cargo.lock` was not listed explicitly but is included as mandatory generated output of the approved `Cargo.toml` dependency change. No unrelated source files changed.

## Hermes review

| Field | Value |
| --- | --- |
| Decision | `pending` |
| Reviewer | Hermes |
| Reviewed at | `Not reviewed` |
| Findings | None |
| Follow-up task IDs | None |
