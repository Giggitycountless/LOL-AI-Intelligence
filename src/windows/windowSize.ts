/** Margin kept around windows so they never butt directly against screen edges. */
const SCREEN_MARGIN = 48;

export type WindowSize = { width: number; height: number };

/**
 * Clamps a desired initial window size to the screen's available work area
 * (taskbar excluded). Fixed initial sizes otherwise overflow small laptop
 * screens — e.g. an 880-tall window on a 1366x768 display spawns with its
 * bottom under the taskbar. Never clamps below the window's minimum size;
 * `screen.avail*` and Tauri logical units are both CSS pixels, so the values
 * compare directly.
 */
export function clampToScreen(desired: WindowSize, min: WindowSize): WindowSize {
  // Guarded for non-browser test environments; `|| fallback` (not `??`)
  // because jsdom reports 0 for screen.avail*.
  const screen = typeof window === "undefined" ? undefined : window.screen;
  const availWidth = (screen?.availWidth || desired.width + SCREEN_MARGIN) - SCREEN_MARGIN;
  const availHeight = (screen?.availHeight || desired.height + SCREEN_MARGIN) - SCREEN_MARGIN;
  return {
    width: Math.max(min.width, Math.min(desired.width, availWidth)),
    height: Math.max(min.height, Math.min(desired.height, availHeight)),
  };
}
