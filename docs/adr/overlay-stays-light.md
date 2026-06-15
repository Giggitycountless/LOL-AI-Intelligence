# ADR: In-Game Overlay Stays Light (Ignores App Theme)

**Status:** Active
**Code:** `src/pages/SelfHistoryOverlay.tsx`
**Last updated:** 2026-06-15

---

## Background

The main app supports a light/dark theme (the `dark` class is toggled on `<html>` in `AppShell`). The `In-Game Overlay` is a separate Tauri window that renders outside `AppShell`, so it never receives that class. Its styling is hardcoded light (`bg-zinc-100`, white cards). A reasonable instinct during UX review was to "fix" this by adding `dark:` variants and wiring the overlay to the theme.

---

## Decision

**The in-game overlay intentionally stays light and does not follow the app theme.**

The overlay floats over live League gameplay, which is a dark, visually busy scene. A light, high-contrast panel is easier to read at a glance mid-game than a dark panel that blends into the background. The overlay's job is fast scouting (rank, Scout Score, recent form) during the few seconds of champ select and loading; legibility under those conditions outweighs visual consistency with the desk-mode app.

So the overlay is deliberately exempt from the theme. No `dark:` variants are added to it, and it is not wired to receive the `dark` class.

---

## Consequences

- The overlay looks the same regardless of the user's light/dark preference. This is by design, not an oversight — that's the main reason this ADR exists, so a future reader doesn't "fix" it.
- The rest of the app (desk-mode pages, the Match Recap / participant windows) continues to honour the theme.
- If the overlay ever grows long-dwell, read-heavy content (rather than at-a-glance scouting), this decision is worth revisiting.

---

## Alternatives considered

**Wire the overlay to the app theme and add `dark:` variants.** Visually consistent with the rest of the app. Rejected: a dark panel over dark gameplay reduces at-a-glance legibility in the exact moments the overlay matters, and consistency with desk-mode UI has little value for a transient in-game HUD.

**Make it a user toggle (light/dark overlay independent of app theme).** Maximum flexibility. Rejected for now as unnecessary surface area — there's no evidence users want a dark overlay, and a light default already serves the readability goal. Can be added later if requested.
