# Aether Architecture

## Layers

```
┌─────────────────────────────────────────┐
│  React UI (components, routes, stores)  │
├─────────────────────────────────────────┤
│  Database Layer (TS types, Zod, invoke)  │
├─────────────────────────────────────────┤
│  Tauri Commands (typed wrappers)        │
├─────────────────────────────────────────┤
│  Repository Layer (settings, spaces...)  │
├─────────────────────────────────────────┤
│  SQLite via rusqlite (bundled)           │
├─────────────────────────────────────────┤
│  Tauri Bridge (IPC, native APIs)        │
├─────────────────────────────────────────┤
│  Tauri Bridge (IPC, native APIs)        │
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
- **SQLite (rusqlite):** Local-first, reliable, zero-config, versioned migrations (Phase 2 ✅)

## Security Model

- CSP restricts script sources
- Tauri capabilities use least privilege
- No unrestricted filesystem access
- API keys stored in OS credential storage (planned)
- AI context is explicit and user-controlled
