import { useCallback, useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";

import { useAppCore } from "../state/AppStateProvider";
import { fetchAiAnalysis } from "../backend/leagueClient";
import { isCommandError } from "../backend/commands";
import type { AiAnalysisCache } from "../backend/types";
import type { TranslationKey } from "../i18n";

type AnalysisScope = "all" | "top" | "jungle" | "middle" | "bottom" | "support";
type AnalysisTone = "objective" | "rage" | "flatter";

const tones: Array<{ id: AnalysisTone; labelKey: TranslationKey; className: string }> = [
  { id: "objective", labelKey: "recap.toneObjective", className: "border-zinc-300 dark:border-zinc-600 text-zinc-700 dark:text-zinc-300 hover:bg-zinc-50 dark:hover:bg-zinc-800 data-[active=true]:bg-zinc-950 data-[active=true]:text-white dark:data-[active=true]:bg-zinc-100 dark:data-[active=true]:text-zinc-900 data-[active=true]:border-zinc-950" },
  { id: "rage",      labelKey: "recap.toneRage", className: "border-orange-300 dark:border-orange-700 text-orange-700 dark:text-orange-400 hover:bg-orange-50 dark:hover:bg-orange-950 data-[active=true]:bg-orange-600 data-[active=true]:text-white data-[active=true]:border-orange-600" },
  { id: "flatter",   labelKey: "recap.toneFlatter", className: "border-pink-300 dark:border-pink-700 text-pink-700 dark:text-pink-400 hover:bg-pink-50 dark:hover:bg-pink-950 data-[active=true]:bg-pink-500 data-[active=true]:text-white data-[active=true]:border-pink-500" },
];

const scopes: Array<{ id: AnalysisScope; labelKey: TranslationKey }> = [
  { id: "all", labelKey: "aiAdvisor.scopeAll" },
  { id: "top", labelKey: "aiAdvisor.scopeTop" },
  { id: "jungle", labelKey: "aiAdvisor.scopeJungle" },
  { id: "middle", labelKey: "aiAdvisor.scopeMiddle" },
  { id: "bottom", labelKey: "aiAdvisor.scopeBottom" },
  { id: "support", labelKey: "aiAdvisor.scopeSupport" },
];

const NEW_GAME_THRESHOLD = 5;

export function Advisor({ onOpenSettings }: { onOpenSettings: () => void }) {
  const { snapshot, t } = useAppCore();
  const [scope, setScope] = useState<AnalysisScope>("all");
  const [tone, setTone] = useState<AnalysisTone>("objective");
  const [cached, setCached] = useState<AiAnalysisCache | null>(null);
  const [streaming, setStreaming] = useState("");
  const [isAnalyzing, setIsAnalyzing] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [currentGameCount, setCurrentGameCount] = useState<number | null>(null);
  const unlistenersRef = useRef<Array<() => void>>([]);

  const aiConfigured = Boolean(
    snapshot?.settings.aiBaseUrl &&
    snapshot?.settings.aiApiKey &&
    snapshot?.settings.aiModel,
  );

  // Load cached result when scope changes
  useEffect(() => {
    setCached(null);
    setError(null);
    void fetchAiAnalysis(scope).then((result) => {
      setCached(result);
    }).catch(() => null);
  }, [scope]);

  // Get current game count to detect new games since last analysis
  useEffect(() => {
    if (!cached) return;
    void invoke<{ recentMatches: unknown[] }>("get_league_self_snapshot", {
      input: { matchLimit: 100 },
    })
      .then((s) => setCurrentGameCount(s.recentMatches.length))
      .catch(() => null);
  }, [cached]);

  const newGamesSinceAnalysis =
    cached && currentGameCount !== null
      ? Math.max(0, currentGameCount - cached.gameCountAtAnalysis)
      : 0;
  const shouldPromptRefresh = newGamesSinceAnalysis >= NEW_GAME_THRESHOLD;

  const stopListeners = useCallback(() => {
    unlistenersRef.current.forEach((fn) => fn());
    unlistenersRef.current = [];
  }, []);

  const runAnalysis = useCallback(async () => {
    if (isAnalyzing) return;
    setError(null);
    setStreaming("");
    setIsAnalyzing(true);
    stopListeners();

    try {
      const [chunkUnlisten, doneUnlisten, errorUnlisten] = await Promise.all([
        listen<string>("ai-analysis-chunk", (event) => {
          setStreaming((prev) => prev + event.payload);
        }),
        listen<string>("ai-analysis-done", (event) => {
          setCached({
            scope,
            resultText: event.payload,
            gameCountAtAnalysis: 0,
            analyzedAt: new Date().toISOString(),
          });
          setStreaming("");
          setIsAnalyzing(false);
          stopListeners();
          // Reload the cached version from backend to get accurate gameCount
          void fetchAiAnalysis(scope).then((r) => { if (r) setCached(r); }).catch(() => null);
        }),
        listen<string>("ai-analysis-error", (event) => {
          setError(event.payload);
          setIsAnalyzing(false);
          stopListeners();
        }),
      ]);

      unlistenersRef.current = [chunkUnlisten, doneUnlisten, errorUnlisten];

      await invoke("run_ai_analysis", { scope, tone });
    } catch (err: unknown) {
      const message =
        typeof err === "string"
          ? err
          : isCommandError(err)
          ? err.message
          : err instanceof Error
          ? err.message
          : t("recap.aiStartFailed");
      setError(message);
      setIsAnalyzing(false);
      stopListeners();
    }
  }, [isAnalyzing, scope, tone, stopListeners, t]);

  useEffect(() => () => stopListeners(), [stopListeners]);

  const displayText = isAnalyzing ? streaming : (cached?.resultText ?? "");

  return (
    <main className="min-h-0 flex-1 overflow-auto px-8 py-7">
      <div className="mx-auto flex w-full max-w-4xl flex-col gap-6">
        <header>
          <p className="text-sm font-medium uppercase tracking-wide text-rose-700">{t("aiAdvisor.eyebrow")}</p>
          <h1 className="mt-2 text-3xl font-semibold text-zinc-950 dark:text-zinc-50">{t("aiAdvisor.title")}</h1>
          <p className="mt-1 text-sm text-zinc-500 dark:text-zinc-400">
            {t("aiAdvisor.subtitle")}
          </p>
        </header>

        {!aiConfigured && (
          <div className="flex flex-col items-start gap-3 rounded-lg border border-amber-200 bg-amber-50 px-5 py-4 text-sm font-medium text-amber-800 dark:border-amber-800 dark:bg-amber-950 dark:text-amber-300">
            <span>
              {t("aiAdvisor.notConfiguredPrefix")}
              <strong>{t("aiAdvisor.notConfiguredPath")}</strong>
              {t("aiAdvisor.notConfiguredSuffix")}
            </span>
            <button
              type="button"
              onClick={onOpenSettings}
              className="inline-flex h-8 items-center rounded-md bg-amber-700 px-3 text-xs font-semibold text-white transition hover:bg-amber-800"
            >
              {t("home.configureAi")}
            </button>
          </div>
        )}

        <div className="flex flex-wrap items-center gap-3">
          <div className="flex rounded-md border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-900">
            {scopes.map((s) => (
              <button
                key={s.id}
                type="button"
                onClick={() => { if (!isAnalyzing) setScope(s.id); }}
                className={[
                  "h-9 px-3 text-sm font-semibold transition first:rounded-l-md last:rounded-r-md",
                  scope === s.id
                    ? "bg-zinc-950 text-white dark:bg-zinc-100 dark:text-zinc-900"
                    : "text-zinc-600 dark:text-zinc-400 hover:bg-zinc-50 dark:hover:bg-zinc-800",
                ].join(" ")}
              >
                {t(s.labelKey)}
              </button>
            ))}
          </div>

          <div className="flex gap-2">
            {tones.map((toneOption) => (
              <button
                key={toneOption.id}
                type="button"
                disabled={isAnalyzing}
                data-active={tone === toneOption.id}
                onClick={() => { if (!isAnalyzing) setTone(toneOption.id); }}
                className={`h-9 rounded-md border px-3 text-sm font-semibold transition disabled:cursor-not-allowed disabled:opacity-50 ${toneOption.className}`}
              >
                {t(toneOption.labelKey)}
              </button>
            ))}
          </div>

          <button
            type="button"
            disabled={isAnalyzing || !aiConfigured}
            onClick={() => void runAnalysis()}
            className="inline-flex h-9 items-center gap-2 rounded-md bg-rose-700 px-4 text-sm font-semibold text-white shadow-sm transition hover:bg-rose-800 disabled:cursor-not-allowed disabled:opacity-60"
          >
            {isAnalyzing ? (
              <>
                <SpinnerIcon />
                <span>{t("aiAdvisor.analyzing")}</span>
              </>
            ) : (
              <span>{cached ? t("aiAdvisor.reanalyze") : t("aiAdvisor.analyze")}</span>
            )}
          </button>
        </div>

        {shouldPromptRefresh && !isAnalyzing && (
          <div className="rounded-lg border border-sky-200 bg-sky-50 px-4 py-3 text-sm font-medium text-sky-800 dark:border-sky-800 dark:bg-sky-950 dark:text-sky-300">
            {t("aiAdvisor.newGamesPrefix")}{newGamesSinceAnalysis}{t("aiAdvisor.newGamesSuffix")}
          </div>
        )}

        {error && (
          <div className="rounded-lg border border-rose-200 bg-rose-50 px-4 py-3 text-sm font-medium text-rose-800 dark:border-rose-800 dark:bg-rose-950 dark:text-rose-300">
            {error}
          </div>
        )}

        {(displayText || isAnalyzing) && (
          <div className="rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-900 shadow-sm">
            {cached && !isAnalyzing && (
              <div className="border-b border-zinc-100 dark:border-zinc-700 px-5 py-2 text-xs text-zinc-400">
                {t("aiAdvisor.analyzedAt")}{formatAnalyzedAt(cached.analyzedAt)}
              </div>
            )}
            <div className="px-5 py-5">
              <AnalysisText text={displayText} isStreaming={isAnalyzing} />
            </div>
          </div>
        )}

        {!displayText && !isAnalyzing && !error && (
          <div className="flex flex-col items-center justify-center gap-3 rounded-lg border border-dashed border-zinc-300 dark:border-zinc-600 py-16 text-center">
            <p className="text-sm font-medium text-zinc-500 dark:text-zinc-400">
              {aiConfigured ? t("aiAdvisor.emptyConfigured") : t("aiAdvisor.emptyUnconfigured")}
            </p>
            {!aiConfigured && (
              <button
                type="button"
                onClick={onOpenSettings}
                className="inline-flex h-9 items-center rounded-md bg-rose-700 px-4 text-sm font-semibold text-white shadow-sm transition hover:bg-rose-800"
              >
                {t("home.configureAi")}
              </button>
            )}
          </div>
        )}
      </div>
    </main>
  );
}

function AnalysisText({ text, isStreaming }: { text: string; isStreaming: boolean }) {
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

function SpinnerIcon() {
  return (
    <svg className="h-4 w-4 animate-spin" viewBox="0 0 24 24" fill="none">
      <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
      <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8v4a4 4 0 00-4 4H4z" />
    </svg>
  );
}

function formatAnalyzedAt(iso: string): string {
  try { return new Date(iso).toLocaleString(); } catch { return iso; }
}
