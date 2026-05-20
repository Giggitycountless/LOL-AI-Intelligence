# ADR: Tencent LoL API as Primary Source for Spell Numerics

**Status:** Active  
**Code:** `crates/adapters/src/tencent_lol_api.rs`, `crates/adapters/src/champion_mapping.rs`  
**Last updated:** 2026-05-20 (updated: resolver removed, fallback description suppressed)

---

## Background

The existing system uses CDragon's `bin.json` files with a four-layer token resolution
chain to extract per-rank spell numerics (cooldown, cost, range) from LCU HTML
descriptions. This requires approximately 1,600 lines of resolver code and still fails
for ~23 known champion edge cases documented in `docs/known-unresolved-tokens.md`.

The Tencent LoL API at `https://game.gtimg.cn/images/lol/act/img/js/hero/{id}.js`
serves pre-resolved champion data as raw JSON. Spell numerics arrive as slash-separated
per-rank strings (e.g. `"8/7/6/5/4"`) — no token parsing required. All five spell slots
(Passive, Q, W, E, R) are present as entries in a unified `spells` array, distinguished
by a `spellKey` field. Champion IDs are compatible with Riot's standard numeric IDs.

Since Patch 13.22 (November 2023), all regions including CN receive patches
simultaneously. Tencent data is therefore always current for all players globally.

---

## Decision

Use the Tencent LoL API as the **primary source for spell numerics**. CDragon remains
as a complete fallback path, invoked when Tencent data is unavailable. LCU is reduced to
a supplement for runtime metadata (champion icon, current level) that static sources
cannot provide.

`map_champion_details` receives an additional `tencent_data: Option<&TencentChampionData>`
parameter. When present, Tencent values take precedence over CDragon bin data in a
single priority chain: Tencent → CDragon → LCU.

Ability descriptions come from Tencent in Simplified Chinese with `【value】` bracket
notation for numeric ranges. LCU descriptions are no longer used. An English translation
layer is planned but deferred; English users see Chinese text in the interim.

---

## Consequences

- Spell numerics are always pre-resolved on the primary path; the CDragon four-layer
  resolver is bypassed entirely.
- The Tencent adapter caches responses for 48 hours (patch-stable data). A background
  startup prefetch warms the cache before the user enters champion select. The
  champ-select path uses a 2-second timeout and falls back to CDragon on cache miss.
- CDragon description-token resolution code (~2,700 lines including tests) has been
  removed. Audit infrastructure (`audit_spell_tokens`, `unresolved_tokens`,
  `cdragonAvailable`) has also been removed. Ability descriptions on the Tencent-unavailable
  fallback path are suppressed (empty string) rather than showing raw LCU text with
  unresolved `@Token@` placeholders.
- The `alias` field in the Tencent response is validated against the expected champion
  name from LCU on each fetch; a mismatch triggers a warning and falls back to CDragon.
- The integration test for the Tencent adapter is marked `#[ignore]` (single network
  request; not a CI gate). Run with:
  `cargo test -p adapters -- --ignored tencent_fetch_garen`

---

## Alternatives considered

**Keep CDragon as primary.** Requires maintaining the four-layer resolver with known gaps
for ~23 champion tokens. Tencent eliminates this complexity on the primary path without
losing the CDragon fallback.

**LCU effectAmounts only.** LCU provides positional `effectAmounts` (used in CDragon
Layer 1), but these fail for named DataValue tokens and give incomplete coverage.
Not viable as a standalone source.
