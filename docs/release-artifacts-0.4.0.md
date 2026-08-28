# Aether 0.4.0 alpha artifacts

Built on Windows x64 on 2026-08-28 from `codex/release-040`, based on merged Milestones A–G at `da4ca98`.

| Artifact                     |      Bytes | SHA-256                                                            | Signature  |
| ---------------------------- | ---------: | ------------------------------------------------------------------ | ---------- |
| `aether.exe`                 | 16,814,080 | `22F0739253D224C2B72DC79DA60CCEBAF230094657D057896FF55962B5FA10E9` | Not signed |
| `Aether_0.4.0_x64_en-US.msi` |  7,286,784 | `C9CCFD0E6768BD64FBEFA6C55B7525CB81A3C2ECD0EB736263E4FDB284C5382F` | Not signed |
| `Aether_0.4.0_x64-setup.exe` |  4,369,273 | `54B463BDA61F36065BA1931CE64CB52D3C6B6D6FF327211162C503303602A2CC` | Not signed |

## Validation

- Frozen pnpm install passed its supply-chain policy check; typecheck, lint, 77/77 frontend tests, production build, and high-severity dependency audit passed.
- Rust formatting, strict all-target/all-feature Clippy, 95/95 Rust tests, and the optimized release build passed.
- `pnpm tauri:build` produced both 0.4.0 x64 installers with updater artifacts disabled.
- The loose executable and NSIS installer both report product/file version 0.4.0. The MSI filename and package configuration identify 0.4.0.
- Diff and security review found no signing material, updater endpoint, credential, database, backup, generated bundle, or behavior change in the publishable patch.
- GitHub Actions run `33122769813` passed frontend quality/build, Rust formatting, strict lint, and Rust tests on exact implementation head `8f97dbc`.

## Protected installation

- Aether 0.3.2 was stopped before backup or installation.
- The exact runtime data directory was resolved as `%APPDATA%/com.aether.desktop`; older documentation that claimed `%APPDATA%/Aether` was corrected.
- The complete app-data and installed application directories were copied to `C:\Users\bim\AppData\Local\AetherInstallBackups\pre-release-040-20260828-001657` before installation.
- The live and backup databases were both 360,448 bytes with SHA-256 `059A087A6E3EB9F57BE4301C5BAE87666C11B3FE06C6B4542F4B77D8D84F6AED`; SQLite integrity checks returned `ok` for both.
- Before and after the upgrade, row counts matched for Spaces (2), Notes (1), Tasks (1), Vault items (2), AI conversations (4), AI messages (24), Memory items (1), and Sources (0).
- The NSIS hash was rechecked immediately before its silent installation; the installer exited with code 0.
- Before first launch, the installed database retained its exact size and SHA-256. After launch, SQLite integrity remained `ok`.
- Windows registers Aether 0.4.0. The installed executable reports product/file version 0.4.0, has SHA-256 `5677599DA71EFB9DAC855ED7C5269223D7C9D4C9CB83BD5079166D4E43C7D9BB`, and remained responsive from `C:\Users\bim\AppData\Local\Aether\aether.exe`.

Tauri patches executable bundle metadata separately for each bundle type, so the installed executable does not have to match the loose `target/release/aether.exe` hash. Generated binaries, personal databases, credentials, and backups remain outside Git. This Alpha is not code-signed and has no active automatic updater.
