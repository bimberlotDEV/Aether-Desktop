# AGENTS.md — Aether Coding Agent Instructions

## Project identity

You are working on **Aether** — a local-first, modular personal workspace for Windows built with Tauri 2, React, TypeScript, and Tailwind CSS.

## Architecture

- **Shell:** Tauri 2 (Rust) — `src-tauri/`
- **Frontend:** React 19 + TypeScript 6 + Vite 8 — `src/`
- **Styling:** Tailwind CSS v4 with custom design tokens — `src/styles/index.css`
- **State:** Zustand stores — `src/stores/`
- **Routing:** React Router — `src/App.tsx`
- **Package manager:** pnpm

## Design rules (mandatory)

Aether must NOT look like:
- A Tailwind starter dashboard
- A shadcn/ui demo
- A purple-gradient ChatGPT clone
- A generic SaaS admin panel

The visual standard is inspired by Linear, Raycast, Arc, Things, and Craft:
- Calm, restrained, premium
- Neutral color palette with indigo accent
- Consistent spacing, typography, and border radii
- Dark/light/system theme support
- No excessive gradients, glows, or decorative elements
- No generic Lucide icons without curation

## Forbidden patterns

- Do NOT use random gradients, neon glows, or glassmorphism
- Do NOT add fake charts, lorem ipsum, or dead buttons
- Do NOT add purple-to-blue AI gradients
- Do NOT add placeholder content that looks real
- Do NOT disable focus outlines globally
- Do NOT use excessive rounded corners or pill shapes
- Do NOT scatter raw CSS throughout components — use design tokens

## Development commands

```bash
pnpm dev           # Vite dev server (browser)
pnpm tauri:dev     # Tauri desktop app
pnpm build         # TypeScript + Vite build
pnpm typecheck     # TypeScript check only
pnpm lint          # Oxlint
pnpm test          # Vitest
pnpm format        # Prettier
pnpm check         # typecheck + lint + test
```

## Commit style

Use conventional commits:
- `feat(scope): description`
- `fix(scope): description`
- `refactor(scope): description`
- `docs(scope): description`
- `test(scope): description`

## File conventions

- Always use strict TypeScript
- Use `@/` path alias for src imports
- Keep domain logic outside React components
- Use repository/service interfaces for data access (Phase 2+)
- No `any` unless documented and unavoidable

## Current phase

Phase 0-2 are complete. Phase 3 (Spaces) and Phase 4 (Notes) are substantially implemented. Phase 7 (AI integration) has a backend prototype but is not production-ready. The canonical live status is `.ai/PROJECT_STATE.md`.

## Multi-agent collaboration

Aether uses a strict Hermes/Codex workflow defined in `WORKFLOW.md`.

- **Hermes** owns analysis, planning, architecture, prioritization, decomposition, and review. Hermes does not write production code unless explicitly requested.
- **Codex** owns implementation, debugging, refactoring, testing, verification, and implementation documentation. Codex does not redesign architecture unless the active handoff explicitly authorizes it.
- `.ai/HANDOFF.md` is the only authority for the task Codex may implement.
- `.ai/PROJECT_STATE.md` is the canonical current-state snapshot.
- `.ai/TODO.md` is the prioritized backlog, not implementation authorization.

At the start of every session, read the control documents in the order defined by `WORKFLOW.md`. Do not begin product implementation unless the handoff status is `ready` and its readiness gate is complete.

## Database architecture

- SQLite via `rusqlite` (bundled) — no system dependency
- Database at `%APPDATA%/Aether/aether.db`
- All queries through Tauri commands — no frontend SQL
- Versioned migrations in `src-tauri/src/db/migrations.rs`
- Repository pattern: Rust repos → Tauri commands → TS invoke wrappers
- Add new migrations as entries in `MIGRATIONS` array

## Adding a new domain entity
1. Add migration SQL to `MIGRATIONS`
2. Create repository in `src-tauri/src/db/repositories/`
3. Add Tauri commands in `src-tauri/src/commands.rs`
4. Register commands in `src-tauri/src/lib.rs`
5. Add TypeScript types + Zod schemas in `src/lib/db/types.ts`
6. Add invoke wrappers in `src/lib/db/tauri.ts`
7. Write Rust tests (in-memory DB)

Future phases are documented in `IDEA.md` §35.
