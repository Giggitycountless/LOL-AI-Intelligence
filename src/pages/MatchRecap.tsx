import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

import { PostMatchAnalysis } from "../components/PostMatchAnalysis";
import { listenWithCleanup } from "../backend/events";
import { isCommandError } from "../backend/commands";
import { useAppCore, useLeagueAssets } from "../state/AppStateProvider";
import {
  isMatchRecapSelection,
  matchRecapHash,
  MATCH_RECAP_SELECTED_EVENT,
  type MatchRecapSelection,
} from "../windows/matchRecapWindow";

type RecapTone = "objective" | "rage" | "flatter";

const tones: Array<{ id: RecapTone; label: string; className: string }> = [
  { id: "objective", label: "客观分析", className: "border-zinc-300 dark:border-zinc-600 text-zinc-700 dark:text-zinc-300 hover:bg-zinc-50 dark:hover:bg-zinc-800 data-[active=true]:bg-zinc-950 data-[active=true]:text-white dark:data-[active=true]:bg-zinc-100 dark:data-[active=true]:text-zinc-900 data-[active=true]:border-zinc-950" },
  { id: "rage",      label: "🤬 极端怒喷", className: "border-orange-300 dark:border-orange-700 text-orange-700 dark:text-orange-400 hover:bg-orange-50 dark:hover:bg-orange-950 data-[active=true]:bg-orange-600 data-[active=true]:text-white data-[active=true]:border-orange-600" },
  { id: "flatter",   label: "🌸 专业夸夸", className: "border-pink-300 dark:border-pink-700 text-pink-700 dark:text-pink-400 hover:bg-pink-50 dark:hover:bg-pink-950 data-[active=true]:bg-pink-500 data-[active=true]:text-white data-[active=true]:border-pink-500" },
];

export function MatchRecap({ initialSelection }: { initialSelection: MatchRecapSelection }) {
  const [selection, setSelection] = useState<MatchRecapSelection>(initialSelection);
  const { snapshot, postMatchDetails, loadPostMatchDetail } = useAppCore();
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
        const msg = isCommandError(err) ? err.message : err instanceof Error ? err.message : "无法加载对局数据";
        setDetailError(msg);
      });
    }
  }, [selection.gameId, detail, loadPostMatchDetail]);

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

    try {
      const [chunkUnlisten, doneUnlisten, errorUnlisten] = await Promise.all([
        listen<string>("match-recap-chunk", (event) => {
          setStreaming((prev) => prev + event.payload);
        }),
        listen<string>("match-recap-done", (event) => {
          setCache((prev) => ({ ...prev, [tone]: event.payload }));
          setStreaming("");
          setStreamingTone(null);
          stopListeners();
        }),
        listen<string>("match-recap-error", (event) => {
          setAiError(event.payload);
          setStreaming("");
          setStreamingTone(null);
          stopListeners();
        }),
      ]);

      unlistenersRef.current = [chunkUnlisten, doneUnlisten, errorUnlisten];
      await invoke("run_match_recap_analysis", { gameId: selection.gameId, tone });
    } catch (err: unknown) {
      const msg = isCommandError(err) ? err.message : err instanceof Error ? err.message : "AI 分析启动失败";
      setAiError(msg);
      setStreaming("");
      setStreamingTone(null);
      stopListeners();
    }
  }, [detail, streamingTone, cache, stopListeners, selection.gameId]);

  const isStreaming = streamingTone !== null;
  const displayText = isStreaming
    ? streaming
    : activeTone
    ? cache[activeTone] ?? ""
    : "";

  const buttonsDisabled = isStreaming || !detail || !detail.selfParticipantId || !aiConfigured;

  return (
    <main className="min-h-0 flex-1 overflow-auto bg-zinc-50 dark:bg-zinc-950 px-6 py-6">
      <div className="flex w-full flex-col gap-5">
        <header className="flex flex-wrap items-center gap-3">
          <h1 className="text-2xl font-semibold text-zinc-950 dark:text-zinc-50">赛后复盘</h1>
          <div className="ml-auto flex flex-wrap gap-2">
            {tones.map((t) => (
              <button
                key={t.id}
                type="button"
                disabled={buttonsDisabled}
                data-active={activeTone === t.id}
                onClick={() => void runAnalysis(t.id)}
                className={`h-9 rounded-md border px-3 text-sm font-semibold transition disabled:cursor-not-allowed disabled:opacity-50 ${t.className}`}
              >
                {t.label}
              </button>
            ))}
          </div>
        </header>

        {!aiConfigured && (
          <div className="rounded-lg border border-amber-200 bg-amber-50 px-4 py-3 text-sm font-medium text-amber-800 dark:border-amber-800 dark:bg-amber-950 dark:text-amber-300">
            未配置 AI。请在主窗口的<strong>设置 → AI 顾问</strong>填入 API 地址、密钥和模型名称。
          </div>
        )}

        {detail && !detail.selfParticipantId && (
          <div className="rounded-lg border border-amber-200 bg-amber-50 px-4 py-3 text-sm font-medium text-amber-800 dark:border-amber-800 dark:bg-amber-950 dark:text-amber-300">
            无法在本局中识别当前玩家，AI 分析已禁用。
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
              AI 分析{activeTone ? ` · ${tones.find((t) => t.id === activeTone)?.label}` : ""}
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
            详细数据
          </h2>
          {!detail && !detailError && (
            <div className="rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-900 px-5 py-6 text-sm text-zinc-500 dark:text-zinc-400">
              加载对局数据中…
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
              teamsLayoutClassName="xl:grid-cols-2"
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
