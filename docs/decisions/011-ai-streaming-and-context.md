# ADR-011: AI streaming, cancellation, and explicit context

- Status: Accepted
- Date: 2026-08-10
- Owner: Codex

## Decision

Aether uses the current DeepSeek OpenAI-compatible endpoint with `deepseek-v4-flash` as the default and `deepseek-v4-pro` as an explicit higher-quality option. Provider requests run in Rust; API keys never cross IPC.

Streaming responses use a typed Tauri IPC channel. The frontend supplies a unique request ID, and Rust registers an async cancellation token before opening the HTTP stream. Cancellation races the network stream itself, marks the partial assistant message `cancelled`, and always unregisters the request.

Context is opt-in per conversation. Rust resolves attached entity IDs immediately before a request and enforces Space isolation: a Space conversation can only resolve entities from that Space. Notes include title and Markdown content, Tasks include their visible metadata, and Vault items include metadata only—file bytes and stored paths are never sent.

## Consequences

- Conversation history and explicit context are the only user data sent to DeepSeek.
- The UI can show the exact attachment inventory before sending.
- Partial output is persisted with a terminal `complete`, `cancelled`, or `error` state.
- Provider errors are classified for actionable UI without exposing credentials or raw request bodies.
- Retries reuse the failed user turn and create a new assistant attempt rather than duplicating user content.
