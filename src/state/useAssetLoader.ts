import { useCallback, useRef, type Dispatch, type SetStateAction } from "react";

import { fetchLeagueChampionDetails, fetchLeagueChampionIcon, fetchLeagueGameAsset, fetchLeagueProfileIcon, fetchRankTierIcon } from "../backend/leagueClient";
import type { LeagueChampionDetails, LeagueGameAssetKind, LeagueImageAsset } from "../backend/types";
import type { LeagueChampionDetailsView, LeagueGameAssetView, LeagueChampionAbilityView, LeagueImageUrls } from "./types";
import { imageAssetUrl } from "./utils";

const ASSET_LOAD_CONCURRENCY = 4;
const ASSET_LOAD_DELAY_MS = 16;

const inFlightRankIcons = new Map<string, Promise<void>>();

export function useAssetLoader(
  setLeagueImages: Dispatch<SetStateAction<LeagueImageUrls>>,
  setChampionDetailsById: Dispatch<SetStateAction<Record<number, LeagueChampionDetailsView>>>,
) {
  const imageUrlsRef = useRef<LeagueImageUrls>({ profileIcons: {}, championIcons: {}, gameAssets: {}, rankTierIcons: {} });
  const championDetailsRef = useRef<Record<number, LeagueChampionDetailsView>>({});
  const pendingImageKeysRef = useRef(new Set<string>());
  const assetQueueRef = useRef<Array<() => void>>([]);
  const activeAssetLoadsRef = useRef(0);
  const assetQueueTimerRef = useRef<number | null>(null);
  const imageFlushRef = useRef<number | null>(null);
  const drainAssetQueueRef = useRef<(() => void) | null>(null);

  const scheduleLeagueImagesUpdate = useCallback(() => {
    if (imageFlushRef.current !== null) {
      return;
    }

    imageFlushRef.current = window.requestAnimationFrame(() => {
      imageFlushRef.current = null;
      setLeagueImages(imageUrlsRef.current);
    });
  }, [setLeagueImages]);

  const scheduleAssetQueueDrain = useCallback(() => {
    if (assetQueueTimerRef.current !== null) {
      return;
    }

    assetQueueTimerRef.current = window.setTimeout(() => {
      assetQueueTimerRef.current = null;
      drainAssetQueueRef.current?.();
    }, ASSET_LOAD_DELAY_MS);
  }, []);

  const enqueueAssetLoad = useCallback(
    (task: () => Promise<boolean>) =>
      new Promise<boolean>((resolve) => {
        assetQueueRef.current.push(() => {
          activeAssetLoadsRef.current += 1;
          void task()
            .then(resolve)
            .catch(() => resolve(false))
            .finally(() => {
              activeAssetLoadsRef.current = Math.max(0, activeAssetLoadsRef.current - 1);
              scheduleAssetQueueDrain();
            });
        });

        scheduleAssetQueueDrain();
      }),
    [scheduleAssetQueueDrain],
  );

  drainAssetQueueRef.current = () => {
    while (activeAssetLoadsRef.current < ASSET_LOAD_CONCURRENCY) {
      const runNext = assetQueueRef.current.shift();
      if (!runNext) {
        return;
      }

      runNext();
    }
  };

  const loadLeagueProfileIconAction = useCallback(async (profileIconId: number | null | undefined) => {
    if (!profileIconId || imageUrlsRef.current.profileIcons[profileIconId]) {
      return true;
    }

    const key = `profile:${profileIconId}`;
    if (pendingImageKeysRef.current.has(key)) {
      return true;
    }

    pendingImageKeysRef.current.add(key);
    return enqueueAssetLoad(async () => {
      try {
        const asset = await fetchLeagueProfileIcon(profileIconId);
        const url = imageAssetUrl(asset);
        imageUrlsRef.current = {
          ...imageUrlsRef.current,
          profileIcons: {
            ...imageUrlsRef.current.profileIcons,
            [profileIconId]: url,
          },
        };
        scheduleLeagueImagesUpdate();
        return true;
      } catch {
        return false;
      } finally {
        pendingImageKeysRef.current.delete(key);
      }
    });
  }, [enqueueAssetLoad, scheduleLeagueImagesUpdate]);

  const loadLeagueChampionIconAction = useCallback(async (championId: number | null | undefined) => {
    if (!championId || imageUrlsRef.current.championIcons[championId]) {
      return true;
    }

    const key = `champion:${championId}`;
    if (pendingImageKeysRef.current.has(key)) {
      return true;
    }

    pendingImageKeysRef.current.add(key);
    return enqueueAssetLoad(async () => {
      try {
        const asset = await fetchLeagueChampionIcon(championId);
        const url = imageAssetUrl(asset);
        imageUrlsRef.current = {
          ...imageUrlsRef.current,
          championIcons: {
            ...imageUrlsRef.current.championIcons,
            [championId]: url,
          },
        };
        scheduleLeagueImagesUpdate();
        return true;
      } catch {
        return false;
      } finally {
        pendingImageKeysRef.current.delete(key);
      }
    });
  }, [enqueueAssetLoad, scheduleLeagueImagesUpdate]);

  const loadLeagueGameAssetAction = useCallback(async (kind: LeagueGameAssetKind, assetId: number | null | undefined) => {
    if (!assetId) {
      return true;
    }

    const key = leagueGameAssetKey(kind, assetId);
    if (imageUrlsRef.current.gameAssets[key] || pendingImageKeysRef.current.has(key)) {
      return true;
    }

    pendingImageKeysRef.current.add(key);
    return enqueueAssetLoad(async () => {
      try {
        const asset = await fetchLeagueGameAsset(kind, assetId);
        const resolvedUrl = imageAssetUrl(asset.image);
        imageUrlsRef.current = {
          ...imageUrlsRef.current,
          gameAssets: {
            ...imageUrlsRef.current.gameAssets,
            [key]: {
              kind: asset.kind,
              assetId: asset.assetId,
              name: asset.name,
              description: asset.description,
              imageUrl: resolvedUrl,
            },
          },
        };
        scheduleLeagueImagesUpdate();
        return true;
      } catch {
        return false;
      } finally {
        pendingImageKeysRef.current.delete(key);
      }
    });
  }, [enqueueAssetLoad, scheduleLeagueImagesUpdate]);

  const loadLeagueChampionDetailsAction = useCallback(async (championId: number | null | undefined) => {
    if (!championId || championDetailsRef.current[championId]) {
      return true;
    }

    const key = `champion-details:${championId}`;
    if (pendingImageKeysRef.current.has(key)) {
      return true;
    }

    pendingImageKeysRef.current.add(key);
    try {
      const details = await fetchLeagueChampionDetails(championId);
      const view = championDetailsView(details);
      championDetailsRef.current = {
        ...championDetailsRef.current,
        [championId]: view,
      };
      setChampionDetailsById(championDetailsRef.current);
      return true;
    } catch {
      return false;
    } finally {
      pendingImageKeysRef.current.delete(key);
    }
  }, [setChampionDetailsById]);

  const loadLeagueRankTierIconAction = useCallback((tier: string): Promise<void> => {
    const key = tier.toLowerCase();
    if (imageUrlsRef.current.rankTierIcons[key]) return Promise.resolve();
    const existing = inFlightRankIcons.get(key);
    if (existing) return existing;
    const promise = fetchRankTierIcon(key)
      .then((asset) => {
        const url = imageAssetUrl(asset);
        imageUrlsRef.current = {
          ...imageUrlsRef.current,
          rankTierIcons: { ...imageUrlsRef.current.rankTierIcons, [key]: url },
        };
        scheduleLeagueImagesUpdate();
      })
      .catch(() => { /* ignore — icon simply won't show */ })
      .finally(() => { inFlightRankIcons.delete(key); });
    inFlightRankIcons.set(key, promise);
    return promise;
  }, [scheduleLeagueImagesUpdate]);

  const cleanup = useCallback(() => {
    for (const url of Object.values(imageUrlsRef.current.profileIcons)) {
      URL.revokeObjectURL(url);
    }
    for (const url of Object.values(imageUrlsRef.current.championIcons)) {
      URL.revokeObjectURL(url);
    }
    for (const url of Object.values(imageUrlsRef.current.rankTierIcons)) {
      URL.revokeObjectURL(url);
    }
    for (const asset of Object.values(imageUrlsRef.current.gameAssets)) {
      URL.revokeObjectURL(asset.imageUrl);
    }
    for (const details of Object.values(championDetailsRef.current)) {
      if (details.squarePortraitUrl) {
        URL.revokeObjectURL(details.squarePortraitUrl);
      }
      for (const ability of details.abilities) {
        if (ability.iconUrl) {
          URL.revokeObjectURL(ability.iconUrl);
        }
      }
    }
    if (imageFlushRef.current !== null) {
      window.cancelAnimationFrame(imageFlushRef.current);
    }
    if (assetQueueTimerRef.current !== null) {
      window.clearTimeout(assetQueueTimerRef.current);
    }
    assetQueueRef.current = [];
  }, []);

  return {
    loadLeagueChampionDetails: loadLeagueChampionDetailsAction,
    loadLeagueChampionIcon: loadLeagueChampionIconAction,
    loadLeagueGameAsset: loadLeagueGameAssetAction,
    loadLeagueProfileIcon: loadLeagueProfileIconAction,
    loadLeagueRankTierIcon: loadLeagueRankTierIconAction,
    cleanup,
    imageUrlsRef,
  };
}

// Re-exported for use by AppStateProvider
export function leagueGameAssetKey(kind: LeagueGameAssetKind, assetId: number) {
  return `${kind}:${assetId}`;
}

// Internal helpers
function championDetailsView(details: LeagueChampionDetails): LeagueChampionDetailsView {
  return {
    championId: details.championId,
    championName: details.championName,
    title: details.title,
    squarePortraitUrl: details.squarePortrait ? imageAssetUrl(details.squarePortrait) : null,
    abilities: details.abilities.map((ability): LeagueChampionAbilityView => ({
      slot: ability.slot,
      name: ability.name,
      description: ability.description,
      summaryDescription: ability.summaryDescription,
      cooldown: ability.cooldown,
      cost: ability.cost,
      range: ability.range,
      cooldownValues: ability.cooldownValues,
      costValues: ability.costValues,
      rangeValues: ability.rangeValues,
      stats: ability.stats,
      iconUrl: ability.icon ? imageAssetUrl(ability.icon) : null,
    })),
  };
}
