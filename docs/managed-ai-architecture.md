# Managed AI architecture boundary

## Purpose and current status

Commercial Aether may eventually offer managed AI so normal paying users do not need a provider key. This is not implemented and must not be simulated in production. BYOK DeepSeek/OpenAI remains the only working AI path until the owner approves the prerequisites below.

## Trust boundary

```text
Aether desktop
  explicit prompt + explicit attached context + short-lived account token
    -> Aether managed-AI gateway
       authentication / entitlement / quota / budget / abuse checks
       provider routing with server-held provider credentials
         -> approved AI provider
       bounded streamed response
    <- provider/model/cost-class provenance and safe error
```

The desktop never receives the managed provider key, gateway signing key, billing secret or other users' data. The gateway never receives a whole workspace, database, Source folder, Vault file, Memory set or diagnostic report by default.

## Required request contract

Every request must be bounded and versioned with:

- authenticated account and entitlement reference;
- unique idempotency/request ID;
- user-selected response mode;
- explicit provider/model route or transparent server policy version;
- prompt and only the exact context attachments shown to the user;
- per-item type, local scope/provenance label and byte/token limits;
- client/app protocol version;
- no arbitrary provider URL, filesystem path, shell instruction or mutation authority.

The response includes provider, model, policy version, request ID, finish/error status and usage class. Usage metadata must not be presented as exact billing unless the authoritative billing system confirms it.

## Privacy and consent

Before first managed-AI use, the user must see who processes the data, what explicit content leaves the PC, retention/training policy, region where relevant, deletion/account controls and the difference from BYOK. Consent cannot be buried in a general onboarding screen.

Default requirements:

- no model training on Aether customer content unless separately explicit and genuinely optional;
- shortest operational retention compatible with abuse, billing and incident obligations;
- logs redact or hash request identifiers appropriately and never store provider keys;
- prompts/responses are not general observability payloads;
- account deletion/export and legal hold behavior are documented before launch;
- subprocessor and cross-border terms are owner/legal decisions.

## Cost and abuse containment

The gateway must enforce all of the following server-side, independent of the client:

- per-request context/output caps and timeouts;
- per-account rolling quotas, concurrency and rate limits;
- owner-set daily/monthly budget ceilings plus provider-level kill switches;
- idempotency so retries cannot double-charge unexpectedly;
- abuse detection and appeal/recovery paths that avoid collecting unnecessary content;
- model allowlist and deterministic fallback rules—never a silent cross-provider retry that changes data handling;
- explicit overload, quota, payment and provider-unavailable errors;
- auditable cost reconciliation without exposing one user's usage to another.

## Reliability and incident response

Define SLOs only after the owner chooses a support promise. At minimum, implement health checks, dependency isolation, bounded retries, circuit breakers, streaming cancellation, deploy rollback, key rotation, breach response, customer notification ownership and reconciliation for uncertain billing outcomes.

Managed AI must degrade to honest unavailability. It must not corrupt local conversations, create Actions without approval, silently switch to BYOK, or block local Notes/Tasks/Search/backup.

## Required decisions before code

- account/identity authority and recovery model;
- entitlement and billing authority;
- provider(s), model(s), regions, data-processing/training/retention terms;
- quota, fair-use, pricing and hard budget ceilings derived from beta evidence and cost tests;
- content safety/abuse policy and support ownership;
- privacy notice, terms, deletion/export and incident obligations;
- infrastructure, secrets, monitoring and on-call owners.

Implementation requires a separate backend repository/security model or an explicitly approved structure; do not place a pretend server inside the desktop repository.
