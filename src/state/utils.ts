import { isCommandError } from "../backend/commands";
import type { ChampSelectSnapshot, LeagueImageAsset } from "../backend/types";

export function errorMessage(error: unknown) {
  if (isCommandError(error)) {
    return error.message;
  }

  return error instanceof Error ? error.message : "Unexpected error";
}

export function imageAssetUrl(asset: LeagueImageAsset) {
  return URL.createObjectURL(new Blob([Uint8Array.from(asset.bytes)], { type: asset.mimeType }));
}

export function participantProfileKey(gameId: number, participantId: number) {
  return `${gameId}:${participantId}`;
}

// Computes a fingerprint from player identities + recentMatchIds for change detection.
// NOTE: The Rust-side champ_select_fingerprint omits recentMatchIds (it uses puuid for
// cache identity instead). These serve different purposes and are intentionally divergent.
export function champSelectFingerprint(snapshot: ChampSelectSnapshot) {
  return snapshot.players
    .map((player) => {
      const recentMatchIds = player.recentStats?.recentMatches.map((match) => match.gameId).join(",") ?? "";
      return [
        player.summonerId,
        player.displayName,
        player.championId ?? "",
        player.team,
        recentMatchIds,
      ].join(":");
    })
    .join("|");
}
