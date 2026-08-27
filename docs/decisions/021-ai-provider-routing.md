# ADR-021 — Transparent multi-provider routing and approved AI proposals

- **Status:** Accepted
- **Date:** 2026-08-27
- **Task:** `AI-EVOL-001`

## Context

Aether's AI UX is usable but DeepSeek-specific across configuration, persistence validation, model schemas, and request creation. The roadmap calls for provider expansion and Auto without turning Aether into an unrestricted agent. Milestone F supplies a trusted Action boundary that AI must reuse rather than bypass.

## Decision

Rust owns a closed provider registry with fixed official endpoints, model capability descriptors, provider-specific request shaping, encrypted credential lookup, and deterministic route selection. Milestone G supports DeepSeek and one OpenAI Chat Completions adapter. Arbitrary base URLs are excluded. Chat Completions preserves Aether's current message-history and SSE abstraction; adapters may migrate independently later.

`Auto` is stored as a routing preference. Before any external request, Rust selects exactly one configured provider/model using the request mode and centralized capabilities. The resolved route and a bounded presentation-safe reason are persisted on the assistant message and shown to the user. A failed request never silently sends the same context to another remote provider.

Credentials use namespaced DPAPI-protected secret keys. The legacy DeepSeek key remains a read fallback so existing installations continue working. IPC exposes configuration status only.

AI may emit a strict Task or Note Action draft. Rust treats output as untrusted, parses only the closed draft vocabulary, validates active Space scope, and returns a typed proposal. It has no execution authority. The frontend must pass the draft into the existing Safe Actions preview and the user must separately approve the opaque one-time token. File proposals, shell, delete, arbitrary network tools, and model-owned approval are excluded.

## Consequences

- Provider differences remain localized and current DeepSeek conversations continue to work.
- Users can choose explicitly or use an explainable Auto policy without hidden data-recipient changes.
- Nullable per-message provenance requires one append-only migration but no content rewrite.
- A configured provider can fail without fallback; this is preferable to undisclosed context transfer.
- OpenAI availability cannot be live-tested without an owner key, so deterministic protocol tests and honest unconfigured UI are required.
- Later providers/local endpoints need a new reviewed registry entry and capability adapter, not a user-supplied arbitrary URL.

## Evidence basis

- Existing boundaries: `src-tauri/src/ai/provider.rs`, `credentials.rs`, `context.rs`, `commands.rs`, and ADR-020.
- DeepSeek documents SSE Chat Completions, fixed models, JSON output, and untrusted tool arguments at `https://api-docs.deepseek.com/api/create-chat-completion/`.
- Official OpenAI documentation defines `POST /v1/chat/completions`, streamed completion chunks, tool calls, and the requirement to validate generated arguments at `https://developers.openai.com/api/reference/resources/chat`.
