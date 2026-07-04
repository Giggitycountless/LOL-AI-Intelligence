import { WebviewWindow } from "@tauri-apps/api/webviewWindow";

import { callBackend } from "../backend/commands";
import { clampToScreen } from "./windowSize";

export const SELF_HISTORY_OVERLAY_WINDOW_LABEL = "self-history-overlay";

let overlayOpenPromise: Promise<boolean> | null = null;

export function canOpenSelfHistoryOverlayWindow() {
  return callBackend<boolean>("can_open_self_history_overlay").catch(() => false);
}

export function destroySelfHistoryOverlayWindow() {
  return callBackend<void>("destroy_self_history_overlay_window").catch(() => undefined);
}

export async function openSelfHistoryOverlayWindow(options?: { devMode?: boolean }) {
  if (overlayOpenPromise) {
    return overlayOpenPromise;
  }

  overlayOpenPromise = openSelfHistoryOverlayWindowOnce(options?.devMode ?? false).finally(() => {
    overlayOpenPromise = null;
  });
  return overlayOpenPromise;
}

async function openSelfHistoryOverlayWindowOnce(devMode: boolean) {
  try {
    if (!devMode) {
      const canOpen = await canOpenSelfHistoryOverlayWindow();
      if (!canOpen) {
        return false;
      }
    }

    const existing = await WebviewWindow.getByLabel(SELF_HISTORY_OVERLAY_WINDOW_LABEL);

    if (existing) {
      await existing.show();
      return true;
    }

    // Stacked (top/bottom) team layout: tall window instead of the old
    // side-by-side wide one — five cards per row, two team rows.
    const size = clampToScreen({ width: 1280, height: 920 }, { width: 1080, height: 640 });
    const overlayWindow = new WebviewWindow(SELF_HISTORY_OVERLAY_WINDOW_LABEL, {
      alwaysOnTop: true,
      center: true,
      decorations: false,
      focus: false,
      height: size.height,
      minHeight: 640,
      minWidth: 1080,
      resizable: true,
      title: "Self History",
      url: selfHistoryOverlayWindowUrl(devMode),
      width: size.width,
    });
    void overlayWindow.once("tauri://error", () => {
      console.warn("Self history overlay window could not be opened.");
    });
    return true;
  } catch (error: unknown) {
    console.warn("openSelfHistoryOverlayWindow failed:", error);
    return false;
  }
}

export function selfHistoryOverlayWindowUrl(devMode = false) {
  return devMode ? "index.html#/self-history-overlay?devMode=1" : "index.html#/self-history-overlay";
}

export function isSelfHistoryOverlayHash(hash: string) {
  return hash === "#/self-history-overlay" || hash.startsWith("#/self-history-overlay?");
}
