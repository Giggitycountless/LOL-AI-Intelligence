import { WebviewWindow } from "@tauri-apps/api/webviewWindow";

export const POST_GAME_NOTES_WINDOW_LABEL = "post-game-notes";

let openPromise: Promise<boolean> | null = null;

export async function openPostGameNotesWindow() {
  if (openPromise) {
    return openPromise;
  }

  openPromise = openPostGameNotesWindowOnce().finally(() => {
    openPromise = null;
  });
  return openPromise;
}

async function openPostGameNotesWindowOnce() {
  try {
    const existing = await WebviewWindow.getByLabel(POST_GAME_NOTES_WINDOW_LABEL);
    if (existing) {
      await existing.show();
      await existing.setFocus();
      return true;
    }

    const win = new WebviewWindow(POST_GAME_NOTES_WINDOW_LABEL, {
      center: true,
      focus: true,
      height: 680,
      minHeight: 480,
      minWidth: 480,
      resizable: true,
      title: "Post-Game Notes",
      url: "index.html#/post-game-notes",
      width: 520,
    });
    void win.once("tauri://error", () => {
      console.warn("Post-game notes window could not be opened.");
    });
    return true;
  } catch (error: unknown) {
    console.warn("openPostGameNotesWindow failed:", error);
    return false;
  }
}

export function isPostGameNotesHash(hash: string) {
  return hash === "#/post-game-notes";
}
