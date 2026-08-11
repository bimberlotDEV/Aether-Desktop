# Aether 0.3.0 alpha artifacts

Rebuilt on Windows x64 on 2026-08-11 after `AI-CHAT-001` with `pnpm tauri:build`.

| Artifact                     |      Bytes | SHA-256                                                            |
| ---------------------------- | ---------: | ------------------------------------------------------------------ |
| `aether.exe`                 | 16,046,592 | `FDB3C2C19405211744B7F47D043944F65088E7A2D779D489BDA88EFAE22940C9` |
| `Aether_0.3.0_x64_en-US.msi` |  5,922,816 | `52A67B541ABBD692A53A023A8F69D422CEC0A1DE61B10685A07844D077A03C9E` |
| `Aether_0.3.0_x64-setup.exe` |  4,215,395 | `89539639C4569FDCF613B83A6076333F9CDBBD5257159323C7294E148D27F75C` |

The preceding `STAB-001` release executable passed startup and single-instance smoke tests. The `AI-CHAT-001` candidate was fully packaged while the installed application remained open, so Codex did not interrupt the user's active session for another startup smoke. Generated binaries remain ignored and are not committed to Git. These hashes identify this local candidate only; rebuilding may change them. The alpha is not code-signed and has no active auto-updater.
