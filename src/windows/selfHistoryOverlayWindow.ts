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

export async function openSelfHistoryOverlayWindow() {
  if (overlayOpenPromise) {
    return overlayOpenPromise;
  }

  overlayOpenPromise = openSelfHistoryOverlayWindowOnce().finally(() => {
    overlayOpenPromise = null;
  });
  return overlayOpenPromise;
}

async function openSelfHistoryOverlayWindowOnce() {
  try {
    const canOpen = await canOpenSelfHistoryOverlayWindow();
    if (!canOpen) {
      return false;
    }

    const existing = await WebviewWindow.getByLabel(SELF_HISTORY_OVERLAY_WINDOW_LABEL);

    if (existing) {
      await existing.show();
      return true;
    }

    const size = clampToScreen({ width: 1600, height: 800 }, { width: 1280, height: 560 });
    const overlayWindow = new WebviewWindow(SELF_HISTORY_OVERLAY_WINDOW_LABEL, {
      alwaysOnTop: true,
      center: true,
      decorations: false,
      focus: false,
      height: size.height,
      minHeight: 560,
      minWidth: 1280,
      resizable: true,
      title: "Self History",
      url: selfHistoryOverlayWindowUrl(),
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

export function selfHistoryOverlayWindowUrl() {
  return "index.html#/self-history-overlay";
}

export function isSelfHistoryOverlayHash(hash: string) {
  return hash === "#/self-history-overlay";
}
