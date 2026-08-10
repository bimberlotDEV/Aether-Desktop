# ADR-008 — Codex-only engineering workflow

- **Status:** Accepted
- **Date:** 2026-08-10
- **Decided by:** Project owner

## Context

Aether previously required Hermes to plan and review complex work while Codex implemented it. In practice, this introduced coordination overhead, duplicated repository analysis, left completed pull requests waiting for a second agent, and made project progress depend on manually transferring prompts between chats.

The project owner has explicitly chosen to continue development with Codex as the sole engineering agent.

## Options considered

| Option | Description | Verdict |
| --- | --- | --- |
| Keep Hermes/Codex split | Preserve independent planning and review. | Rejected: coordination cost outweighs the current project benefit. |
| Codex-only without formal planning | Let Codex implement every request directly. | Rejected: insufficient structure for security, migrations, and large desktop features. |
| Codex-only with risk-based task contracts and self-review | Codex owns the full cycle while complex work still receives explicit planning, acceptance criteria, verification, and a separate review pass. | **Accepted.** |

## Decision

Codex is the sole engineering agent for Aether. Tasks use one of two routes:

- `direct_codex` for small, clear, reversible, low-risk changes.
- `planned_codex` for features, architecture, security, migrations, cross-layer changes, ambiguity, or high risk.

`planned_codex` work requires a complete `.ai/HANDOFF.md` task contract before implementation and a distinct Codex self-review afterward. The file name remains for compatibility, but it is an execution contract rather than an inter-agent handoff.

The human owner remains the decision authority for destructive, irreversible, materially product-shaping, externally coordinated, or scope-expanding choices that cannot be safely inferred.

## Consequences

- Hermes is no longer required for planning, prioritization, architecture, review, acceptance, or task promotion.
- Codex may maintain the backlog, architecture registry, ADRs, task contract, implementation, tests, and project state.
- Complex work retains explicit readiness and acceptance gates.
- Self-review must be evidence-based and separate from implementation completion.
- Draft pull requests remain visible to the human owner but do not require a second AI reviewer.
- Historical Hermes records remain unchanged as project history.

## Supersedes

This decision supersedes `ADR-004` and the multi-agent process portions of `AIWF-001` and `PROC-001`. It does not change product architecture or the automatic GitHub publication decision in `ADR-007`.
