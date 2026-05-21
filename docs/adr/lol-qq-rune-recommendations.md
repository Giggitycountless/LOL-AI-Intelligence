# ADR: lol.qq.com as Primary Source for Rune Recommendations

**Status:** Active  
**Code:** `crates/adapters/src/tencent_lol_api.rs` (to be extended), `crates/adapters/src/lcu_perks.rs` (new)  
**Last updated:** 2026-05-22

---

## Background

The existing Advisor system carries a `runes` field (`AdvisorRunePage`) on each `AdvisorRecord`. This data comes from a hand-maintained remote JSON file — it is not generated from live meta analysis. The sample fallback in `sample_advisor_snapshot()` is entirely hardcoded (Garen, Xin Zhao, Ahri, Jinx, Thresh with Conqueror pages).

A new Rune Page feature is needed that:
1. Fetches real, win-rate-ranked rune recommendations per champion.
2. Auto-applies the top recommendation to the LCU on champion lock-in.
3. Lets users save per-champion preferences in SQLite.

---

## Decision

Use the Tencent QQ Games champion-detail endpoint as the rune recommendation source:

```
https://lol.qq.com/act/lbp/common/guides/champDetail/champDetail_{champId}.js
```

This endpoint returns per-lane rune configurations sorted by pick frequency, covering all champions globally. It is part of the same Tencent data ecosystem already used for spell numerics (`game.gtimg.cn`).

The `runes` field is removed from `AdvisorRecord`. The Advisor feature is retained for lane advice, matchup data, and item builds — those fields remain static/manually-maintained and are marked with `// TODO: replace with live data` in `sample_advisor_snapshot`.

---

## Consequences

- Rune recommendations are always current (Tencent updates the endpoint with each patch).
- The adapter parses a JS-wrapped JSON response (same pattern as `hero/{id}.js` already handled).
- LCU write path: `GET /lol-perks/v1/pages` → `DELETE /lol-perks/v1/pages/{id}` (first deletable non-temporary page) → `POST /lol-perks/v1/pages`.
- `AdvisorRecord.runes` field and `AdvisorRunePage` domain type are removed; callers that displayed advisor runes must be updated.
- Per-champion overrides stored in a new `champion_rune_configs` SQLite table take precedence over API recommendations.

---

## Alternatives considered

**Keep Advisor rune data.** The `AdvisorRunePage` struct already matches LCU's format. But the data is static and manually maintained — it will drift from the meta after each patch. Not viable for auto-apply.

**Use a third-party API (OP.GG, U.GG).** Higher data quality and English support, but requires API keys, rate limits, and ToS compliance. The Tencent endpoint is free, keyless, and already trusted in this codebase.
