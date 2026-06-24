import { useCallback, useEffect, useState } from "react";

import {
  applyRunePage,
  deleteChampionRuneConfig,
  fetchChampionRuneConfig,
  fetchRuneRecommendations,
  saveChampionRuneConfig,
} from "../backend/leagueClient";
import type { ChampionRuneConfig, RunePage, RuneRecommendation } from "../backend/types";
import { useAppCore, useLeagueAssets, leagueGameAssetKey } from "../state/AppStateProvider";
import type { LeagueGameAssetView } from "../state/types";

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
  const { leagueImages, loadLeagueGameAsset } = useLeagueAssets();
  const [championName] = useState<string>(lockedChampionId ? `Champion ${lockedChampionId}` : "");
  const [recommendations, setRecommendations] = useState<RuneRecommendation[]>([]);
  const [savedConfig, setSavedConfig] = useState<ChampionRuneConfig | null>(null);
  const [autoApplied, setAutoApplied] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [applyingIndex, setApplyingIndex] = useState<number | null>(null);
  const [appliedIndex, setAppliedIndex] = useState<number | null>(null);
  const [applyErrorIndex, setApplyErrorIndex] = useState<number | null>(null);
  const [configError, setConfigError] = useState(false);

  const championId = lockedChampionId;

  const loadRuneData = useCallback(async (champId: number) => {
    setIsLoading(true);
    setAppliedIndex(null);
    setApplyErrorIndex(null);
    setConfigError(false);
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

  // Load rune icons when recommendations change
  useEffect(() => {
    const ids = new Set<number>();
    for (const rec of recommendations) {
      ids.add(rec.page.primaryStyleId);
      ids.add(rec.page.subStyleId);
      rec.page.selectedPerkIds.forEach((id) => ids.add(id));
    }
    if (savedConfig) {
      ids.add(savedConfig.page.primaryStyleId);
      ids.add(savedConfig.page.subStyleId);
      savedConfig.page.selectedPerkIds.forEach((id) => ids.add(id));
    }
    ids.forEach((id) => void loadLeagueGameAsset("rune", id));
  }, [recommendations, savedConfig, loadLeagueGameAsset]);

  const handleApply = useCallback(async (rec: RuneRecommendation, index: number) => {
    if (!championId) return;
    setApplyingIndex(index);
    setApplyErrorIndex(null);
    setAppliedIndex((current) => (current === index ? null : current));
    try {
      await applyRunePage(championId, rec.page, championName);
      setAppliedIndex(index);
    } catch {
      setApplyErrorIndex(index);
    } finally {
      setApplyingIndex(null);
    }
  }, [championId, championName]);

  const handleSaveConfig = useCallback(async (page: RunePage) => {
    if (!championId) return;
    setConfigError(false);
    try {
      const saved = await saveChampionRuneConfig(championId, page);
      setSavedConfig(saved);
    } catch {
      setConfigError(true);
    }
  }, [championId]);

  const handleDeleteConfig = useCallback(async () => {
    if (!championId) return;
    setConfigError(false);
    try {
      await deleteChampionRuneConfig(championId);
      setSavedConfig(null);
    } catch {
      setConfigError(true);
    }
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

            {configError && (
              <div className="rounded-md border border-rose-200 dark:border-rose-800 bg-rose-50 dark:bg-rose-950 px-4 py-2 text-sm font-medium text-rose-800 dark:text-rose-300">
                {t("rune.configFailed")}
              </div>
            )}

            {isLoading && (
              <div className="text-sm text-zinc-500 dark:text-zinc-400">{t("common.loading")}</div>
            )}

            {savedConfig && (
              <RuneCard
                rec={{ position: "", pickCount: 0, page: savedConfig.page }}
                index={-1}
                gameAssets={leagueImages.gameAssets}
                applyingIndex={applyingIndex}
                appliedIndex={appliedIndex}
                applyErrorIndex={applyErrorIndex}
                isSavedConfig
                onApply={handleApply}
                onSave={handleSaveConfig}
                onDelete={handleDeleteConfig}
                positionLabel={positionLabel}
                t={t}
              />
            )}

            {!isLoading && recommendations.length === 0 && (
              <div className="rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-900 p-6 text-sm text-zinc-500 dark:text-zinc-400 shadow-sm">
                {t("rune.noRecommendations")}
              </div>
            )}

            {recommendations.map((rec, index) => (
              <RuneCard
                key={index}
                rec={rec}
                index={index}
                gameAssets={leagueImages.gameAssets}
                applyingIndex={applyingIndex}
                appliedIndex={appliedIndex}
                applyErrorIndex={applyErrorIndex}
                isSavedConfig={false}
                onApply={handleApply}
                onSave={handleSaveConfig}
                onDelete={null}
                positionLabel={positionLabel}
                t={t}
              />
            ))}
          </div>
        )}
      </div>
    </main>
  );
}

