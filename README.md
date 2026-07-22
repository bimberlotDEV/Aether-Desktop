# Aether

> Your personal operating system.

Aether is a local-first, modular and AI-assisted personal workspace for Windows. It brings together spaces, notes, tasks, files, calendars, knowledge, and AI assistance — all in one calm, polished desktop application.

**Status:** Alpha 0.1.0 — Foundation & Shell (Phase 0–1)

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
