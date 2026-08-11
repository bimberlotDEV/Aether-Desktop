# Aether 0.3.1 alpha artifacts

Rebuilt on Windows x64 on 2026-08-11 after `AI-CHAT-003` with `pnpm tauri:build`.

| Artifact                     |      Bytes | SHA-256                                                            |
| ---------------------------- | ---------: | ------------------------------------------------------------------ |
| `aether.exe`                 | 16,046,592 | `E496EFCF3D74048563A2966541582AD38A54BCE808B5A764EF9FCBAD89277BC3` |
| `Aether_0.3.1_x64_en-US.msi` |  5,922,816 | `FF93871A52719193A758B922F3191FAFF3773D708BBDB4248193693568B0399C` |
| `Aether_0.3.1_x64-setup.exe` |  4,215,465 | `C5F0BE6C35F609785F0F214EA93C315D917FEE5B8DEB58B4D3AAE0B65E4AFEC4` |

The NSIS bundle was installed over the preceding candidate. Windows registered version 0.3.1, the installed executable changed, and binary inspection confirmed the expected `index-DW1rFMl4.js` frontend bundle while excluding the prior `index-B62YOGGe.js` bundle. The installed app then opened the AI route with its message input visible and without the error boundary.

Tauri patches the executable separately for each bundle type, so the installed executable does not have to match the loose `target/release/aether.exe` hash. Generated binaries remain ignored and are not committed to Git. The alpha is not code-signed and has no active auto-updater.
