# ADR-017 — Local deterministic Universal Search

- **Status:** Accepted
- **Date:** 2026-08-26
- **Decision owner:** Codex

## Context

Aether's Ctrl+K surface currently filters navigation commands and loaded Spaces only. Notes have SQLite FTS5, while Tasks, Vault, Memory, AI conversations, Activity, and Context Sources expose separate metadata queries. Milestone C requires one fast search experience without leaking local data to AI, introducing a vector database before conventional search is reliable, or letting raw SQL move into React.

## Decision

Implement one narrow Rust search repository and Tauri command that returns a typed, bounded result set across active Spaces, Notes, Tasks, Vault metadata, explicit Memory, AI conversation titles, meaningful Activity metadata, and present indexed-file metadata. Commands remain frontend-owned and are merged into the same Ctrl+K interaction.

Notes content uses the existing FTS5 index. Other currently compact domains use parameterized, escaped metadata matching with strict per-domain and global limits. Rust assigns an explainable deterministic score from text match quality, current-Space relevance, favourite/pinned state, open-Task state, and recency, then applies stable tie-breaking. Search never reads Source file content, reconstructs absolute child paths, invokes an AI provider, or mutates data.

## Consequences

- Ctrl+K becomes the primary Universal Search surface while remaining useful in browser mode for local commands.
- Search ranking can be tested with fixed signals and does not require opaque AI behavior.
- Notes content scales through FTS5; metadata domains can receive dedicated FTS migrations later only when measured scale justifies them.
- File results contain Source identity and relative paths only. Memory and AI conversation results remain local display/navigation data and are not attached to model context.
- Semantic/vector retrieval and Source file-content extraction remain separate privacy-reviewed milestones.

## Rejected alternatives

- **Frontend fan-out across domain hooks:** duplicates policy, increases round trips, and leaks domain ranking into React.
- **One large SQL UNION with opaque numeric constants:** harder to test and evolve safely across heterogeneous schemas.
- **Vector database or AI reranking now:** adds dependency, privacy, latency, and explainability costs before conventional search has evidence.
- **Indexing arbitrary Source contents:** exceeds the explicit metadata-only authorization established by ADR-016.

## Rollback

Remove the search command/repository and restore the command-only palette. No new persistent schema or user data is required for this milestone.
