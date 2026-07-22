import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

import { PostMatchAnalysis } from "../components/PostMatchAnalysis";
import { ChampionImage } from "../components/common";
import { listenWithCleanup } from "../backend/events";
import { isCommandError } from "../backend/commands";
import { useAppCore, useLeagueAssets } from "../state/AppStateProvider";
import { formatDuration, formatResult, formatTimestamp } from "../utils/formatting";
import type { TranslationKey } from "../i18n";
import {
  isMatchRecapSelection,
  matchRecapHash,
  MATCH_RECAP_SELECTED_EVENT,
  type MatchRecapSelection,
} from "../windows/matchRecapWindow";

type RecapTone = "objective" | "rage" | "flatter";

const tones: Array<{ id: RecapTone; labelKey: TranslationKey; className: string }> = [
  { id: "objective", labelKey: "recap.toneObjective", className: "border-zinc-300 dark:border-zinc-600 text-zinc-700 dark:text-zinc-300 hover:bg-zinc-50 dark:hover:bg-zinc-800 data-[active=true]:bg-zinc-950 data-[active=true]:text-white dark:data-[active=true]:bg-zinc-100 dark:data-[active=true]:text-zinc-900 data-[active=true]:border-zinc-950" },
  { id: "rage",      labelKey: "recap.toneRage", className: "border-orange-300 dark:border-orange-700 text-orange-700 dark:text-orange-400 hover:bg-orange-50 dark:hover:bg-orange-950 data-[active=true]:bg-orange-600 data-[active=true]:text-white data-[active=true]:border-orange-600" },
  { id: "flatter",   labelKey: "recap.toneFlatter", className: "border-pink-300 dark:border-pink-700 text-pink-700 dark:text-pink-400 hover:bg-pink-50 dark:hover:bg-pink-950 data-[active=true]:bg-pink-500 data-[active=true]:text-white data-[active=true]:border-pink-500" },
];

const OVERVIEW_TONE = {
  win: "border-emerald-300 bg-emerald-50 dark:border-emerald-700 dark:bg-emerald-950/60",
  loss: "border-rose-300 bg-rose-50 dark:border-rose-700 dark:bg-rose-950/60",
  unknown: "border-zinc-200 bg-white dark:border-zinc-700 dark:bg-zinc-900",
} as const;

const OVERVIEW_TITLE_TONE = {
  win: "text-emerald-800 dark:text-emerald-300",
  loss: "text-rose-700 dark:text-rose-300",
  unknown: "text-zinc-700 dark:text-zinc-300",
} as const;

