# Aether 0.3.2 alpha artifacts

Built on Windows x64 on 2026-08-25 from the `RELEASE-032` branch after PRs #34, #35, and #36 were merged in order on `master`.

| Artifact                     |      Bytes | SHA-256                                                            | Signature  |
| ---------------------------- | ---------: | ------------------------------------------------------------------ | ---------- |
| `aether.exe`                 | 16,096,768 | `1AFF3FFA67E741B2B13A5CB59C13A171DA39250F184449F7CE4A762CACA450E7` | Not signed |
| `Aether_0.3.2_x64_en-US.msi` |  7,086,080 | `C5A02C8FEFA9A99D55BF2130E421EFFA310BF0B677A16EF56DDE6974BDB3FEFB` | Not signed |
| `Aether_0.3.2_x64-setup.exe` |  4,228,661 | `D0402AB4ADAC9E7A392957CD0A84404CF9C53ABDB691BFD9B603C37E91E7BE2B` | Not signed |

## Validation and installation

- Frozen pnpm install, typecheck, lint, 56/56 frontend tests, production build, and high-severity dependency audit passed.
- Rust formatting, strict Clippy, 63/63 Rust tests, and optimized release build passed.
- `pnpm tauri:build` produced both 0.3.2 x64 installers.
- The NSIS hash was rechecked immediately before installation and the silent installer exited with code 0.
- A complete pre-install copy was verified at `C:\Users\bim\AppData\Local\AetherInstallBackups\pre-release-032-20260825-220627`.
- The source and backup databases were both 258,048 bytes with SHA-256 `F2808F56E2D421A54745118D880B566A3809B0632018806C4C9C1DD8A6E3BD2F`.
- After upgrade and an offline verification, the installed database retained the same size and hash.
- The installed executable reports product version 0.3.2, has SHA-256 `9BA06D8CD3E367C45356FEA7A1009F2B418E480BBCC456E7CBA4435C80283390`, and the restarted process remained responsive.

Tauri patches executable bundle metadata separately for MSI and NSIS packaging, so the installed executable does not have to match the loose `target/release/aether.exe` hash. Generated binaries and backups remain ignored and are not committed. This Alpha is not code-signed and has no active automatic updater.
