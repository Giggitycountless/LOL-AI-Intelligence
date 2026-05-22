# ADR: Frontend SSE for AI Response Streaming

**Status:** Active  
**Code:** `src/pages/Advisor.tsx`, `src/backend/aiAdvisor.ts` (new)  
**Last updated:** 2026-05-23

---

## Background

Every other external API call in this project follows the same pattern: Rust backend makes the HTTP request via `reqwest`, returns the result as a Tauri command response to the frontend. The AI Advisor feature needs **streaming** — the response must appear word-by-word as the model generates it, because a full analysis can take 10–30 seconds and a blocking spinner for that duration is unacceptable UX.

---

## Decision

The **frontend calls the AI API directly** via `fetch` with `{ stream: true }`, consuming the Server-Sent Events (SSE) response natively. The Rust backend exposes one read-only Tauri command (`get_ai_config`) that returns the stored `{ base_url, api_key, model }` from SQLite. The frontend reads this config once, constructs the request, and owns the streaming connection for its lifetime.

```
Frontend
  │ invoke("get_ai_config")           ← single Tauri command
  │ ← { base_url, api_key, model }
  │
  │ fetch(`${base_url}/chat/completions`, { stream: true })
  │ ← SSE chunks → append to textarea in real time
```

The completed analysis text is sent to the Rust backend via a second Tauri command (`save_ai_analysis`) for SQLite caching once the stream finishes.

---

## Consequences

- Streaming works without implementing a Tauri event-based chunking relay in Rust (which would require `async_stream`, channel management, and matching frontend listener teardown logic).
- The `api_key` is briefly present in the frontend's JavaScript heap during the request. Acceptable for a desktop app: Tauri's WebView runs in a sandboxed process with no external script execution, so XSS extraction is not a realistic threat vector.
- Tauri's HTTP capability must permit outbound requests to `{base_url}`. This requires either a permissive capability or a dynamic allow-list. We use a permissive outbound rule scoped to the Advisor window context.
- All other API calls remain in Rust. This is the only exception, justified by streaming requirements.

---

## Alternatives considered

**Rust backend streams via Tauri events.** The backend reads SSE chunks from `reqwest` and emits them as Tauri events (`app_handle.emit("ai-chunk", chunk)`). Works but requires: an async streaming `reqwest` client in a new background task, a cancellation token for early abort, and a matching `listen` / cleanup pair on the frontend. Three moving parts for the same outcome; adds ~150 lines of infrastructure for no user-visible benefit over the direct-fetch approach.

**Block until full response, then return via Tauri command.** Simplest Rust-side implementation. Unacceptable: users wait 10–30 seconds with no feedback, making the feature feel broken.