// ── RuneCard ─────────────────────────────────────────────────────────────────

function RuneCard({
  rec,
  index,
  gameAssets,
  applyingIndex,
  appliedIndex,
  applyErrorIndex,
  isSavedConfig,
  onApply,
  onSave,
  onDelete,
  positionLabel,
  t,
}: {
  rec: RuneRecommendation;
  index: number;
  gameAssets: Record<string, LeagueGameAssetView>;
  applyingIndex: number | null;
  appliedIndex: number | null;
  applyErrorIndex: number | null;
  isSavedConfig: boolean;
  onApply: (rec: RuneRecommendation, index: number) => void;
  onSave: (page: RunePage) => void;
  onDelete: (() => void) | null;
  positionLabel: (pos: string) => string;
  t: (key: import("../i18n").TranslationKey) => string;
}) {
  const { page } = rec;

  // selectedPerkIds layout (typical 9-slot page):
  // [0]     keystone
  // [1..3]  primary path rows
  // [4..5]  secondary path picks
  // [6..8]  stat shards
  const keystone = page.selectedPerkIds[0];
  const primarySlots = page.selectedPerkIds.slice(1, 4);
  const secondarySlots = page.selectedPerkIds.slice(4, 6);
  const shards = page.selectedPerkIds.slice(6, 9);

  const primaryStyleAsset = gameAssets[leagueGameAssetKey("rune", page.primaryStyleId)];
  const subStyleAsset = gameAssets[leagueGameAssetKey("rune", page.subStyleId)];
  const keystoneAsset = keystone ? gameAssets[leagueGameAssetKey("rune", keystone)] : undefined;

  const borderClass = isSavedConfig ? "border-rose-200 dark:border-rose-800" : "border-zinc-200 dark:border-zinc-700";

  return (
    <div className={`rounded-lg border ${borderClass} bg-white dark:bg-zinc-900 p-5 shadow-sm`}>
      {/* Header row */}
      <div className="flex items-start justify-between gap-4">
        <div className="flex min-w-0 flex-1 flex-col gap-2">
          {/* Label row */}
          <div className="flex items-center gap-2">
            {isSavedConfig ? (
              <span className="inline-flex items-center rounded-md bg-rose-100 dark:bg-rose-900 px-2 py-0.5 text-xs font-semibold text-rose-700 dark:text-rose-300">
                {t("rune.savedConfig")}
              </span>
            ) : (
              <>
                {rec.position && (
                  <span className="inline-flex items-center rounded-md bg-zinc-100 dark:bg-zinc-800 px-2 py-0.5 text-xs font-semibold text-zinc-700 dark:text-zinc-300">
                    {positionLabel(rec.position)}
                  </span>
                )}
                <span className="text-xs text-zinc-500 dark:text-zinc-400">
                  {rec.pickCount.toLocaleString()} {t("rune.pickCount")}
                </span>
              </>
            )}
          </div>

          {/* Style path row */}
          <div className="flex items-center gap-3">
            <StylePill asset={primaryStyleAsset} styleId={page.primaryStyleId} label={styleLabel(page.primaryStyleId)} />
            <span className="text-xs font-medium text-zinc-400 dark:text-zinc-500">+</span>
            <StylePill asset={subStyleAsset} styleId={page.subStyleId} label={styleLabel(page.subStyleId)} />
          </div>

          {/* Rune grid */}
          <div className="mt-1 flex flex-wrap items-start gap-4">
            {/* Primary path */}
            <div className="flex items-center gap-1.5">
              {/* Keystone (slightly larger) */}
              {keystone !== undefined && (
                <RuneIcon
                  asset={keystoneAsset}
                  perkId={keystone}
                  size="lg"
                />
              )}
              {/* Primary rows */}
              {primarySlots.map((perkId, i) => (
                <RuneIcon
                  key={`primary-${i}`}
                  asset={gameAssets[leagueGameAssetKey("rune", perkId)]}
                  perkId={perkId}
                  size="md"
                />
              ))}
            </div>

            {/* Divider */}
            <div className="self-stretch w-px bg-zinc-200 dark:bg-zinc-700" />

            {/* Secondary path */}
            <div className="flex items-center gap-1.5">
              {secondarySlots.map((perkId, i) => (
                <RuneIcon
                  key={`secondary-${i}`}
                  asset={gameAssets[leagueGameAssetKey("rune", perkId)]}
                  perkId={perkId}
                  size="md"
                />
              ))}
            </div>

            {/* Stat shards */}
            {shards.length > 0 && (
              <>
                <div className="self-stretch w-px bg-zinc-200 dark:bg-zinc-700" />
                <div className="flex items-center gap-1.5">
                  {shards.map((perkId, i) => (
                    <RuneIcon
                      key={`shard-${i}`}
                      asset={gameAssets[leagueGameAssetKey("rune", perkId)]}
                      perkId={perkId}
                      size="sm"
                    />
                  ))}
                </div>
              </>
            )}
          </div>
        </div>

        {/* Action buttons */}
        <div className="flex shrink-0 flex-col gap-2">
          <button
            type="button"
            onClick={() => onApply(rec, index)}
            disabled={applyingIndex === index}
            className="inline-flex h-8 items-center rounded-md bg-rose-700 px-3 text-xs font-semibold text-white transition hover:bg-rose-800 disabled:bg-zinc-300 dark:disabled:bg-zinc-700"
          >
            {applyingIndex === index
              ? t("rune.applying")
              : appliedIndex === index
                ? t("rune.applied")
                : t("rune.apply")}
          </button>
          {!isSavedConfig && (
            <button
              type="button"
              onClick={() => onSave(page)}
              className="inline-flex h-8 items-center rounded-md border border-zinc-300 dark:border-zinc-600 px-3 text-xs font-semibold text-zinc-700 dark:text-zinc-300 transition hover:bg-zinc-50 dark:hover:bg-zinc-800"
            >
              {t("rune.saveConfig")}
            </button>
          )}
          {isSavedConfig && onDelete && (
            <button
              type="button"
              onClick={onDelete}
              className="inline-flex h-8 items-center rounded-md border border-zinc-300 dark:border-zinc-600 px-3 text-xs font-semibold text-zinc-700 dark:text-zinc-300 transition hover:bg-zinc-50 dark:hover:bg-zinc-800"
            >
              {t("rune.deleteConfig")}
            </button>
          )}
        </div>
      </div>

      {applyErrorIndex === index && (
        <p className="mt-3 text-xs font-medium text-rose-700 dark:text-rose-400">{t("rune.applyFailed")}</p>
      )}
    </div>
  );
}

