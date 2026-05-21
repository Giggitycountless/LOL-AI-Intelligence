import { useEffect, useState } from "react";

import { fetchLeagueSelfSnapshot, fetchPostMatchDetail, savePlayerNote, clearPlayerNote } from "../backend/leagueClient";
import type { PostMatchDetail, PostMatchParticipant, PlayerNoteView } from "../backend/types";
import { AppStateProvider } from "../state/AppStateProvider";
import { useAppCore } from "../state/AppStateProvider";
import type { TranslationKey } from "../i18n";

// Standalone window — wraps in its own AppStateProvider
export function PostGameNotesWindowRoot() {
  return (
    <AppStateProvider mode="main">
      <PostGameNotesWindow />
    </AppStateProvider>
  );
}

function PostGameNotesWindow() {
  const { t, effectiveLanguage } = useAppCore();
  const [matchDetail, setMatchDetail] = useState<PostMatchDetail | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [notes, setNotes] = useState<Record<number, PlayerNoteView>>({});

  useEffect(() => {
    void loadRecentMatch();
  }, []);

  async function loadRecentMatch() {
    setIsLoading(true);
    setError(null);
    try {
      // Get the most recent match from self data
      const selfData = await fetchLeagueSelfSnapshot({ matchLimit: 1 });
      const recentMatch = selfData.recentMatches[0];
      if (!recentMatch) {
        setError("No recent match found");
        return;
      }
      const detail = await fetchPostMatchDetail(recentMatch.gameId);
      setMatchDetail(detail);
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : "Failed to load match data");
    } finally {
      setIsLoading(false);
    }
  }

  async function handleSaveNote(participant: PostMatchParticipant, noteText: string, tags: string[]) {
    if (!matchDetail) return;
    try {
      const saved = await savePlayerNote({
        gameId: matchDetail.gameId,
        participantId: participant.participantId,
        note: noteText || null,
        tags,
      });
      setNotes((prev) => ({ ...prev, [participant.participantId]: saved }));
    } catch {
      // silent fail
    }
  }

  async function handleClearNote(participant: PostMatchParticipant) {
    if (!matchDetail) return;
    try {
      await clearPlayerNote({ gameId: matchDetail.gameId, participantId: participant.participantId });
      setNotes((prev) => {
        const next = { ...prev };
        delete next[participant.participantId];
        return next;
      });
    } catch {
      // silent fail
    }
  }

  const isDark = document.documentElement.classList.contains("dark");

  return (
    <div className={`${isDark ? "dark" : ""} flex h-screen flex-col bg-zinc-100 dark:bg-zinc-950 text-zinc-950 dark:text-zinc-50`}>
      <header className="flex h-12 shrink-0 items-center justify-between border-b border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-900 px-4">
        <p className="text-sm font-semibold text-zinc-950 dark:text-zinc-50">
          {t("postGame.title" as TranslationKey)}
        </p>
        <p className="text-xs text-zinc-500 dark:text-zinc-400">
          {matchDetail ? `${matchDetail.queueName ?? "Match"} · ${matchDetail.gameId}` : ""}
        </p>
      </header>

      <div className="min-h-0 flex-1 overflow-auto p-4">
        {isLoading && (
          <div className="flex h-32 items-center justify-center text-sm text-zinc-500 dark:text-zinc-400">
            {t("common.loading" as TranslationKey)}
          </div>
        )}

        {!isLoading && error && (
          <div className="flex flex-col items-center gap-3 py-8 text-sm text-zinc-500 dark:text-zinc-400">
            <p>{error}</p>
            <button
              type="button"
              onClick={() => void loadRecentMatch()}
              className="inline-flex h-8 items-center rounded-md border border-zinc-300 dark:border-zinc-600 bg-white dark:bg-zinc-800 px-3 text-sm font-medium text-zinc-700 dark:text-zinc-300 transition hover:bg-zinc-50 dark:hover:bg-zinc-700"
            >
              {t("common.refresh" as TranslationKey)}
            </button>
          </div>
        )}

        {!isLoading && matchDetail && (
          <div className="flex flex-col gap-3">
            {matchDetail.teams.flatMap((team) =>
              team.participants.map((participant) => (
                <ParticipantNoteCard
                  key={participant.participantId}
                  participant={participant}
                  savedNote={participant.noteSummary}
                  onSave={(note, tags) => void handleSaveNote(participant, note, tags)}
                  onClear={() => void handleClearNote(participant)}
                  t={t}
                />
              ))
            )}
          </div>
        )}
      </div>
    </div>
  );
}

