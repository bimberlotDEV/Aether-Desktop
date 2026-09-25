# ADR-032 — Provider-neutral AI routing, locality, and disclosure

- **Status:** Accepted
- **Date:** 2026-09-25
- **Task:** `AI-ROUTER-001`
- **Extends:** ADR-021

## Context

ADR-021 established a closed DeepSeek/OpenAI registry, deterministic cloud routing, fixed endpoints, response provenance, and strict separation between generated proposals and Safe Actions. Its `auto`/manual route vocabulary does not express local-first policy, execution locality, typed capabilities, context capacity, or a distinct data-egress authority.

## Decision

Rust owns a closed provider-neutral `ModelBackend` contract. Each backend exposes an immutable descriptor, health/model discovery inside the closed registry, and cancellable streaming. Provider wire formats, endpoints, credentials, and error classification remain inside adapters. DeepSeek and OpenAI are Cloud backends; Phase 1 deliberately registers no OnDevice or RemotePrivate backend.

Routing uses `LocalOnly`, `CloudOnly`, or `Automatic`. It filters an ordered candidate list by registration, enablement, mode/locality, availability, hard capabilities, context capacity, privacy, and cloud approval, then applies an eligible preference and stable registry order. Missing metadata is unsupported. There is no weighted scoring, model-directed selection, or map-order dependency. The resulting native `RouteDecision` is immutable before request serialization and cannot change after dispatch; provider failure never switches backend or locality.

Routing policy, privacy/data-egress policy, and tool authorization are independent authorities. Native classification floors cannot be lowered by frontend input. Ordinary typed prompts/history are Personal; explicitly attached Aether Note/Task/Memory/Vault context is Sensitive; credentials and raw/unbounded stores are Prohibited. Prohibited content is rejected before cloud serialization. `CloudOnly` is standing consent for prompt-only ordinary cloud use. Sensitive Aether context follows the configured disclosure policy and, when required, a short-lived opaque one-time approval bound to request hash, backend, model, and data-category inventory. This token is separate from Safe Actions approval. `ToolScope` is a typed empty placeholder and grants no tool execution.

Dedicated native commands own routing settings. Credentials remain in the existing secure store and never enter `app_settings`. Migration 019 adds nullable, content-free route/disclosure provenance. Legacy `auto` maps to `Automatic`; explicit DeepSeek/OpenAI maps to `CloudOnly`. Presentation reasons are generated only from closed reason codes.

## Consequences

- Local only truthfully fails until a reviewed local backend exists.
- Automatic is local-first and may consider cloud only when availability, policy, and approval permit it.
- DeepSeek/OpenAI preserve current request shaping, streaming, cancellation, and error behavior behind one backend interface.
- The frontend can explain routing/locality without receiving raw capability, hash, credential, or disclosure internals.
- Ollama, llama.cpp, packaged runtimes, native tools, calendar tools, tool loops, vision behavior, embeddings, and arbitrary endpoints require later reviewed tasks.

## Relationship to ADR-021

This ADR extends rather than replaces ADR-021. ADR-021 remains authoritative for the fixed cloud registry, credential isolation, no silent cloud-provider retry, proposal parsing, and Safe Actions separation. ADR-032 supersedes only its two-state `auto`/manual routing vocabulary and limited provenance model.
