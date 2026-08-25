# Codex Task Contract

| Field             | Value                            |
| ----------------- | -------------------------------- |
| Schema version    | 2                                |
| Task ID           | `UI-001`                         |
| Status            | `self_review`                    |
| Owner             | Codex                            |
| Last updated      | 2026-08-25                       |
| Related milestone | Milestone A — Product experience |
| Classification    | `planned_codex`                  |

## Objective

Transform Aether's complete frontend into a calm, premium, recognizably futuristic Windows workspace with a coherent shell, stronger information hierarchy, richer empty and populated states, and consistent interaction styling—without changing persistence, domain behavior, or security boundaries.

## Context

The current application is functional but visually flat. At 1280×720, the fixed 208px sidebar has little product identity, route headers and content widths are inconsistent, empty screens leave most of the canvas unused, controls vary between routes, and the dark theme relies on near-black surfaces without enough hierarchy. The owner explicitly requested a full layout and UI makeover. The repository requires restraint inspired by Linear, Raycast, Arc, Things, and Craft, and forbids neon, decorative gradients, glassmorphism, fake data, dead controls, and generic dashboard styling.

## Acceptance criteria

- [ ] Aether has a distinctive, cohesive application shell with clear brand, grouped navigation, command access, contextual status, and an intentional content canvas.
- [ ] Pulse, Spaces, Tasks, Vault, AI, Memory, Activity, Settings, Space Detail, Notes, dialogs, and command palette share one visual language without losing existing behavior.
- [ ] Page headers, primary/secondary buttons, fields, filters, surfaces, empty states, badges, icon treatments, and destructive actions use reusable design primitives or shared tokens.
- [ ] Dark and light themes both have legible layered surfaces, restrained indigo/cyan accents, high text contrast, and no forbidden gradients, glows, or decorative glass effects.
- [ ] The interface feels alive through meaningful status, hierarchy, micro-interactions, and spatial rhythm rather than fake content or gratuitous animation.
- [ ] Core layouts remain usable at 1024×640, 1280×720, and 1600×900 without clipped actions or inaccessible content.
- [ ] Keyboard focus remains visible; dialogs, navigation, buttons, filters, and command palette preserve accessible names and interaction semantics.
- [ ] Reduced-motion preferences disable nonessential motion.
- [ ] Existing route and interaction tests pass; new tests cover the shell and shared visual primitives where behavior changes.
- [ ] Production build size stays below the existing warning threshold and no new runtime dependency is required without evidence.
- [ ] Browser visual QA covers representative empty states, populated browser-mode flows, dark/light themes, command palette, dialogs, and supported viewport sizes.
- [ ] Self-review finds no functionality regression, dead control, fake persistence, inaccessible interaction, or styling outside the approved token system.

## Allowed paths

- `src/App.tsx`
- `src/styles/`
- `src/components/`
- `src/routes/`
- Frontend tests under `src/` that cover changed UI behavior
- `docs/decisions/015-aether-interface-system.md`
- `.ai/ARCHITECTURE.md`
- `.ai/CHANGELOG.md`
- `.ai/PROJECT_STATE.md`
- `.ai/HANDOFF.md`
- `.ai/SESSION_NOTES.md`
- `.ai/TODO.md`

## Non-goals

- Changing SQLite schemas, repositories, Tauri commands, Vault ownership, AI provider behavior, Memory scope, backup semantics, or credentials.
- Adding dashboard charts, sample data, decorative metrics, marketing surfaces, plugins, accounts, telemetry, sync, or collaboration.
- Replacing Tailwind, React, routing, Zustand, or Lucide with a new framework.
- Activating signing or the updater.
- Hiding unfinished product behavior behind fake UI.

## Dependencies and evidence

- Existing route/component behavior and automated tests.
- `AGENTS.md` visual rules and `IDEA.md` accessibility/product requirements.
- Existing Tailwind v4 token foundation in `src/styles/index.css`.
- Current browser-mode fallbacks for safe UI verification without desktop credentials or user data.
- `ADR-015`, which defines the durable interface-system direction for this redesign.

## Risks and safeguards

- **Regression across broad surface area:** preserve component props and event handlers, work through shared primitives first, and run route tests after each slice.
- **Style drift:** centralize semantic tokens and primitives; avoid per-component arbitrary values where a shared role exists.
- **Futuristic becoming noisy:** use geometry, contrast, typography, alignment, restrained accent lines, and motion—not gradients, glow, glass, or decoration.
- **Accessibility regression:** retain semantic elements, visible focus, reduced motion, readable contrast, and keyboard-verifiable dialogs/navigation.
- **Branch dependency:** UI-001 starts cleanly from current `master`. AI response synchronization and legacy-upgrade work remain in PRs #34 and #35 and will receive a separate combined integration smoke before installation.
- **Rollback:** UI-001 changes frontend code and documentation only; reverting its commits restores the prior interface without data migration.

## Required validation

```text
pnpm typecheck
pnpm lint
pnpm test
pnpm build
pnpm audit --audit-level high
git diff --check
Browser dark/light visual QA at 1024×640, 1280×720, and 1600×900
Browser keyboard/focus smoke for navigation, command palette, and dialogs
Combined local integration with PR #34 and PR #35
pnpm tauri:build
Installed Windows startup smoke without modifying user content
```

## Blocking decisions

None. The owner chose the product direction, and the concrete visual system can be inferred from the binding design constraints. New functionality or a material navigation-model change will be excluded or separately proposed rather than assumed.

## Self-review record

- **Status:** Pending implementation.
- **Acceptance mapping:** Pending.
- **Findings:** Pending.
