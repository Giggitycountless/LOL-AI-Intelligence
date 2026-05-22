import { useCallback, useEffect, useRef, useState } from "react";

import { useAppCore } from "../state/AppStateProvider";
import { fetchAiConfig, fetchAiAnalysis, saveAiAnalysis, fetchLeagueSelfSnapshot } from "../backend/leagueClient";
import type { AiAnalysisCache, AiConfig } from "../backend/types";

type AnalysisScope = "all" | "top" | "jungle" | "middle" | "bottom" | "support";

const scopes: Array<{ id: AnalysisScope; label: string }> = [
  { id: "all", label: "全部" },
  { id: "top", label: "上路" },
  { id: "jungle", label: "打野" },
  { id: "middle", label: "中路" },
  { id: "bottom", label: "下路" },
  { id: "support", label: "辅助" },
];

const NEW_GAME_THRESHOLD = 5;

export function Advisor() {
  const { snapshot, effectiveLanguage } = useAppCore();
  const [scope, setScope] = useState<AnalysisScope>("all");
  const [cached, setCached] = useState<AiAnalysisCache | null>(null);
  const [streaming, setStreaming] = useState("");
  const [isAnalyzing, setIsAnalyzing] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [currentGameCount, setCurrentGameCount] = useState<number | null>(null);
  const abortRef = useRef<AbortController | null>(null);

  const aiConfigured = Boolean(
    snapshot?.settings.aiBaseUrl && snapshot?.settings.aiApiKey && snapshot?.settings.aiModel
  );

  // Load cached result and current game count when scope changes
  useEffect(() => {
    setCached(null);
    setError(null);
    void fetchAiAnalysis(scope).then(setCached).catch(() => null);
    void fetchLeagueSelfSnapshot({ matchLimit: 50 })
      .then((s) => setCurrentGameCount(s.recentMatches.length))
      .catch(() => null);
  }, [scope]);

  const newGamesSinceAnalysis =
    cached && currentGameCount !== null
      ? Math.max(0, currentGameCount - cached.gameCountAtAnalysis)
      : 0;
  const shouldPromptRefresh = newGamesSinceAnalysis >= NEW_GAME_THRESHOLD;

  const runAnalysis = useCallback(async () => {
    if (isAnalyzing) return;
    setError(null);
    setStreaming("");
    setIsAnalyzing(true);

    abortRef.current?.abort();
    const abort = new AbortController();
    abortRef.current = abort;

    try {
      const [config, selfSnapshot] = await Promise.all([
        fetchAiConfig(),
        fetchLeagueSelfSnapshot({ matchLimit: 50 }),
      ]);

      if (!config.baseUrl || !config.apiKey || !config.model) {
        setError("请先在设置中配置 AI API Key、地址和模型。");
        return;
      }

      const prompt = buildPrompt(selfSnapshot, scope, effectiveLanguage);
      const full = await streamAnalysis(config, prompt, abort.signal, (chunk) => {
        setStreaming((prev) => prev + chunk);
      });

      const gameCount = selfSnapshot.recentMatches.length;
      await saveAiAnalysis(scope, full, gameCount);
      setCached({
        scope,
        resultText: full,
        gameCountAtAnalysis: gameCount,
        analyzedAt: new Date().toISOString(),
      });
      setStreaming("");
    } catch (err: unknown) {
      if ((err as { name?: string }).name === "AbortError") return;
      setError(err instanceof Error ? err.message : "分析请求失败，请检查 API 配置后重试。");
    } finally {
      setIsAnalyzing(false);
    }
  }, [isAnalyzing, scope, effectiveLanguage]);

  // Cleanup on unmount
  useEffect(() => () => { abortRef.current?.abort(); }, []);

  const displayText = isAnalyzing ? streaming : (cached?.resultText ?? "");

  return (
    <main className="min-h-0 flex-1 overflow-auto px-8 py-7">
      <div className="mx-auto flex w-full max-w-4xl flex-col gap-6">
        <header>
          <p className="text-sm font-medium uppercase tracking-wide text-rose-700">AI 顾问</p>
          <h1 className="mt-2 text-3xl font-semibold text-zinc-950 dark:text-zinc-50">个人数据分析</h1>
          <p className="mt-1 text-sm text-zinc-500 dark:text-zinc-400">
            基于近50场对局，AI 分析你的优势、弱点和改进方向
          </p>
        </header>

        {!aiConfigured && (
          <div className="rounded-lg border border-amber-200 bg-amber-50 px-5 py-4 text-sm font-medium text-amber-800 dark:border-amber-800 dark:bg-amber-950 dark:text-amber-300">
            未配置 AI。请前往<strong>设置 → AI 顾问</strong>填入 API 地址、密钥和模型名称。
          </div>
        )}

        <div className="flex flex-wrap items-center gap-3">
          <div className="flex rounded-md border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-900">
            {scopes.map((s) => (
              <button
                key={s.id}
                type="button"
                onClick={() => setScope(s.id)}
                className={[
                  "h-9 px-3 text-sm font-semibold transition first:rounded-l-md last:rounded-r-md",
                  scope === s.id
                    ? "bg-zinc-950 text-white dark:bg-zinc-100 dark:text-zinc-900"
                    : "text-zinc-600 dark:text-zinc-400 hover:bg-zinc-50 dark:hover:bg-zinc-800",
                ].join(" ")}
              >
                {s.label}
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
                <span>分析中…</span>
              </>
            ) : (
              <span>{cached ? "重新分析" : "开始分析"}</span>
            )}
          </button>
        </div>

        {shouldPromptRefresh && !isAnalyzing && (
          <div className="rounded-lg border border-sky-200 bg-sky-50 px-4 py-3 text-sm font-medium text-sky-800 dark:border-sky-800 dark:bg-sky-950 dark:text-sky-300">
            已有 {newGamesSinceAnalysis} 场新对局，建议重新分析以获取最新建议。
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
                分析时间：{formatAnalyzedAt(cached.analyzedAt)}
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
              {aiConfigured ? "选择位置范围，点击「开始分析」" : "请先在设置中配置 AI"}
            </p>
          </div>
        )}
      </div>
    </main>
  );
}

// ── Streaming helper ─────────────────────────────────────────────────────────

async function streamAnalysis(
  config: AiConfig,
  prompt: string,
  signal: AbortSignal,
  onChunk: (chunk: string) => void,
): Promise<string> {
  const base = (config.baseUrl ?? "").replace(/\/$/, "");
  const response = await fetch(`${base}/chat/completions`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${config.apiKey}`,
    },
    body: JSON.stringify({
      model: config.model,
      stream: true,
      messages: [{ role: "user", content: prompt }],
    }),
    signal,
  });

  if (!response.ok) {
    const body = await response.text().catch(() => "");
    throw new Error(`API 返回 ${response.status}: ${body.slice(0, 200)}`);
  }

  const reader = response.body?.getReader();
  if (!reader) throw new Error("无法读取响应流");

  const decoder = new TextDecoder();
  let full = "";

  while (true) {
    const { done, value } = await reader.read();
    if (done) break;
    const lines = decoder.decode(value, { stream: true }).split("\n");
    for (const line of lines) {
      if (!line.startsWith("data: ")) continue;
      const data = line.slice(6).trim();
      if (data === "[DONE]") break;
      try {
        const json = JSON.parse(data) as { choices?: Array<{ delta?: { content?: string } }> };
        const chunk = json.choices?.[0]?.delta?.content;
        if (chunk) {
          full += chunk;
          onChunk(chunk);
        }
      } catch {
        // ignore malformed SSE lines
      }
    }
  }

  return full;
}

// ── Prompt builder ───────────────────────────────────────────────────────────

function buildPrompt(
  selfSnapshot: Awaited<ReturnType<typeof fetchLeagueSelfSnapshot>>,
  scope: AnalysisScope,
  language: string,
): string {
  const langInstruction =
    language === "zh" || language === "system"
      ? "请用中文回复。"
      : "Please reply in English.";

  // Lane filtering is not yet available in RecentMatchSummary; scope is passed
  // to the AI as context so it can interpret the data with that framing.
  const matches = selfSnapshot.recentMatches;

  const totalGames = matches.length;
  const wins = matches.filter((m) => m.result === "win").length;
  const winRate = totalGames > 0 ? ((wins / totalGames) * 100).toFixed(1) : "N/A";

  const avgKda =
    totalGames > 0
      ? (
          matches.reduce((sum, m) => sum + (m.kda ?? 0), 0) / totalGames
        ).toFixed(2)
      : "N/A";

  const champCount: Record<string, number> = {};
  for (const m of matches) {
    if (m.championName) champCount[m.championName] = (champCount[m.championName] ?? 0) + 1;
  }
  const topChamps = Object.entries(champCount)
    .sort((a, b) => b[1] - a[1])
    .slice(0, 5)
    .map(([name, n]) => `${name}(${n}场)`)
    .join(", ");

  const ranked = selfSnapshot.rankedQueues.find((q) => q.queue === "soloDuo");
  const rankStr = ranked?.isRanked
    ? `${ranked.tier ?? ""} ${ranked.division ?? ""} ${ranked.leaguePoints ?? 0}LP（${ranked.wins}胜${ranked.losses}负）`
    : "未排名";

  const matchDetails = matches
    .slice(0, 50)
    .map((m) => {
      const result = m.result === "win" ? "胜" : "败";
      return `${m.championName ?? "?"} ${result} KDA:${m.kills ?? 0}/${m.deaths ?? 0}/${m.assists ?? 0}`;
    })
    .join("\n");

  const scopeLabel = scopes.find((s) => s.id === scope)?.label ?? "全部";

  return `${langInstruction}

你是一位英雄联盟教练，请根据以下玩家数据进行分析，给出简洁、实用的建议。

## 玩家信息
- 段位：${rankStr}
- 分析范围：${scopeLabel}（近${totalGames}场）
- 胜率：${winRate}%
- 平均KDA：${avgKda}
- 常用英雄：${topChamps || "无数据"}

## 近期对局明细
${matchDetails || "无数据"}

## 输出格式（请严格遵守）

**优势**
（2-3条，每条一行，具体说明做得好的方面）

**弱点**
（2-3条，每条一行，具体说明需要改进的问题）

**本阶段改进重点**
（1条最重要的改进建议，必须具体可执行）

---

**详细分析**
（3-5段文字，深入分析表现规律、英雄池选择、与当前段位的差距等）`;
}

// ── UI components ─────────────────────────────────────────────────────────────

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
        if (line.trim() === "") {
          return <div key={i} className="h-1" />;
        }
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
  try {
    return new Date(iso).toLocaleString();
  } catch {
    return iso;
  }
}
