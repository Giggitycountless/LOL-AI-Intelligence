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
A user-saved `Rune Page` for a specific champion, stored in SQLite. When present, it takes precedence over the `Rune Recommendation` on lock-in. The user can save, update, or delete it from the Rune view, which is a `Live Mode` surface shown contextually once a champion is locked in rather than a permanent navigation tab.

### Champ-Select Lock-in
The moment the local player's pick action transitions to `completed: true` in the LCU champ-select session. Triggers auto-apply of the winning `Rune Page` (Champion Rune Config if present, else top Rune Recommendation) and navigates the main window to the Rune view (a `Live Mode` surface that appears only while a champion is locked in).

### Pick/Ban Delay
A per-action (pick and ban are independent) timer, 0–5 seconds in 0.5-second increments, that starts the moment the local player's action becomes available in the LCU session. After the delay elapses the action is locked (`completed: true`). Default 0 seconds.

### Teammate Page
A dedicated sidebar tab showing raw stats (recent KDA, win rate, match history) for all 10 players during champ select. Data-only — no recommendations. Distinct from the Advisor page.

### Advisor Page
An on-demand AI analysis page. The user triggers analysis manually; the app sends the player's recent match data to an external AI API and streams back a structured report. The prior champion-guide content (static sample data) has been removed entirely.

### Player Analysis
The AI-generated output produced by the `Advisor Page`. Structure: a concise block of three named sections (strengths, weaknesses, one improvement focus) followed by a detailed free-text analysis paragraph. Language follows the app's current `effectiveLanguage`. Results are cached in SQLite and re-used until the player has played 5 or more new games since the last analysis, at which point a prompt appears suggesting a refresh.

### Analysis Scope
The filter applied before sending data to the AI. Defaults to **All** (all recent games regardless of position). The user can narrow to a single position (Top / Jungle / Mid / Bot / Support) to get role-specific advice. The 50-game window is applied after the scope filter.

### Analysis Tone
The persona the AI adopts when producing a `Player Analysis` or a `Match Recap`. Three options: **Objective** (default — neutral coach), **Rage** (毒舌喷子, harsh insults with data-grounded justification), **Flatter** (专业夸夸, surface-professional analysis that is actually pure flattery). The tone is independent of scope and applies the same way to multi-game and single-game analysis.

### Match Recap
The AI-generated single-game analysis produced by the **Match Recap window**, opened by clicking a row in the historical matches list. Always written from the local player's perspective (the "you" view), with the other nine participants' data included in the prompt as contextual reference — never as the primary subject. Each tone selection produces an independent cached result for the lifetime of the window; switching games discards the cache. Language follows the app's `effectiveLanguage`.

### Match Recap Window
A single-instance Tauri window (label reused across clicks, like `participant-profile`) that displays one historical match in full. Layout: **AI section on top** (title "赛后复盘" + three `Analysis Tone` tab buttons + streaming output), **detailed scoreboard below** (the existing `PostMatchAnalysis` component — both teams, comparison strip, builds). Clicking another match in the parent window updates this same window and aborts any in-flight AI stream. Replaces the previous inline-dropdown UX entirely.

### AI Provider Config
Three user-supplied values stored in SQLite: `base_url`, `api_key`, and `model`. Configured once in the Settings page. The app uses the OpenAI-compatible chat-completions format, so any provider that implements that interface (OpenAI, DeepSeek, Qwen, Moonshot, etc.) works without code changes.

### In-Game Overlay
A separate Tauri window (`SelfHistoryOverlay`) that appears only during the `InProgress` game phase. Displays the ten-player roster with historical stats collected during the preceding Champ-Select phase. Primary use: in-game scouting — identifying which allies need extra support and which enemies are exploitable based on their recent performance.

### Player Card
The per-player display unit inside the `In-Game Overlay`. One card per participant (5 ally cards, 5 enemy cards). Carries: champion portrait, `Summoner Level`, rank (solo/duo and flex), a composite `Scout Score`, `Champion Mastery Badge`, advisor tags, and a recent match strip showing the last six games with per-game K/D/A, KDA, and average KDA. Rank and Scout Score are the primary visual anchors; all other fields are secondary.

### Scout Score
A composite number derived from a player's recent match volume, win count, and average KDA. Used to rank players visually within the `In-Game Overlay` so the local player can quickly identify the weakest enemies and teammates needing support. Not meaningful as a standalone number — only useful for relative comparison across the ten players in the same match.

### Summoner Level
The LCU account level displayed on the `Player Card`. Sourced from the LCU summoner batch response alongside the player's identity. A secondary signal — indicates account age rather than skill.

### Champion Mastery Badge
The LCU mastery level for the champion a player has **locked in** during champ-select. Fetched once per player at `InProgress` phase start (when all ten champions are confirmed) via the LCU champion-mastery endpoint. Absent when the champion is not yet locked. Displayed on the `Player Card` as a tier badge (e.g. M7). Distinct from `Summoner Level` and from recent-match usage frequency.

### Desk Mode
The browsing half of the app — the user is at their desk deciding where to go. Desk-mode destinations (Profile, match history, AI analysis, Settings) are always reachable from the primary navigation. Contrast with `Live Mode`.

### Live Mode
The reactive half of the app — League is running and the app responds to the client's game-flow phase rather than to navigation. Live-mode surfaces (champ-select runes, the `In-Game Overlay`, post-game review) appear contextually when their phase is active rather than as permanent navigation destinations. The `Live Status Strip` connects Live Mode back to `Desk Mode`.

### Live Status Strip
A persistent indicator in the main window that reflects the current League client state: offline, connecting, in client, not logged in, in queue, match ready, or accepted. Derived from the LCU connection phase and the auto-accept state. Read-only — it communicates state, it does not control it.

### LCU Supplement
The League Client Update API provides **runtime metadata** (champion icon, current ability level, summoner spell state, game-flow phase, etc.).

On the **primary (Tencent) path**, LCU does NOT supply spell numerics (cooldown/cost/range) or ability descriptions — those come from Tencent.

On the **fallback path** (Tencent unavailable), LCU provides spell numerics as **coefficient arrays** (`cooldown_coefficients` / `cost_coefficients` / `range`). These are the last-resort values when no other source is available. Ability descriptions are suppressed (empty string) on the fallback path — raw LCU text contained unresolved `@Token@` placeholders and was worse than nothing.
