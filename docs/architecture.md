# Aether Architecture

## Layers

```
┌─────────────────────────────────────────┐
│  React UI (components, routes, stores)  │
├─────────────────────────────────────────┤
│  Service Layer (AI, DB, File system)    │
├─────────────────────────────────────────┤
│  Tauri Bridge (IPC, native APIs)        │
├─────────────────────────────────────────┤
│  Rust Backend (commands, plugins)       │
├─────────────────────────────────────────┤
│  Windows OS (WebView2, filesystem)      │
└─────────────────────────────────────────┘
```

## Data Flow

1. **React components** render UI and dispatch actions
2. **Zustand stores** manage client state
3. **Tauri commands** (Rust) handle native operations
4. **SQLite** stores persistent data (Phase 2)

## Key Decisions

- **Tauri 2 over Electron:** Smaller bundle, better performance, native feel
- **React 19 + Vite 8:** Modern, fast HMR, TypeScript-native
- **Tailwind CSS v4:** Utility-first with CSS custom properties for theming
- **Zustand:** Simple, performant state management
- **SQLite:** Local-first, reliable, zero-config (Phase 2)

## Security Model

- CSP restricts script sources
- Tauri capabilities use least privilege
- No unrestricted filesystem access
- API keys stored in OS credential storage (planned)
- AI context is explicit and user-controlled