export function MatchRecap({ initialSelection }: { initialSelection: MatchRecapSelection }) {
  const [selection, setSelection] = useState<MatchRecapSelection>(initialSelection);
  const { snapshot, postMatchDetails, loadPostMatchDetail, t } = useAppCore();
  const { leagueImages, loadLeagueChampionIcon, loadLeagueGameAsset } = useLeagueAssets();
  const detail = postMatchDetails[selection.gameId];

  const [detailError, setDetailError] = useState<string | null>(null);
  const [activeTone, setActiveTone] = useState<RecapTone | null>(null);
  const [streaming, setStreaming] = useState("");
  const [streamingTone, setStreamingTone] = useState<RecapTone | null>(null);
  const [cache, setCache] = useState<Record<RecapTone, string | undefined>>({
    objective: undefined,
    rage: undefined,
    flatter: undefined,
  });
  const [aiError, setAiError] = useState<string | null>(null);
  const unlistenersRef = useRef<Array<() => void>>([]);

  const aiConfigured = Boolean(
    snapshot?.settings.aiBaseUrl &&
    snapshot?.settings.aiApiKey &&
    snapshot?.settings.aiModel,
  );

  const stopListeners = useCallback(() => {
    unlistenersRef.current.forEach((fn) => fn());
    unlistenersRef.current = [];
  }, []);

  const resetForNewGame = useCallback(() => {
    stopListeners();
    setCache({ objective: undefined, rage: undefined, flatter: undefined });
    setActiveTone(null);
    setStreaming("");
    setStreamingTone(null);
    setAiError(null);
    setDetailError(null);
  }, [stopListeners]);

  // Load detail when selection changes
  useEffect(() => {
    if (!detail) {
      void loadPostMatchDetail(selection.gameId).catch((err: unknown) => {
        const msg = isCommandError(err) ? err.message : err instanceof Error ? err.message : t("recap.detailLoadFailed");
        setDetailError(msg);
      });
    }
  }, [selection.gameId, detail, loadPostMatchDetail, t]);

  // Prefetch icons/items/runes/spells when detail loads
  useEffect(() => {
    if (!detail) return;
    const championIds = new Set<number>();
    const itemIds = new Set<number>();
    const runeIds = new Set<number>();
    const spellIds = new Set<number>();

    for (const team of detail.teams) {
      for (const p of team.participants) {
        if (p.championId) championIds.add(p.championId);
        p.items.forEach((id) => itemIds.add(id));
        p.runes.forEach((id) => runeIds.add(id));
        p.spells.forEach((id) => spellIds.add(id));
      }
    }
    championIds.forEach((id) => void loadLeagueChampionIcon(id));
    itemIds.forEach((id) => void loadLeagueGameAsset("item", id));
    runeIds.forEach((id) => void loadLeagueGameAsset("rune", id));
    spellIds.forEach((id) => void loadLeagueGameAsset("spell", id));
  }, [detail, loadLeagueChampionIcon, loadLeagueGameAsset]);

  // Listen for window-reuse events
  useEffect(() => {
    return listenWithCleanup<unknown>(MATCH_RECAP_SELECTED_EVENT, (event) => {
      if (!isMatchRecapSelection(event.payload)) return;
      const next = event.payload;
      if (next.gameId === selection.gameId) return;
      resetForNewGame();
      setSelection(next);
      window.history.replaceState(null, "", matchRecapHash(next));
    });
  }, [selection.gameId, resetForNewGame]);

  useEffect(() => () => stopListeners(), [stopListeners]);

  const runAnalysis = useCallback(async (tone: RecapTone) => {
    if (streamingTone === tone) return;
    if (!detail || !detail.selfParticipantId) return;
    if (cache[tone] !== undefined) {
      setActiveTone(tone);
      setStreaming("");
      setAiError(null);
      return;
    }
    setActiveTone(tone);
    setAiError(null);
    setStreaming("");
    setStreamingTone(tone);
    stopListeners();

    // Scoped to this request so a still-running analysis from a previous
    // match/tone (its backend task isn't cancelled, only its listeners are
    // detached) can't land its late "done"/"chunk" payload on this run.
    const requestId = crypto.randomUUID();

    try {
      const [chunkUnlisten, doneUnlisten, errorUnlisten] = await Promise.all([
        listen<string>(`match-recap-${requestId}-chunk`, (event) => {
          setStreaming((prev) => prev + event.payload);
        }),
        listen<string>(`match-recap-${requestId}-done`, (event) => {
          setCache((prev) => ({ ...prev, [tone]: event.payload }));
          setStreaming("");
          setStreamingTone(null);
          stopListeners();
        }),
        listen<string>(`match-recap-${requestId}-error`, (event) => {
          setAiError(event.payload);
          setStreaming("");
          setStreamingTone(null);
          stopListeners();
        }),
      ]);

      unlistenersRef.current = [chunkUnlisten, doneUnlisten, errorUnlisten];
      await invoke("run_match_recap_analysis", { gameId: selection.gameId, tone, requestId });
    } catch (err: unknown) {
      const msg = isCommandError(err) ? err.message : err instanceof Error ? err.message : t("recap.aiStartFailed");
      setAiError(msg);
      setStreaming("");
      setStreamingTone(null);
      stopListeners();
    }
  }, [detail, streamingTone, cache, stopListeners, selection.gameId, t]);

  const isStreaming = streamingTone !== null;
  const displayText = isStreaming
    ? streaming
    : activeTone
    ? cache[activeTone] ?? ""
    : "";

  const buttonsDisabled = isStreaming || !detail || !detail.selfParticipantId || !aiConfigured;

  const activeToneEntry = tones.find((tone) => tone.id === activeTone);
  const activeToneLabel = activeTone && activeToneEntry ? t(activeToneEntry.labelKey) : "";

  const self =
    detail?.selfParticipantId != null
      ? detail.teams.flatMap((team) => team.participants).find((p) => p.participantId === detail.selfParticipantId)
      : undefined;
  const overviewResult = self ? detail!.result : "unknown";

  return (
    <main className="min-h-0 flex-1 overflow-auto bg-zinc-50 dark:bg-zinc-950 px-6 py-6">
      <div className="flex w-full flex-col gap-5">
        <header className={`flex flex-wrap items-center gap-4 rounded-xl border px-5 py-4 shadow-sm transition ${OVERVIEW_TONE[overviewResult]}`}>
          {self ? (
            <>
              <ChampionImage
                championName={self.championName}
                imageUrl={self.championId ? leagueImages.championIcons[self.championId] : undefined}
                size="lg"
              />
              <div className="min-w-0">
                <p className={`text-xs font-semibold uppercase tracking-wide ${OVERVIEW_TITLE_TONE[overviewResult]}`}>
                  {formatResult(overviewResult, t)} · {self.championName}
                </p>
                <p className="mt-0.5 truncate text-lg font-semibold text-zinc-950 dark:text-zinc-50">{self.displayName}</p>
                <p className="mt-1 flex flex-wrap items-center gap-x-2 gap-y-0.5 text-xs text-zinc-600 dark:text-zinc-400">
                  <span className="font-semibold text-zinc-800 dark:text-zinc-200">
                    {self.kills}/{self.deaths}/{self.assists}
                  </span>
                  <span>·</span>
                  <span>{t("analysis.score")} {self.performanceScore.toFixed(1)}</span>
                  {detail?.queueName && (<><span>·</span><span>{detail.queueName}</span></>)}
                  <span>·</span>
                  <span>{formatDuration(detail?.gameDurationSeconds ?? null, t)}</span>
                  <span>·</span>
                  <span>{formatTimestamp(detail?.playedAt, t)}</span>
                </p>
              </div>
            </>
          ) : (
            <h1 className="text-2xl font-semibold text-zinc-950 dark:text-zinc-50">{t("recap.title")}</h1>
          )}
          <div className="ml-auto flex flex-wrap gap-2">
            {tones.map((tone) => (
              <button
                key={tone.id}
                type="button"
                disabled={buttonsDisabled}
                data-active={activeTone === tone.id}
                onClick={() => void runAnalysis(tone.id)}
                className={`h-9 rounded-md border px-3 text-sm font-semibold transition disabled:cursor-not-allowed disabled:opacity-50 ${tone.className}`}
              >
                {t(tone.labelKey)}
              </button>
            ))}
          </div>
        </header>

        {!aiConfigured && (
          <div className="rounded-lg border border-amber-200 bg-amber-50 px-4 py-3 text-sm font-medium text-amber-800 dark:border-amber-800 dark:bg-amber-950 dark:text-amber-300">
            {t("recap.notConfiguredPrefix")}<strong>{t("recap.notConfiguredPath")}</strong>{t("recap.notConfiguredSuffix")}
          </div>
        )}

        {detail && !detail.selfParticipantId && (
          <div className="rounded-lg border border-amber-200 bg-amber-50 px-4 py-3 text-sm font-medium text-amber-800 dark:border-amber-800 dark:bg-amber-950 dark:text-amber-300">
            {t("recap.selfUnidentified")}
          </div>
        )}

        {aiError && (
          <div className="rounded-lg border border-rose-200 bg-rose-50 px-4 py-3 text-sm font-medium text-rose-800 dark:border-rose-800 dark:bg-rose-950 dark:text-rose-300">
            {aiError}
          </div>
        )}

        {(activeTone !== null || displayText || isStreaming) && (
          <section className="rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-900 shadow-sm">
            <div className="border-b border-zinc-100 dark:border-zinc-700 px-5 py-3 text-sm font-semibold text-zinc-700 dark:text-zinc-300">
              {t("recap.aiSection")}{activeToneLabel ? ` · ${activeToneLabel}` : ""}
            </div>
            <div className="px-5 py-5">
              {(displayText || isStreaming) && (
                <RecapText text={displayText} isStreaming={isStreaming} />
              )}
            </div>
          </section>
        )}

        <section>
          <h2 className="mb-3 text-sm font-semibold uppercase tracking-wide text-zinc-500 dark:text-zinc-400">
            {t("recap.detailedData")}
          </h2>
          {!detail && !detailError && (
            <div className="rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-900 px-5 py-6 text-sm text-zinc-500 dark:text-zinc-400">
              {t("recap.loadingDetail")}
            </div>
          )}
          {detailError && (
            <div className="rounded-lg border border-rose-200 bg-rose-50 px-4 py-3 text-sm font-medium text-rose-800 dark:border-rose-800 dark:bg-rose-950 dark:text-rose-300">
              {detailError}
            </div>
          )}
          {detail && (
            <PostMatchAnalysis
              detail={detail}
              gameAssets={leagueImages.gameAssets}
              onParticipantSelect={() => {}}
              participantImages={leagueImages.championIcons}
              teamsLayoutClassName="min-[1380px]:grid-cols-2"
            />
          )}
        </section>
      </div>
    </main>
  );
}

function RecapText({ text, isStreaming }: { text: string; isStreaming: boolean }) {
  const lines = text.split("\n");
  return (
    <div className="space-y-1 text-sm leading-relaxed text-zinc-800 dark:text-zinc-200">
      {lines.map((line, i) => {
        if (line.startsWith("**") && line.endsWith("**")) {
          return (
            <p key={i} className="mt-4 font-bold text-zinc-950 dark:text-zinc-50 first:mt-0">
              {line.slice(2, -2)}
            </p>
          );
        }
        if (line === "---") {
          return <hr key={i} className="my-3 border-zinc-200 dark:border-zinc-700" />;
        }
        if (line.trim() === "") return <div key={i} className="h-1" />;
        return <p key={i}>{line}</p>;
      })}
      {isStreaming && (
        <span className="inline-block h-4 w-0.5 animate-pulse bg-zinc-400" />
      )}
    </div>
  );
}
