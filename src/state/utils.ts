import { isCommandError } from "../backend/commands";
import type { ChampSelectSnapshot, LeagueImageAsset } from "../backend/types";

export function errorMessage(error: unknown) {
  if (isCommandError(error)) {
    return error.message;
  }

  return error instanceof Error ? error.message : "Unexpected error";
}

export function imageAssetUrl(asset: LeagueImageAsset) {
  const binary = atob(asset.bytes);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i += 1) {
    bytes[i] = binary.charCodeAt(i);
  }
  return URL.createObjectURL(new Blob([bytes], { type: asset.mimeType }));
}

export function participantProfileKey(gameId: number, participantId: number) {
  return `${gameId}:${participantId}`;
}

// Computes a fingerprint from player identities + recentMatchIds + rank/mastery for
// change detection. Rank and mastery must participate: background hydration can push a
// snapshot whose only difference is freshly fetched ranked queues, and it would
// otherwise be dropped as a duplicate.
// NOTE: The Rust-side champ_select_fingerprint omits these fields (it uses puuid for
// cache identity instead). These serve different purposes and are intentionally divergent.
export function champSelectFingerprint(snapshot: ChampSelectSnapshot) {
  return snapshot.players
    .map((player) => {
      const recentMatchIds = player.recentStats?.recentMatches.map((match) => match.gameId).join(",") ?? "";
      const rankSignature = player.rankedQueues
        .map((queue) => `${queue.queue}${queue.tier ?? ""}${queue.division ?? ""}${queue.leaguePoints ?? ""}`)
        .join(",");
      return [
        player.summonerId,
        player.displayName,
        player.championId ?? "",
        player.team,
        recentMatchIds,
        rankSignature,
        player.masteryLevel ?? "",
        player.summonerLevel ?? "",
      ].join(":");
    })
    .join("|");
}
