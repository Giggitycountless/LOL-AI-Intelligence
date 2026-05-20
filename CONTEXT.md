# CONTEXT.md

## Glossary

### Spell Numerics
The per-rank numeric values for a champion ability: cooldown, cost, and range. Each is expressed as a slash-separated per-rank string (e.g. `"8/7/6/5/4"`) where each segment corresponds to one ability rank.

### Per-rank string
A slash-separated string where each segment is the value at a specific ability rank. Length equals the spell's rank count (typically 5 for active spells, 1 for passives). Example: `"8/7/6/5/4"` for a 5-rank cooldown. This is the canonical display format for spell numerics in the overlay.

### Champion Ability
A single champion skill (Passive, Q, W, E, or R) as shown in the overlay. Carries: slot identifier, name, description, icon, and spell numerics.

### LCU Supplement
The League Client Update API provides **runtime metadata** (champion icon, current ability level, summoner spell state, game-flow phase, etc.).

On the **primary (Tencent) path**, LCU does NOT supply spell numerics (cooldown/cost/range) or ability descriptions — those come from Tencent.

On the **fallback path** (Tencent unavailable), LCU provides spell numerics as **coefficient arrays** (`cooldown_coefficients` / `cost_coefficients` / `range`). These are the last-resort values when no other source is available. Ability descriptions are suppressed (empty string) on the fallback path — raw LCU text contained unresolved `@Token@` placeholders and was worse than nothing.
