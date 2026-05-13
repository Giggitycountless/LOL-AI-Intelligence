import type { TranslationKey } from "../i18n";
import type { MatchResult } from "../backend/types";

export type T = (key: TranslationKey) => string;

export function initials(value: string): string {
  const letters = value
    .split(/\s+/)
    .filter(Boolean)
    .slice(0, 2)
    .map((part) => part[0]?.toUpperCase() ?? "")
    .join("");

  return letters || "?";
}

export function formatTimestamp(value: string | null | undefined, t: T): string {
  if (!value) {
    return t("common.pending");
  }

  const numeric = Number(value);
  const date = Number.isFinite(numeric)
    ? new Date(numeric > 10_000_000_000 ? numeric : numeric * 1_000)
    : new Date(value);

  if (Number.isNaN(date.getTime())) {
    return value;
  }

  return date.toLocaleString();
}

export function formatDuration(value: number | null, t: T): string {
  if (value === null || value < 0) {
    return t("common.unavailable");
  }

  const minutes = Math.floor(value / 60);
  const seconds = value % 60;

  return `${minutes}:${String(seconds).padStart(2, "0")}`;
}

export function formatResult(result: MatchResult, t: T): string {
  switch (result) {
    case "win":
      return t("common.win");
    case "loss":
      return t("common.loss");
    default:
      return t("common.unknown");
  }
}

export function formatLeaguePhase(phase: string | undefined, t: T): string {
  switch (phase) {
    case "connected":
      return t("common.connected");
    case "partialData":
      return "Partial data";
    case "unauthorized":
      return "Unauthorized";
    case "notLoggedIn":
      return "Not logged in";
    case "patching":
      return "Preparing";
    case "notRunning":
      return "Not running";
    case "lockfileMissing":
      return "No lockfile";
    case "connecting":
      return "Connecting";
    case "unavailable":
      return t("common.unavailable");
    default:
      return t("common.pending");
  }
}
