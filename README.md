# Aether

> Your personal operating system.

Aether is a local-first, modular and AI-assisted personal workspace for Windows. It brings together spaces, notes, tasks, files, calendars, knowledge, and AI assistance — all in one calm, polished desktop application.

**Status:** Alpha 0.3.0 — Notes & Spaces (Phases 3-4)

## Technology

- **Desktop shell:** Tauri 2 (Rust + Windows WebView2)
- **Frontend:** React 19, TypeScript, Vite, Tailwind CSS v4
- **State:** Zustand, React Router
- **Package manager:** pnpm

## Prerequisites

- Node.js ≥ 20
- pnpm ≥ 9
- Rust ≥ 1.77 (with Windows MSVC target)
- Windows 10/11 64-bit

## Development

```bash
# Install dependencies
pnpm install

# Start dev server (browser)
pnpm dev

# Start Tauri desktop app (requires Rust)
pnpm tauri:dev

# Type check
pnpm typecheck

# Lint
pnpm lint

# Test
pnpm test

# Format
pnpm format

# Full check
pnpm check
```

## Building

```bash
pnpm tauri:build
```

## Project Structure

```
aether/
├── src/               # React frontend
│   ├── components/    # Reusable UI components
│   ├── routes/        # Page components
│   ├── stores/        # Zustand state stores
│   ├── styles/        # Design tokens + CSS
│   ├── lib/           # Utilities
│   └── test/          # Test setup
├── src-tauri/         # Rust/Tauri backend
├── docs/              # Documentation
├── IDEA.md            # Product specification
├── AGENTS.md          # Agent instructions
└── README.md
```

## Design Philosophy

Aether is built to be calm, precise, premium, fast, and personal. Every detail matters. The visual standard is inspired by products like Linear, Raycast, and Arc — not generic SaaS dashboards.

## Security & Privacy

- Local-first: all data stored on your machine
- No account required
- No telemetry
- AI features are opt-in and explicit
- API keys stored securely, never committed

## License

MIT


## Database

Aether uses SQLite via `rusqlite` (bundled) for local-first storage. The database is stored at `%APPDATA%/Aether/aether.db`.

### Schema

- `app_settings` — key-value preferences
- `user_profile` — local user profile
- `spaces` — workspace containers (with hierarchy support)
- `module_instances` — modules within spaces
- `activity_events` — action history
- `notes` — Markdown notes with full-text search
- `ai_conversations` — AI chat conversations
- `ai_messages` — individual chat messages
- `ai_context_items` — explicit AI context references

See `docs/database.md` for full schema documentation.
