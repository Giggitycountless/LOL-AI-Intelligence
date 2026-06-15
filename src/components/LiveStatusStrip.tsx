import { useAppCore } from "../state/AppStateProvider";
import type { AutoAcceptStatus, LeagueClientStatus } from "../backend/types";
import type { TranslationKey } from "../i18n";

// Tone drives the dot colour + text colour. Ordered loosely from "nothing happening"
// to "act now" so the visual weight tracks how live the moment is.
type LiveTone = "offline" | "pending" | "online" | "active" | "ready";

export type LiveStatus = {
  labelKey: TranslationKey;
  tone: LiveTone;
  pulse: boolean;
};

// Synthesises a single "live" status from the two real signals the main window already
// polls: the LCU connection phase and the auto-accept queue state. Queue/ready-check
// signals win because they're the most time-sensitive thing on screen. (Gameflow phases
// like champ-select / in-game live in the overlay's own contexts, not here.)
export function deriveLiveStatus(
  status: LeagueClientStatus | undefined,
  autoAccept: AutoAcceptStatus | null,
  isLoading: boolean,
): LiveStatus {
  switch (autoAccept?.state) {
    case "searching":
      return { labelKey: "live.inQueue", tone: "active", pulse: true };
    case "readyCheckDetected":
    case "accepting":
      return { labelKey: "live.matchReady", tone: "ready", pulse: true };
    case "accepted":
      return { labelKey: "live.accepted", tone: "online", pulse: false };
  }

  switch (status?.phase) {
    case "connected":
    case "partialData":
      return { labelKey: "live.online", tone: "online", pulse: false };
    case "connecting":
    case "patching":
      return { labelKey: "live.connecting", tone: "pending", pulse: true };
    case "notLoggedIn":
    case "unauthorized":
      return { labelKey: "live.notLoggedIn", tone: "pending", pulse: false };
    case "notRunning":
    case "lockfileMissing":
    case "unavailable":
      return { labelKey: "live.offline", tone: "offline", pulse: false };
    default:
      return isLoading
        ? { labelKey: "live.connecting", tone: "pending", pulse: true }
        : { labelKey: "live.offline", tone: "offline", pulse: false };
  }
}

const TONE_STYLES: Record<LiveTone, { dot: string; text: string; ring: string }> = {
  offline: { dot: "bg-zinc-400", text: "text-zinc-500 dark:text-zinc-400", ring: "border-zinc-200 dark:border-zinc-700" },
  pending: { dot: "bg-amber-500", text: "text-amber-700 dark:text-amber-400", ring: "border-amber-200 dark:border-amber-800" },
  online: { dot: "bg-emerald-500", text: "text-emerald-700 dark:text-emerald-400", ring: "border-emerald-200 dark:border-emerald-800" },
  active: { dot: "bg-sky-500", text: "text-sky-700 dark:text-sky-400", ring: "border-sky-200 dark:border-sky-800" },
  ready: { dot: "bg-rose-600", text: "text-rose-700 dark:text-rose-400", ring: "border-rose-200 dark:border-rose-800" },
};

export function LiveStatusStrip() {
  const { leagueSelfSnapshot, autoAcceptStatus, isLeagueClientLoading, t } = useAppCore();
  const live = deriveLiveStatus(leagueSelfSnapshot?.status, autoAcceptStatus, isLeagueClientLoading);
  const tone = TONE_STYLES[live.tone];

  return (
    <div
      aria-label={t("live.label")}
      aria-live="polite"
      className={["inline-flex h-7 items-center gap-2 rounded-full border bg-white px-3 dark:bg-zinc-900", tone.ring].join(" ")}
      role="status"
    >
      <span className={["h-2 w-2 rounded-full", tone.dot, live.pulse ? "animate-pulse" : ""].join(" ")} />
      <span className={["text-xs font-semibold", tone.text].join(" ")}>{t(live.labelKey)}</span>
    </div>
  );
}
