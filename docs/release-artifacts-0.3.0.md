# Aether 0.3.0 alpha artifacts

Rebuilt on Windows x64 on 2026-08-11 after `STAB-001` with `pnpm tauri:build`.

| Artifact                     |      Bytes | SHA-256                                                            |
| ---------------------------- | ---------: | ------------------------------------------------------------------ |
| `aether.exe`                 | 16,046,592 | `81464F41A6629241C3C6DDEE3E52F4AF9FA28027F26768E87C553A5BBFEB186A` |
| `Aether_0.3.0_x64_en-US.msi` |  5,922,816 | `1FE3123FE1D2D30A66BB7D65BEC8E138D2D292EB5403A7614702CD8CC3DF2F13` |
| `Aether_0.3.0_x64-setup.exe` |  4,218,158 | `B3FF62A4F1C8AD6BA465A844218B48912237041885239032C19A2C935C29B83E` |

The release executable passed startup and single-instance smoke tests: a second launch retained exactly one process. Generated binaries remain ignored and are not committed to Git. These hashes identify this local candidate only; rebuilding may change them. The alpha is not code-signed and has no active auto-updater.