// ── StylePill ─────────────────────────────────────────────────────────────────

function StylePill({
  asset,
  styleId,
  label,
}: {
  asset: LeagueGameAssetView | undefined;
  styleId: number;
  label: string;
}) {
  return (
    <div className="flex items-center gap-1.5" title={label}>
      <span className="inline-flex h-5 w-5 shrink-0 items-center justify-center overflow-hidden rounded-full bg-zinc-800">
        {asset ? (
          <img alt={label} className="h-full w-full object-cover" src={asset.imageUrl} />
        ) : (
          <span className="text-[8px] font-bold text-zinc-400">{styleId}</span>
        )}
      </span>
      <span className="text-xs font-semibold text-zinc-700 dark:text-zinc-300">{label}</span>
    </div>
  );
}

// ── RuneIcon ──────────────────────────────────────────────────────────────────

function RuneIcon({
  asset,
  perkId,
  size,
}: {
  asset: LeagueGameAssetView | undefined;
  perkId: number;
  size: "sm" | "md" | "lg";
}) {
  const sizeClass = size === "lg" ? "h-9 w-9" : size === "md" ? "h-7 w-7" : "h-5 w-5";
  const label = asset?.name ?? String(perkId);
  const title = asset?.description ? `${label}\n${asset.description}` : label;

  return (
    <span
      className={[
        "group relative inline-flex shrink-0 items-center justify-center overflow-hidden rounded-full border border-zinc-700 bg-zinc-800",
        sizeClass,
      ].join(" ")}
      title={title}
    >
      {asset ? (
        <img alt={label} className="h-full w-full object-cover" src={asset.imageUrl} />
      ) : (
        <span className="text-[8px] font-semibold text-zinc-500">{perkId}</span>
      )}
    </span>
  );
}
