import { emitTo } from "@tauri-apps/api/event";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";

import { clampToScreen } from "./windowSize";

export const MATCH_RECAP_WINDOW_LABEL = "match-recap";
export const MATCH_RECAP_SELECTED_EVENT = "match-recap-selected";

export type MatchRecapSelection = {
  gameId: number;
};

export async function openMatchRecapWindow(selection: MatchRecapSelection) {
  const existing = await WebviewWindow.getByLabel(MATCH_RECAP_WINDOW_LABEL);

  if (existing) {
    await emitTo(MATCH_RECAP_WINDOW_LABEL, MATCH_RECAP_SELECTED_EVENT, selection);
    await existing.show();
    await existing.setFocus();
    return;
  }

  const size = clampToScreen({ width: 1400, height: 880 }, { width: 880, height: 640 });
  const recapWindow = new WebviewWindow(MATCH_RECAP_WINDOW_LABEL, {
    center: true,
    focus: true,
    height: size.height,
    minHeight: 640,
    minWidth: 880,
    resizable: true,
    title: "赛后复盘",
    url: matchRecapWindowUrl(selection),
    width: size.width,
  });
  void recapWindow.once("tauri://error", () => {
    console.warn("Match recap window could not be opened.");
  });
}

export function matchRecapHash(selection: MatchRecapSelection) {
  return `#/match-recap?gameId=${encodeURIComponent(selection.gameId)}`;
}

export function matchRecapWindowUrl(selection: MatchRecapSelection) {
  return `index.html${matchRecapHash(selection)}`;
}

export function selectionFromMatchRecapHash(hash: string): MatchRecapSelection | null {
  const prefix = "#/match-recap";

  if (!hash.startsWith(prefix)) {
    return null;
  }

  const query = hash.slice(prefix.length);
  const params = new URLSearchParams(query.startsWith("?") ? query.slice(1) : query);
  const gameId = Number(params.get("gameId"));

  if (!Number.isSafeInteger(gameId) || gameId <= 0) {
    return null;
  }

  return { gameId };
}

export function isMatchRecapSelection(value: unknown): value is MatchRecapSelection {
  if (!value || typeof value !== "object") {
    return false;
  }

  const candidate = value as Partial<MatchRecapSelection>;
  return Number.isSafeInteger(candidate.gameId) && Number(candidate.gameId) > 0;
}