function ParticipantNoteCard({
  participant,
  savedNote,
  onSave,
  onClear,
  t,
}: {
  participant: PostMatchParticipant;
  savedNote: { hasNote: boolean; tags: string[] };
  onSave: (note: string, tags: string[]) => void;
  onClear: () => void;
  t: (key: TranslationKey) => string;
}) {
  const [note, setNote] = useState("");
  const [tagInput, setTagInput] = useState("");
  const [tags, setTags] = useState<string[]>(savedNote.tags);
  const [isExpanded, setIsExpanded] = useState(false);
  const hasContent = savedNote.hasNote || tags.length > 0;

  function handleAddTag() {
    const trimmed = tagInput.trim();
    if (trimmed && !tags.includes(trimmed) && tags.length < 5) {
      setTags([...tags, trimmed]);
      setTagInput("");
    }
  }

  function handleRemoveTag(tag: string) {
    setTags(tags.filter((t) => t !== tag));
  }

  return (
    <div className="rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-900 p-3 shadow-sm">
      <div className="flex items-center justify-between gap-2">
        <div className="flex min-w-0 items-center gap-2">
          <span
            className={[
              "inline-flex h-5 w-5 shrink-0 items-center justify-center rounded text-xs font-bold",
              participant.result === "win"
                ? "bg-emerald-100 text-emerald-700"
                : "bg-red-100 text-red-700",
            ].join(" ")}
          >
            {participant.result === "win" ? "W" : "L"}
          </span>
          <p className="truncate text-sm font-semibold text-zinc-950 dark:text-zinc-50">
            {participant.displayName}
          </p>
          <p className="shrink-0 text-xs text-zinc-500 dark:text-zinc-400">
            {participant.championName}
          </p>
          <p className="shrink-0 text-xs text-zinc-500 dark:text-zinc-400">
            {participant.kills}/{participant.deaths}/{participant.assists}
          </p>
        </div>
        <button
          type="button"
          onClick={() => setIsExpanded((v) => !v)}
          className={[
            "shrink-0 text-xs font-medium",
            isExpanded ? "text-rose-700" : hasContent ? "text-emerald-700" : "text-zinc-400 dark:text-zinc-500",
          ].join(" ")}
        >
          {isExpanded ? t("common.save" as TranslationKey) : hasContent ? "★ Note" : "+ Note"}
        </button>
      </div>

      {tags.length > 0 && !isExpanded && (
        <div className="mt-2 flex flex-wrap gap-1">
          {tags.map((tag) => (
            <span key={tag} className="rounded bg-zinc-100 dark:bg-zinc-800 px-1.5 py-0.5 text-xs text-zinc-600 dark:text-zinc-400">
              {tag}
            </span>
          ))}
        </div>
      )}

      {isExpanded && (
        <div className="mt-3 flex flex-col gap-2">
          <textarea
            rows={2}
            value={note}
            onChange={(e) => setNote(e.target.value)}
            placeholder={t("participant.note" as TranslationKey)}
            className="w-full resize-none rounded-md border border-zinc-300 dark:border-zinc-600 bg-white dark:bg-zinc-800 px-3 py-2 text-sm text-zinc-950 dark:text-zinc-50 placeholder:text-zinc-400 outline-none focus:border-rose-700 focus:ring-2 focus:ring-rose-100"
          />
          <div className="flex flex-wrap gap-1">
            {tags.map((tag) => (
              <button
                key={tag}
                type="button"
                onClick={() => handleRemoveTag(tag)}
                className="rounded bg-zinc-100 dark:bg-zinc-800 px-1.5 py-0.5 text-xs text-zinc-600 dark:text-zinc-400 hover:bg-red-50 hover:text-red-600"
              >
                {tag} ×
              </button>
            ))}
          </div>
          <div className="flex gap-2">
            <input
              type="text"
              value={tagInput}
              onChange={(e) => setTagInput(e.target.value)}
              onKeyDown={(e) => e.key === "Enter" && handleAddTag()}
              placeholder={t("participant.tags" as TranslationKey)}
              className="flex-1 rounded-md border border-zinc-300 dark:border-zinc-600 bg-white dark:bg-zinc-800 px-3 py-1.5 text-sm text-zinc-950 dark:text-zinc-50 placeholder:text-zinc-400 outline-none focus:border-rose-700"
            />
            <button
              type="button"
              onClick={() => { onSave(note, tags); setIsExpanded(false); }}
              className="inline-flex h-8 items-center rounded-md bg-rose-700 px-3 text-xs font-semibold text-white transition hover:bg-rose-800"
            >
              {t("common.save" as TranslationKey)}
            </button>
            {hasContent && (
              <button
                type="button"
                onClick={() => { onClear(); setTags([]); setNote(""); setIsExpanded(false); }}
                className="inline-flex h-8 items-center rounded-md border border-zinc-300 dark:border-zinc-600 px-3 text-xs font-semibold text-zinc-700 dark:text-zinc-300 transition hover:bg-zinc-50 dark:hover:bg-zinc-800"
              >
                {t("common.clear" as TranslationKey)}
              </button>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
