import { PostMatchAnalysis } from "../PostMatchAnalysis";
import type { LeagueGameAssetView } from "../../state/AppStateProvider";
import type { PostMatchDetail } from "../../backend/types";
import { formatDuration, formatResult, formatTimestamp, type T } from "../../utils/formatting";
import { CloseIcon } from "./Icons";

export function MatchDetailPanel({
  detail,
  gameAssets,
  hasError,
  isLoading,
  onClose,
  onParticipantSelect,
  participantImages,
  t,
}: {
  detail: PostMatchDetail | undefined;
  gameAssets: Record<string, LeagueGameAssetView>;
  hasError: boolean;
  isLoading: boolean;
  onClose: () => void;
  onParticipantSelect: (participantId: number) => void;
  participantImages: Record<number, string>;
  t: T;
}) {
  return (
    <aside
      className="absolute left-2 top-12 z-20 flex max-h-[calc(100vh-3.5rem)] w-[46rem] max-w-[calc(100vw-1rem)] flex-col overflow-hidden rounded-xl border border-zinc-700 bg-zinc-900 shadow-xl"
      onClick={(event) => event.stopPropagation()}
    >
      {/* Header */}
      <div className="flex items-center gap-3 border-b border-zinc-700 bg-zinc-800 px-4 py-3">
        <div className="min-w-0 flex-1">
          <h2 className="truncate text-lg font-bold leading-tight text-zinc-100">
            {detail?.queueName ?? t("overlay.matchDetails")}
          </h2>
          <p className="mt-0.5 truncate text-xs text-zinc-400">
            {detail
              ? `${formatResult(detail.result, t)} · ${formatTimestamp(detail.playedAt, t)} · ${formatDuration(detail.gameDurationSeconds, t)}`
              : t("overlay.loadingMatchDetails")}
          </p>
        </div>
        <button
          aria-label={t("overlay.close")}
          className="flex h-8 w-8 shrink-0 items-center justify-center rounded-lg border border-zinc-700 bg-zinc-800 text-zinc-400 transition hover:border-zinc-600 hover:bg-zinc-700 hover:text-zinc-200"
          onClick={onClose}
          title={t("overlay.close")}
          type="button"
        >
          <CloseIcon />
        </button>
      </div>

      {/* Body */}
      <div className="min-h-0 flex-1 overflow-auto p-3">
        {isLoading && !detail && (
          <p className="rounded-lg border border-zinc-700 bg-zinc-800 px-4 py-3 text-center text-sm font-medium text-zinc-400">
            {t("overlay.loadingMatchDetails")}
          </p>
        )}
        {hasError && !detail && (
          <p className="rounded-lg border border-rose-700 bg-rose-950 px-4 py-3 text-sm font-medium text-rose-400">
            {t("overlay.matchDetailsUnavailable")}
          </p>
        )}
        {detail && (
          <PostMatchAnalysis
            detail={detail}
            gameAssets={gameAssets}
            onParticipantSelect={onParticipantSelect}
            participantImages={participantImages}
            teamsLayoutClassName=""
          />
        )}
      </div>
    </aside>
  );
}
