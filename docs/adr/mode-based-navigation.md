# ADR: Mode-Based Navigation (Desk vs Live)

**Status:** Active (complete — strip, contextual Rune, Activity-into-Settings, Home re-scope, grouped nav, and the StartupPage contract change all landed)
**Code:** `src/components/LiveStatusStrip.tsx`, `src/App.tsx`, `src/pages/Rune.tsx`, `src/pages/Activity.tsx`, `src/pages/Settings.tsx`, `src/pages/Dashboard.tsx`, `crates/domain`, `crates/storage` (migration 0013)
**Last updated:** 2026-06-15

---

## Background

The app has always presented a single flat sidebar of nine destinations (Dashboard, Profile, Matches, Advisor, Ranked, Rune, Chat, Activity, Settings). But the product actually operates in two distinct modes:

- **Desk mode** — the user is at their PC *browsing*: profile, match history, AI analysis, settings. They choose where to go.
- **Live mode** — League is running and the app *reacts* to the client's game-flow phase: auto-accept fires, the overlay populates during champ select, runes auto-apply on lock-in (`App.tsx`), the post-game notes window opens itself.

Treating these as one flat list produces concrete problems: `Rune` is a permanent tab that shows "waiting for lock-in" ~99% of the time; `Activity` (an internal log) and `Dashboard` (diagnostics + a worse copy of Profile) occupy prime nav real estate; and there is no representation of "you are in a game right now," even though the code already hands off on phase changes.

---

## Decision

**The primary navigation is reserved for Desk-mode destinations only. Live-mode surfaces are contextual, driven by the League client phase, and never appear as permanent (often empty) tabs.**

A persistent **Live Status Strip** in the main window is the connective tissue between the two modes. It reflects the current client state (offline → connecting → in client → in queue → match ready → accepted), synthesised from the two signals the main window already polls: the LCU connection phase (`leagueSelfSnapshot.status`) and the auto-accept state (`autoAcceptStatus`). It is read-only — it communicates state, it does not control it.

This ADR is being rolled out in slices:

- **Slice 1 (done): the Live Status Strip.**
- **Slice 2 (done): `Rune` relocated out of permanent nav.** It now appears as a contextual nav entry (marked with a pulsing "live" dot) only while a champion is locked in — set on the `champion-locked-in` event, cleared when the post-game notes window opens. Auto-navigation to it on lock-in is unchanged.
- **Slice 3 (done): `Activity` removed from primary nav.** The internal activity log / notes view is now reached from Settings (a "View activity log" entry in the data card) with a "Back to Settings" link for the round-trip. It remains a renderable page; it just no longer occupies a top-level slot.
- **Slice 4 (done): `Dashboard` re-scoped to a task-oriented Home.** It now leads with a setup checklist whose completion is **derived from real state** (client connected? AI configured?) rather than a stored "dismissed" flag — so it needs no new persisted setting and collapses to a one-line "all set" once both are done. The checklist carries a "Configure AI" CTA that deep-links to Settings and surfaces the `Shift+Tab` overlay tip (previously README-only). The old DB/schema diagnostics and the settings-echo metrics were demoted into a collapsible "System status" `<details>` rather than removed. The nav label is still "Dashboard" and `StartupPage` is unchanged.

- **Slice 5 (done): sidebar grouped into sections.** The flat list is now grouped under headers — **You** (Profile, Matches, Ranked), **Coaching** (Advisor), **Setup** (Chat, Settings), with Dashboard ungrouped at the top and the contextual Rune entry under a **Live** header when present. Headers render as labels normally and collapse to dividers in compact mode. Items were reordered so each group is contiguous. Group names are a first pass and easy to revise.
- **Slice 6 (done): `StartupPage` enum updated (the contract change).** It now offers `dashboard`/`profile`/`matches`/`advisor`/`settings` and no longer offers `activity` (now buried under Settings). This touched the Rust `StartupPage` enum (`domain`), its validation message (`application`), and required a table-rebuild migration (`0013_startup_page_options.sql`) because the original `app_settings.startup_page` CHECK constraint can't be altered in place; the migration also folds any stored `activity` value back to `dashboard`. A regression test (`migration_folds_removed_activity_startup_page_to_dashboard`) covers the upgrade path.

The mode-based navigation rollout is now complete.

---

## Consequences

- Users stop seeing empty/placeholder tabs; live state (including queue/ready-check) is always visible at a glance.
- The strip is purely additive and low-risk: it reads existing state and renders in the top bar, so it shipped without touching the navigation contract.
- The strip can only show queue/ready-check detail while auto-accept is running; otherwise it falls back to the connection phase. Acceptable — connection state is always available, and gameflow phases like champ-select/in-game already live in the overlay's own contexts.
- Future slices *will* change the navigation contract and the meaning of `StartupPage` (which today only allows `dashboard`/`activity`/`settings`). Those are the genuinely hard-to-reverse parts and are deliberately staged after this one.

---

## Alternatives considered

**Keep the flat nine-item list.** Simplest, zero work. Rejected: it forces permanently-empty tabs (`Rune`), buries the headline feature (Advisor), and gives a log (`Activity`) and diagnostics (`Dashboard`) top-level prominence.

**Show live surfaces as tabs that disable/grey out when inactive.** Keeps everything discoverable in one place. Rejected: a row of mostly-disabled tabs is visual noise and still doesn't answer "what phase am I in?" — which is exactly what the status strip makes explicit.

**Big-bang nav rewrite.** Do the whole restructure at once. Rejected in favour of slices: the strip delivers value immediately and de-risks the parts that change the navigation contract.
