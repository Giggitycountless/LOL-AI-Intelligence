# CONTEXT.md

## Glossary

### Spell Numerics
The per-rank numeric values for a champion ability: cooldown, cost, and range. Each is expressed as a slash-separated per-rank string (e.g. `"8/7/6/5/4"`) where each segment corresponds to one ability rank.

### Per-rank string
A slash-separated string where each segment is the value at a specific ability rank. Length equals the spell's rank count (typically 5 for active spells, 1 for passives). Example: `"8/7/6/5/4"` for a 5-rank cooldown. This is the canonical display format for spell numerics in the overlay.

### Champion Ability
A single champion skill (Passive, Q, W, E, or R) as shown in the overlay. Carries: slot identifier, name, description, icon, and spell numerics.

### Rune Page
A League of Legends rune configuration consisting of a primary style, secondary style, and nine selected perk IDs. Written to the LCU via `POST /lol-perks/v1/pages`. The app maintains one managed page per session; a prior page is deleted before the new one is posted.

### Rune Recommendation
A `Rune Page` sourced from the Tencent QQ Games champion-detail API (`lol.qq.com/act/lbp/common/guides/champDetail/champDetail_{champId}.js`), sorted by pick frequency. The top recommendation is auto-applied on champion lock-in.

### Champion Rune Config
A user-saved `Rune Page` for a specific champion, stored in SQLite. When present, it takes precedence over the `Rune Recommendation` on lock-in. The user can save, update, or delete it from the Rune Page sidebar tab.

### Champ-Select Lock-in
The moment the local player's pick action transitions to `completed: true` in the LCU champ-select session. Triggers auto-apply of the winning `Rune Page` (Champion Rune Config if present, else top Rune Recommendation) and navigates the main window to the Rune Page tab.

### Pick/Ban Delay
A per-action (pick and ban are independent) timer, 0–5 seconds in 0.5-second increments, that starts the moment the local player's action becomes available in the LCU session. After the delay elapses the action is locked (`completed: true`). Default 0 seconds.

### Teammate Page
A dedicated sidebar tab showing raw stats (recent KDA, win rate, match history) for all 10 players during champ select. Data-only — no recommendations. Distinct from the Advisor page.

### Advisor Page
Reserved for AI-powered analysis. Future implementation will send player data to an external AI API and display AI-generated recommendations. Currently shows static sample data marked as TODO.

### LCU Supplement
The League Client Update API provides **runtime metadata** (champion icon, current ability level, summoner spell state, game-flow phase, etc.).

On the **primary (Tencent) path**, LCU does NOT supply spell numerics (cooldown/cost/range) or ability descriptions — those come from Tencent.

On the **fallback path** (Tencent unavailable), LCU provides spell numerics as **coefficient arrays** (`cooldown_coefficients` / `cost_coefficients` / `range`). These are the last-resort values when no other source is available. Ability descriptions are suppressed (empty string) on the fallback path — raw LCU text contained unresolved `@Token@` placeholders and was worse than nothing.
