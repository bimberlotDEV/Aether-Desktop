# Aether 0.3.0 alpha artifacts

Rebuilt on Windows x64 on 2026-08-11 after `AI-CHAT-002` with `pnpm tauri:build`.

| Artifact                     |      Bytes | SHA-256                                                            |
| ---------------------------- | ---------: | ------------------------------------------------------------------ |
| `aether.exe`                 | 16,046,592 | `06563ECD29E08A1B4D6F67F434642B30B879946C9246353E58FD535329B7355F` |
| `Aether_0.3.0_x64_en-US.msi` |  5,922,816 | `FDE89522B4E15CEAD7DC1700C6528C42E6475EF23B6F8E187D0AB5114DB50008` |
| `Aether_0.3.0_x64-setup.exe` |  4,216,511 | `55571594A002EF1760C523F5514C3DB5C0213413290B07CE311E993B49528540` |

The `AI-CHAT-002` candidate passed a packaged-release smoke test on the AI route: the message input was present and the application error boundary was absent. Generated binaries remain ignored and are not committed to Git. These hashes identify this local candidate only; rebuilding may change them. The alpha is not code-signed and has no active auto-updater.
