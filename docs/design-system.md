# Aether Design System

## Typography

- **UI font:** Inter (sans-serif)
- **Mono font:** JetBrains Mono (for code)
- **Scale:** 11px–30px, modular scale
- **Weights:** 400 (normal), 500 (medium), 600 (semibold), 700 (bold)

## Colors

### Light theme
- Background: `#fafafa` → `#ffffff` (elevated)
- Text: `#18181b` (primary), `#52525b` (secondary), `#a1a1aa` (tertiary)
- Border: `#e4e4e7` → `#d4d4d8` (hover)
- Accent: `#4f46e5` (indigo)

### Dark theme
- Background: `#0a0a0b` → `#1e1e23` (elevated)
- Text: `#fafafa` (primary), `#a1a1aa` (secondary), `#71717a` (tertiary)
- Border: `#27272a` → `#3f3f46` (hover)
- Accent: `#6366f1` (indigo)

## Spacing

4px base grid. Scale: 0, 2, 4, 6, 8, 10, 12, 14, 16, 20, 24, 28, 32, 36, 40, 44, 48, 56, 64, 80, 96, 112, 128

## Border Radii

- `xs`: 2px (inputs, small controls)
- `sm`: 4px (buttons, tags)
- `md`: 6px (cards, panels)
- `lg`: 8px (modals)
- `xl`: 12px (large containers)

## Motion

- **Duration:** 75–150ms for micro-interactions
- **Easing:** `cubic-bezier(0.16, 1, 0.3, 1)` for entrances
- **Spring:** `cubic-bezier(0.34, 1.56, 0.64, 1)` for emphasis
- **Reduced motion:** All animations disabled via `prefers-reduced-motion`

## Shadows

- `xs`: 0 1px 2px rgb(0 0 0 / 0.05)
- `sm`: 0 1px 3px rgb(0 0 0 / 0.1), 0 1px 2px rgb(0 0 0 / 0.06)
- `md`: 0 4px 6px -1px rgb(0 0 0 / 0.1), 0 2px 4px -2px rgb(0 0 0 / 0.1)

## Icons

Lucide React, curated. Consistent stroke width (1.75). Used only where they add clarity.

## Empty States

- Centered, calm, informative
- Single icon in muted accent container
- Clear heading + concise description
- One clear action (when applicable)
