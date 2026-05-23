import { useCallback, useEffect, useState } from "react";

import {
  applyRunePage,
  deleteChampionRuneConfig,
  fetchChampionRuneConfig,
  fetchRuneRecommendations,
  saveChampionRuneConfig,
} from "../backend/leagueClient";
import type { ChampionRuneConfig, RunePage, RuneRecommendation } from "../backend/types";
import { useAppCore } from "../state/AppStateProvider";

const RUNE_STYLE_NAMES: Record<number, string> = {
  8000: "Precision",
  8100: "Domination",
  8200: "Sorcery",
  8300: "Inspiration",
  8400: "Resolve",
};

function styleLabel(styleId: number): string {
  return RUNE_STYLE_NAMES[styleId] ?? `Style ${styleId}`;
}

export function Rune({ lockedChampionId }: { lockedChampionId: number | null }) {
  const { t } = useAppCore();
  const [championName] = useState<string>(lockedChampionId ? `Champion ${lockedChampionId}` : "");
  const [recommendations, setRecommendations] = useState<RuneRecommendation[]>([]);
  const [savedConfig, setSavedConfig] = useState<ChampionRuneConfig | null>(null);
  const [autoApplied, setAutoApplied] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [applyingIndex, setApplyingIndex] = useState<number | null>(null);
  const [appliedIndex, setAppliedIndex] = useState<number | null>(null);

  const championId = lockedChampionId;

  const loadRuneData = useCallback(async (champId: number) => {
    setIsLoading(true);
    setAppliedIndex(null);
    try {
      const [recs, config] = await Promise.all([
        fetchRuneRecommendations(champId),
        fetchChampionRuneConfig(champId),
      ]);
      setRecommendations(recs);
      setSavedConfig(config);
    } finally {
      setIsLoading(false);
    }
  }, []);

  // Load rune data whenever a champion is locked in (or page mounts with one already locked).
  useEffect(() => {
    if (championId === null) {
      setRecommendations([]);
      setSavedConfig(null);
      setAutoApplied(false);
      return;
    }
    setAutoApplied(true);
    void loadRuneData(championId);
  }, [championId, loadRuneData]);

  const handleApply = useCallback(async (rec: RuneRecommendation, index: number) => {
    if (!championId) return;
    setApplyingIndex(index);
    try {
      await applyRunePage(championId, rec.page, championName);
      setAppliedIndex(index);
    } catch {
      // error handled silently — user can retry
    } finally {
      setApplyingIndex(null);
    }
  }, [championId, championName]);

  const handleSaveConfig = useCallback(async (page: RunePage) => {
    if (!championId) return;
    const saved = await saveChampionRuneConfig(championId, page);
    setSavedConfig(saved);
  }, [championId]);

  const handleDeleteConfig = useCallback(async () => {
    if (!championId) return;
    await deleteChampionRuneConfig(championId);
    setSavedConfig(null);
  }, [championId]);

  function positionLabel(position: string): string {
    const knownPositions: Record<string, string> = {
      top: t("rune.position.top"),
      jungle: t("rune.position.jungle"),
      middle: t("rune.position.middle"),
      bottom: t("rune.position.bottom"),
      support: t("rune.position.support"),
    };
    return knownPositions[position] ?? position;
  }

  return (
    <main className="min-h-0 flex-1 overflow-auto px-8 py-7">
      <div className="mx-auto flex w-full max-w-4xl flex-col gap-6">
        <header>
          <p className="text-sm font-medium uppercase tracking-wide text-rose-700">{t("rune.eyebrow")}</p>
          <h1 className="mt-2 text-3xl font-semibold text-zinc-950 dark:text-zinc-50">{t("rune.title")}</h1>
        </header>

        {!championId && (
          <div className="rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-900 p-8 text-center text-sm text-zinc-500 dark:text-zinc-400 shadow-sm">
            {t("rune.waiting")}
          </div>
        )}

        {championId && (
          <div className="flex flex-col gap-4">
            {autoApplied && (
              <div className="rounded-md border border-emerald-200 bg-emerald-50 px-4 py-2 text-sm font-medium text-emerald-800">
                {t("rune.autoApplied")}
              </div>
            )}

            {isLoading && (
              <div className="text-sm text-zinc-500 dark:text-zinc-400">{t("common.loading")}</div>
            )}

            {savedConfig && (
              <div className="rounded-lg border border-rose-200 bg-white dark:bg-zinc-900 p-5 shadow-sm">
                <div className="flex items-center justify-between gap-3">
                  <div>
                    <p className="text-xs font-semibold uppercase tracking-wide text-rose-700">{t("rune.savedConfig")}</p>
                    <p className="mt-1 text-sm font-medium text-zinc-950 dark:text-zinc-50">
                      {styleLabel(savedConfig.page.primaryStyleId)} + {styleLabel(savedConfig.page.subStyleId)}
                    </p>
                  </div>
                  <div className="flex gap-2">
                    <button
                      type="button"
                      onClick={() => void handleApply({ position: "", pickCount: 0, page: savedConfig.page }, -1)}
                      disabled={applyingIndex === -1}
                      className="inline-flex h-8 items-center rounded-md bg-rose-700 px-3 text-xs font-semibold text-white transition hover:bg-rose-800 disabled:bg-zinc-300"
                    >
                      {applyingIndex === -1 ? t("rune.applying") : appliedIndex === -1 ? t("rune.applied") : t("rune.apply")}
                    </button>
                    <button
                      type="button"
                      onClick={() => void handleDeleteConfig()}
                      className="inline-flex h-8 items-center rounded-md border border-zinc-300 dark:border-zinc-600 px-3 text-xs font-semibold text-zinc-700 dark:text-zinc-300 transition hover:bg-zinc-50 dark:hover:bg-zinc-800"
                    >
                      {t("rune.deleteConfig")}
                    </button>
                  </div>
                </div>
              </div>
            )}

            {!isLoading && recommendations.length === 0 && (
              <div className="rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-900 p-6 text-sm text-zinc-500 dark:text-zinc-400 shadow-sm">
                {t("rune.noRecommendations")}
              </div>
            )}

            {recommendations.map((rec, index) => (
              <div
                key={index}
                className="rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-900 p-5 shadow-sm"
              >
                <div className="flex items-start justify-between gap-3">
                  <div className="min-w-0">
                    <div className="flex items-center gap-2">
                      <span className="inline-flex items-center rounded-md bg-zinc-100 dark:bg-zinc-900 px-2 py-0.5 text-xs font-semibold text-zinc-700 dark:text-zinc-300">
                        {positionLabel(rec.position)}
                      </span>
                      <span className="text-xs text-zinc-500 dark:text-zinc-400">
                        {rec.pickCount.toLocaleString()} {t("rune.pickCount")}
                      </span>
                    </div>
                    <p className="mt-2 text-sm font-semibold text-zinc-950 dark:text-zinc-50">
                      {styleLabel(rec.page.primaryStyleId)}
                      <span className="mx-1 font-normal text-zinc-400 dark:text-zinc-500">+</span>
                      {styleLabel(rec.page.subStyleId)}
                    </p>
                    <p className="mt-1 font-mono text-xs text-zinc-400 dark:text-zinc-500">
                      {rec.page.selectedPerkIds.join(" · ")}
                    </p>
                  </div>
                  <div className="flex shrink-0 flex-col gap-2">
                    <button
                      type="button"
                      onClick={() => void handleApply(rec, index)}
                      disabled={applyingIndex === index}
                      className="inline-flex h-8 items-center rounded-md bg-rose-700 px-3 text-xs font-semibold text-white transition hover:bg-rose-800 disabled:bg-zinc-300"
                    >
                      {applyingIndex === index
                        ? t("rune.applying")
                        : appliedIndex === index
                          ? t("rune.applied")
                          : t("rune.apply")}
                    </button>
                    <button
                      type="button"
                      onClick={() => void handleSaveConfig(rec.page)}
                      className="inline-flex h-8 items-center rounded-md border border-zinc-300 dark:border-zinc-600 px-3 text-xs font-semibold text-zinc-700 dark:text-zinc-300 transition hover:bg-zinc-50 dark:hover:bg-zinc-800"
                    >
                      {t("rune.saveConfig")}
                    </button>
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </main>
  );
}
