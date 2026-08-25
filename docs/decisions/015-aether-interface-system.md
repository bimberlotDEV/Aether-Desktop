# ADR-015 — Aether interface system

- **Status:** Accepted
- **Date:** 2026-08-25
- **Decision owner:** Codex, following the product owner's explicit makeover direction

## Context

Aether's functional Alpha UI uses consistent neutral tokens but reads as a collection of sparse route templates rather than a distinctive personal workspace. A product-wide redesign must make the application more pleasant, alive, and futuristic while honoring the repository's prohibition on gradients, neon glow, glassmorphism, generic dashboards, fake content, and inaccessible decoration.

## Options considered

1. Restyle each route independently. Fast locally, but guarantees drift and duplicates interaction styling.
2. Adopt an external component kit. Broad coverage, but risks a shadcn/demo appearance, added dependencies, and loss of product identity.
3. Evolve the existing token system into a small Aether-native interface system, then migrate the shell and routes onto shared primitives.

## Decision

Choose option 3. Aether will use:

- a persistent navigation rail with a compact brand mark, grouped destinations, a visible command-palette affordance, and quiet local-status footer;
- one application canvas with semantic surface levels (`canvas`, `base`, `raised`, `strong`) rather than undifferentiated black or white;
- consistent page headers, action hierarchy, fields, filters, panels, empty states, badges, and icon frames implemented as shared React primitives and semantic CSS tokens;
- restrained indigo as the primary accent and cool cyan only as a secondary signal, never as a decorative gradient or glow;
- subtle borders, inset highlights, typography, alignment, and short easing-driven transitions for depth and responsiveness;
- route-specific composition that reflects the work being done instead of forcing every feature into dashboard cards;
- responsive density that preserves desktop productivity down to 1024×640 and expands content intentionally on larger windows;
- visible keyboard focus, accessible semantics, high contrast, and reduced-motion behavior as first-class visual requirements.

## Consequences

- The redesign touches most frontend routes and components but does not alter domain contracts or persistence.
- New interface work should reuse the semantic primitives before introducing route-local styling.
- Lucide remains available, but icon size, weight, framing, and meaning are curated by the interface system.
- Decorative gradients, glow, glass cards, fake activity, fake metrics, and dead actions remain prohibited even when described as “futuristic.”
- Browser-mode empty and locally persisted preview flows remain the safe visual QA surface; final desktop smoke validates the Tauri composition.

