# LCU API Reference

A self-contained catalogue of League Client Update (LCU) and Live Client Data endpoints,
the WebSocket event model, and connection/auth discovery — so we don't have to dig through
another project's source every time we need an endpoint.

**Provenance:** endpoint paths, params, and quirks distilled from
[LeagueAkari](https://github.com/LeagueAkari/LeagueAkari) (`src/shared/http-api-axios-helper/`,
GPL-3.0) plus our own `crates/adapters`. API endpoint facts themselves aren't copyrightable;
this is a reference, not copied code. Snapshot taken 2026-07-01 against a then-current client.

**How to read this:**
- `✅ ours` = we already call this endpoint (see the `crates/adapters/src/lib.rs` line refs).
- `⚠️` = a caveat worth reading before you use it.
- Paths are relative to the LCU base `https://127.0.0.1:{port}` (see [Connection](#connection--auth)).
  Live Client Data uses a different base (see [Live Client Data](#live-client-data-api)).

---

## Connection & Auth

The LCU is a local HTTPS + WebSocket server started by the client. Two ways to get its
port and auth token:

### 1. Lockfile (what we use)
`%LOCALAPPDATA%`-adjacent install dir → `lockfile`, colon-separated:
`LeagueClient:<pid>:<port>:<password>:<protocol>`.
- Auth: HTTP Basic, username `riot`, password = the lockfile password.
- Header: `Authorization: Basic base64("riot:" + password)`.
- TLS cert is self-signed → must skip verification (or pin Riot's cert, below).

### 2. Process command-line (LeagueAkari's approach, alternative/fallback)
Parse the `LeagueClientUx.exe` command line for these flags:

| Flag | Meaning |
|---|---|
| `--app-port=([0-9]+)` | LCU port |
| `--remoting-auth-token=([\w-_]+)` | auth token (the Basic password) |
| `--app-pid=([0-9]+)` | client PID |
| `--rso[_-]platform[_-]id=([\w-_]+)` | platform/region id (flag spelling varies: `_` or `-`) |
| `--region=([\w-_]+)` | region |
| `--riotclient-app-port=([0-9]+)` | Riot Client (not LCU) port |
| `--riotclient-auth-token=([\w-_]+)` | Riot Client auth token |

Riot ships a known self-signed **CA certificate** (`riotgames.com` "LoL Game Engineering
Certificate Authority"); pin it instead of skipping TLS verification if you want strictness.
The full PEM lives in LeagueAkari `src/main/shards/league-client-ux/ux-command-line-parser.ts`.

> The command-line path catches the client a hair earlier than the lockfile and exposes the
> Riot Client port too. Our lockfile approach is simpler and sufficient for everything below.

---

## WebSocket Events

The LCU exposes a WAMP-flavoured WebSocket. Subscribe by sending `[5, "<EventName>"]`;
events arrive as `[8, "<EventName>", { eventType, uri, data }]`.

- **Subscribe to everything:** `OnJsonApiEvent`
- **Subscribe to one resource:** `OnJsonApiEvent_<path-with-slashes-as-underscores>`
  e.g. `/lol-gameflow/v1/gameflow-phase` → `OnJsonApiEvent_lol-gameflow_v1_gameflow-phase`.
  (This is exactly what our `LcuSubscription::JsonApiEvent` in `crates/adapters/src/websocket.rs` builds.)
- `eventType` is `Create` | `Update` | `Delete`. `data` is the new resource state (absent on `Delete`).

High-value resources to subscribe to:

| Resource path | Why subscribe |
|---|---|
| `/lol-gameflow/v1/gameflow-phase` | drives all Live-Mode phase transitions |
| `/lol-champ-select/v1/session` | live pick/ban state, action ids, cell→summoner map |
| `/lol-matchmaking/v1/ready-check` | queue pop → auto-accept |
| `/lol-chat/v1/conversations/{id}` | incoming chat messages |
| `/lol-lobby/v2/lobby` | lobby membership / queue state |
| `/lol-end-of-game/v1/eog-stats-block` | post-game stats available |

---

## Gameflow Phases

`GET /lol-gameflow/v1/gameflow-phase` returns one bare string. Full enum:

| Phase | Meaning |
|---|---|
| `None` | idle, in client |
| `Lobby` | in a lobby |
| `Matchmaking` | searching for a match |
| `ReadyCheck` | queue popped — accept/decline window |
| `ChampSelect` | champion select |
| `GameStart` | game launching |
| `InProgress` | game running |
| `Reconnect` | client offering reconnect to a live game |
| `WaitingForStats` | match over, stats not ready |
| `PreEndOfGame` | pre-EOG sequence (honors etc.) |
| `EndOfGame` | end-of-game stats screen |
| `WatchInProgress` | spectating |
| `TerminatedInError` | error termination |

---

## Endpoints by Domain

### Gameflow — `/lol-gameflow`
| Method | Path | Purpose |
|---|---|---|
| GET | `/lol-gameflow/v1/gameflow-phase` | current phase (string) ✅ ours |
| GET | `/lol-gameflow/v1/session` | full gameflow session (map, gameData, players) ✅ ours |
| POST | `/lol-gameflow/v1/early-exit` | leave game early |
| POST | `/lol-gameflow/v1/session/dodge` | dodge champ select (body `{dodgeIds, phase:'ChampSelect'}`) |
| POST | `/lol-gameflow/v1/reconnect` | reconnect to live game |
| POST | `/lol-gameflow/v1/ack-failed-to-launch` | ack a failed launch |

### Matchmaking & Ready Check — `/lol-matchmaking`
| Method | Path | Purpose |
|---|---|---|
| POST | `/lol-matchmaking/v1/ready-check/accept` | accept queue pop ✅ ours (auto-accept) |
| POST | `/lol-matchmaking/v1/ready-check/decline` | decline queue pop |
| GET | `/lol-matchmaking/v1/search` | matchmaking search state |

### Champ Select — `/lol-champ-select`
| Method | Path | Purpose |
|---|---|---|
| GET | `/lol-champ-select/v1/session` | full champ-select session ✅ ours |
| PATCH | `/lol-champ-select/v1/session/actions/{actionId}` | pick/ban action — body `{championId, completed, type:'pick'\|'ban'}` ✅ ours |
| GET | `/lol-champ-select/v1/summoners/{cellId}` | resolve a cell to a summoner |
| GET | `/lol-champ-select/v1/all-grid-champions` | all selectable champs (grid) |
| GET | `/lol-champ-select/v1/grid-champions/{champId}` | one grid champ |
| GET | `/lol-champ-select/v1/pickable-champion-ids` | pickable ids |
| GET | `/lol-champ-select/v1/bannable-champion-ids` | bannable ids |
| GET | `/lol-champ-select/v1/disabled-champion-ids` | disabled ids |
| GET | `/lol-champ-select/v1/current-champion` | currently selected champ id |
| GET | `/lol-champ-select/v1/session/my-selection` | my selection (skin, spells) |
| PATCH | `/lol-champ-select/v1/session/my-selection` | set `selectedSkinId` / `spell1Id` / `spell2Id` |
| POST | `/lol-champ-select/v1/session/my-selection/reroll` | reroll (ARAM) |
| POST | `/lol-champ-select/v1/session/bench/swap/{champId}` | ARAM bench swap |
| GET | `/lol-champ-select/v1/skin-carousel-skins` | skins in the carousel |
| GET | `/lol-champ-select/v1/skin-selector-info` | skin selector info |
| GET | `/lol-champ-select/v1/ongoing-champion-swap` | ongoing champ-swap trade |
| POST | `/lol-champ-select/v1/session/champion-swaps/{tradeId}/{request\|accept\|decline\|cancel}` | champion-swap trade actions |
| POST | `/lol-champ-select/v1/session/swaps/{id}/{request\|accept\|decline\|cancel}` | position-swap trade actions |

> ⚠️ Lock-in detection (our `Champ-Select Lock-in`) = watch the local player's action turning
> `completed: true` in the session, not a dedicated endpoint.

### Summoner — `/lol-summoner`
| Method | Path | Purpose |
|---|---|---|
| GET | `/lol-summoner/v1/current-summoner` | local summoner ✅ ours |
| GET | `/lol-summoner/v1/summoners/{id}` | summoner by summonerId ✅ ours |
| GET | `/lol-summoner/v2/summoners/puuid/{puuid}` | summoner by puuid ✅ ours |
| GET | `/lol-summoner/v1/summoners?name={name}` | summoner by name (legacy) |
| POST | `/lol-summoner/v1/summoners/aliases` | batch Riot-ID (gameName+tagLine) → summoners ✅ ours |
| POST | `/lol-summoner/v1/save-alias` | set Riot ID alias (body `{gameName, tagLine}`) |
| GET | `/lol-summoner/v1/current-summoner/summoner-profile` | local profile (background skin/augments) |
| GET | `/lol-summoner/v1/summoner-profile?puuid={puuid}` | profile by puuid |
| POST | `/lol-summoner/v1/current-summoner/summoner-profile` | update a profile key (`{key, value}`) |
| PUT | `/lol-summoner/v1/current-summoner/icon` | set profile icon (`{profileIconId}`) |
| POST | `/lol-summoner/v1/current-summoner/name` | change name (body = raw name string) |
| GET | `/lol-summoner/v1/check-name-availability-new-summoners/{name}` | name availability |

### Ranked — `/lol-ranked`
| Method | Path | Purpose |
|---|---|---|
| GET | `/lol-ranked/v1/current-ranked-stats` | local ranked stats ✅ ours |
| GET | `/lol-ranked/v2/ranked-stats/{puuid}` | ranked stats by puuid ✅ ours (we use **v2**) |
| GET | `/lol-ranked/v1/ranked-stats/{puuid}` | ⚠️ LeagueAkari uses **v1** here — keep both as fallbacks if one 404s on a given client/region |
| POST | `/lol-ranked/v1/notifications/{id}/acknowledge` | ack ranked notification |
| POST | `/lol-ranked/v1/eos-notifications/{id}/acknowledge` | ack end-of-season notification |

### Champion Mastery — `/lol-champion-mastery`
| Method | Path | Purpose |
|---|---|---|
| GET | `/lol-champion-mastery/v1/{puuid}/champion-mastery` | all mastery entries for a puuid ✅ ours |
| POST | `/lol-champion-mastery/v1/{puuid}/champion-mastery/top?count={n}` | top-N mastery (body `{skipCache:true}`) — ⚠️ note this is a **POST**, not GET |
| POST | `/lol-champion-mastery/v1/notifications/ack` | ack mastery notifications |

> ⚠️ **Mastery 404 history (ours):** older endpoints like
> `/lol-collections/v1/inventories/{id}/champion-mastery` and
> `/lol-champion-mastery/v1/local-player/champion-mastery` are obsolete and 404 on current
> clients — a 404 here was previously mislabeled `NotLoggedIn` and silently swallowed.
> The puuid-scoped `/lol-champion-mastery/v1/{puuid}/champion-mastery` is the live path.
> See memory `champion-mastery-404-not-displaying`.

### Match History — `/lol-match-history`
| Method | Path | Purpose |
|---|---|---|
| GET | `/lol-match-history/v1/products/lol/current-summoner/matches` | local recent matches ✅ ours |
| GET | `/lol-match-history/v1/products/lol/{puuid}/matches?begIndex=&endIndex=` | paged matches by puuid (default 0–19) ✅ ours |
| GET | `/lol-match-history/v1/games/{gameId}` | full game detail ✅ ours |
| GET | `/lol-match-history/v1/game-timelines/{gameId}` | per-minute timeline |

> ⚠️ The LCU match-history window is shallow (~20 per page, limited depth). For deep history
> LeagueAkari falls back to **SGP** (an authenticated Riot service endpoint) — out of scope here.

### Perks / Runes — `/lol-perks`
| Method | Path | Purpose |
|---|---|---|
| GET | `/lol-perks/v1/pages` | all rune pages ✅ ours |
| POST | `/lol-perks/v1/pages` | create a rune page ✅ ours |
| PUT | `/lol-perks/v1/pages/{id}` | update a page ✅ ours |
| PUT | `/lol-perks/v1/currentpage` | set current page (body = raw id, `Content-Type: application/json`) |
| GET | `/lol-perks/v1/inventory` | perk inventory (page slots etc.) |
| GET | `/lol-perks/v1/recommended-champion-positions` | recommended positions |
| GET | `/lol-perks/v1/recommended-pages/champion/{championId}/position/{position}/map/{mapId}` | Riot's recommended rune pages |
| GET | `/lol-perks/v1/rune-recommender-auto-select` | is system auto-select on |
| POST | `/lol-perks/v1/rune-recommender-auto-select` | toggle system auto-select |

> Note: our rune *recommendations* come from the Tencent QQ champDetail API, not from
> `/lol-perks/.../recommended-pages`. We only use `/lol-perks` to write the page on lock-in.

### Chat — `/lol-chat`
| Method | Path | Purpose |
|---|---|---|
| GET | `/lol-chat/v1/me` | local chat identity ✅ ours |
| PUT | `/lol-chat/v1/me` | set `availability` / `statusMessage` / `lol.ranked*` ✅ ours |
| GET | `/lol-chat/v1/friends` | friends list |
| GET | `/lol-chat/v1/friend-groups` | friend groups |
| DELETE | `/lol-chat/v1/friends/{id}` | remove friend |
| POST | `/lol-chat/v2/friend-requests` | send friend request (`{gameName, tagLine, gameTag}`) |
| GET | `/lol-chat/v1/conversations` | conversations |
| GET | `/lol-chat/v1/conversations/{id}/participants` | participants in a conversation |
| POST | `/lol-chat/v1/conversations/{targetId}/messages` | **send a chat message** (`{body, type:'chat', ...}`) |

> `availability` ∈ `chat \| mobile \| dnd \| away \| offline \| online \| spectating`.
> ⚠️ The chat send endpoint types into the *client* chat — different from our in-game synthetic
> keystroke chat presets (those exist because in-game chat isn't an LCU surface).
> See memory `profile-personalization-chat-status` for the `/lol-chat/v1/me` personalization work.

### Lobby — `/lol-lobby`
| Method | Path | Purpose |
|---|---|---|
| GET | `/lol-lobby/v2/lobby` | current lobby |
| POST | `/lol-lobby/v2/lobby` | create lobby (`{queueId}` for queue, or `{customGameLobby,...}`) |
| DELETE | `/lol-lobby/v2/lobby` | leave lobby |
| GET | `/lol-lobby/v2/lobby/members` | lobby members |
| POST | `/lol-lobby/v2/lobby/members/{summonerId}/promote` | promote to owner |
| POST | `/lol-lobby/v2/lobby/members/{summonerId}/kick` | kick member |
| POST | `/lol-lobby/v2/lobby/matchmaking/search` | start queue |
| DELETE | `/lol-lobby/v2/lobby/matchmaking/search` | stop queue |
| POST | `/lol-lobby/v2/play-again` | play again (back to lobby) |
| GET | `/lol-lobby/v2/party/eog-status` | end-of-game party status |
| GET | `/lol-lobby/v2/received-invitations` | received invites |
| POST | `/lol-lobby/v2/received-invitations/{id}/{accept\|decline}` | respond to invite |
| POST | `/lol-lobby/v2/lobby/invitations` | invite (`[{toSummonerId}]`) |
| POST | `/lol-lobby/v2/eligibility/{party\|self}` | queue eligibility |

### Game Data (static assets) — `/lol-game-data/assets/v1`
All GET, all return bundled JSON the client ships (no network):

| Path | Contents |
|---|---|
| `champion-summary.json` | all champions (id, name, alias, roles) |
| `champions/{champId}.json` | full champ detail (spells, skins) |
| `summoner-spells.json` | summoner spells |
| `items.json` | items |
| `perks.json` / `perkstyles.json` | runes / rune trees |
| `queues.json` | queue definitions |
| `maps.json` / `map-assets/map-assets.json` | maps |
| `game-mode-mutators.json` | mode mutators |
| `cherry-augments.json` | Arena augments |
| `challenges.json` | challenges |
| `loots.json` | loot definitions |

> These are the canonical id→name/icon maps. Cheaper and offline vs. Data Dragon /
> Community Dragon, and always version-matched to the running client.

### End / Post Game
| Method | Path | Purpose |
|---|---|---|
| POST | `/lol-end-of-game/v1/state/dismiss-stats` | dismiss the EOG stats screen |
| GET | `/lol-pre-end-of-game/v1/currentSequenceEvent` | current pre-EOG sequence event |
| POST | `/lol-pre-end-of-game/v1/complete/{sequenceEventName}` | complete a pre-EOG step |
| GET | `/lol-player-report-sender/v1/reported-player/gameId/{gameId}` | who got reported this game |

### Honor — `/lol-honor-v2` (and legacy `/lol-honor`)
| Method | Path | Purpose |
|---|---|---|
| GET | `/lol-honor-v2/v1/ballot/` | honor ballot (who you can honor) |
| POST | `/lol-honor-v2/v1/honor-player/` | honor a player (`{gameId, honorCategory, puuid}`) |
| POST | `/lol-honor-v2/v1/{ack-honor-notification/{mailId}\|late-recognition/ack\|level-change/ack\|mutual-honor/ack\|reward-granted/ack}` | various acks |

> `honorCategory` ∈ `COOL \| SHOTCALLER \| HEART \| '' \| OPT_OUT`.

### Loadouts / Loot / Missions / Rewards (lower priority)
| Domain | Notable paths |
|---|---|
| Loadouts | `GET /lol-loadouts/v4/loadouts/scope/account`, `PATCH /lol-loadouts/v4/loadouts/{contentId}` (emotes, etc.) |
| Loot | `GET /lol-loot/v1/player-loot-map`, `POST /lol-loot/v1/recipes/{recipe}/craft?repeat=`, `POST /lol-loot/v1/craft/mass` |
| Missions | `GET /lol-missions/v1/{missions\|series\|data}` |
| Rewards | `GET /lol-rewards/v1/{grants\|groups}` (⚠️ `groups` is huge — filter by `types`) |
| Replays | `GET /lol-replays/v1/metadata/{gameId}`, `POST /lol-replays/v1/rofls/{gameId}/{watch\|download}` |
| Store | `GET /lol-store/v1/giftablefriends` |
| Regalia | `GET\|PUT /lol-regalia/v2/current-summoner/regalia` |

### Client / Process control
| Method | Path | Purpose |
|---|---|---|
| POST | `/process-control/v1/process/quit` | quit the client |
| POST | `/riotclient/kill-ux` | kill client UX (background it) |
| POST | `/riotclient/launch-ux` | relaunch UX |
| POST | `/riotclient/kill-and-restart-ux` | restart UX |
| GET | `/lol-login/v1/login-queue-state` | login queue state |
| GET | `/entitlements/v1/token` | entitlements token (for external Riot services) |
| GET | `/lol-league-session/v1/league-session-token` | league session token |

---

## Live Client Data API

A **separate** local server, only up while a game is `InProgress`:
base `https://127.0.0.1:2999`, **no auth**, self-signed TLS.
[Riot sample](https://static.developer.riotgames.com/docs/lol/liveclientdata_sample.json).

| Method | Path | Purpose |
|---|---|---|
| GET | `/liveclientdata/allgamedata` | everything in one call |
| GET | `/liveclientdata/playerlist` | all 10 players (champ, scores, items, runes) ✅ ours |
| GET | `/liveclientdata/activeplayer` | the local player (full) ✅ ours |
| GET | `/liveclientdata/activeplayername` | local player name |
| GET | `/liveclientdata/activeplayerabilities` | local ability levels |
| GET | `/liveclientdata/activeplayerrunes` | local runes |
| GET | `/liveclientdata/playerscores` | KDA/CS per player ✅ ours |
| GET | `/liveclientdata/playeritems` | items per player ✅ ours |
| GET | `/liveclientdata/playersummonerspells` | summoner spells per player ✅ ours |
| GET | `/liveclientdata/playermainrunes` | main runes per player |
| GET | `/liveclientdata/eventdata` | game events (kills, objectives) ✅ ours |
| GET | `/liveclientdata/gamestats` | game time, map, mode ✅ ours |

---

## Region / Server Caveats

- **Tencent (CN) servers** behave differently: many community LCU tools (LeagueAkari included)
  exclude Tencent, and CN anti-cheat (Vanguard) may flag synthetic input. Our overlay/data
  reads are passive LCU GETs (safe); only the synthetic-keystroke chat presets carry risk.
- On our **primary (Tencent) path**, spell numerics/descriptions come from the Tencent
  champDetail API, **not** the LCU. LCU only supplies runtime metadata. See `CONTEXT.md` →
  *LCU Supplement* and the Tencent ADRs.
- Endpoint versions drift by client/region. When something 404s, check whether a `v1`/`v2`
  sibling exists (see Ranked above) before assuming a login/permission problem.
