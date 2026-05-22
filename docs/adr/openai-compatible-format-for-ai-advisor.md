# ADR: OpenAI-Compatible Chat Format for AI Advisor

**Status:** Active  
**Code:** `src/pages/Advisor.tsx`, `src/backend/aiAdvisor.ts` (new), Settings AI config section  
**Last updated:** 2026-05-23

---

## Background

The Advisor page is being redesigned to send the player's recent match data to an external AI API and stream back a structured performance analysis. The app needs to support users who have API keys from different providers (OpenAI, DeepSeek, Qwen, Moonshot, etc.), particularly given the user base is primarily Chinese players.

---

## Decision

Use the **OpenAI-compatible chat completions format** with user-supplied `base_url`, `api_key`, and `model` as the only integration point.

```
POST {base_url}/chat/completions
Authorization: Bearer {api_key}
Content-Type: application/json

{ "model": "{model}", "messages": [...], "stream": true }
```

The frontend calls this endpoint directly via `fetch` (SSE stream). The Rust backend supplies the `api_key` via a Tauri command but does not own the HTTP connection.

---

## Consequences

- One HTTP client path covers OpenAI, DeepSeek, Qwen, Moonshot, and any other provider that adopted the OpenAI format — no provider-specific branches.
- Users fill in three fields in Settings once; switching providers is a Settings change, not a code change.
- The `base_url` field allows pointing at local models (Ollama, LM Studio) for users who want fully offline analysis.
- Anthropic's native API (which uses a different message format and SDK) is not supported without the user running a compatibility proxy. Accepted: the majority of target users have OpenAI-format keys.

---

## Alternatives considered

**Anthropic SDK only.** We use Claude Code ourselves, so familiarity is high. But Anthropic's message format diverges from OpenAI (different system-prompt placement, `content` block structure, `x-api-key` header). Chinese users overwhelmingly hold DeepSeek or Qwen keys, not Anthropic keys. Would leave most users unable to use the feature.

**Both SDKs with provider selector.** Covers everyone but doubles the integration surface: two HTTP clients, two prompt formatters, two error normalizers, two sets of Settings fields. Not justified for a single feature.
