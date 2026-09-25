# Session Notes

| Field | Value |
| --- | --- |
| Schema version | 2 |
| Session date | 2026-09-25 |
| Active task | `AI-ROUTER-001` |
| Agent | Codex |
| State | complete; draft PR open |

## Current work

Aether 28 AI Router Phase 1 is implemented and verified on `agent/ai-router`.
DeepSeek and OpenAI share one closed provider-neutral backend contract. Rust owns
typed capabilities, conservative context budgets, Local only / Cloud only /
Automatic policy, native data classification/disclosure policy, and one immutable
route decision before request serialization. No backend/locality switch occurs after
dispatch.

Migration 019 adds content-free route/disclosure provenance and deterministic legacy
mapping. Dedicated commands own validated routing settings; credentials remain in
the existing DPAPI store. The UI exposes truthful routing/privacy controls, no-local-
runtime state, and locality-aware response provenance without raw hashes or JSON.

Validation passes: 216 Rust tests, 136 frontend tests across 37 files, TypeScript,
lint, production build, Rust formatting, strict Clippy, and diff checks. No dependency,
School/Calendar/Integration Sync change, tool execution, or local runtime was added.
ADR-032 extends ADR-021.

Implementation commit `11233f7` is pushed to `origin/agent/ai-router`; draft PR
[#64](https://github.com/bimberlotDEV/Aether-Desktop/pull/64) is open. Exact-head CI
is pending; all required local validation passed.

## Exact resume point

The task is complete. Await review and exact-head CI on draft PR #64. Do not begin
`LOCAL-LLM-001`, `AI-CAL-001`, tool-enabled routing, or a packaged runtime under
this contract.
