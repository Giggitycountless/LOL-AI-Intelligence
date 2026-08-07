import { useCallback, useEffect, useState } from "react";

import { deletePlayerNote, listPlayerNotes, updatePlayerNote } from "../backend/leagueClient";
import type { PlayerNoteRecord } from "../backend/types";
import { useAppCore } from "../state/AppStateProvider";

const PLAYER_NOTES_LIMIT = 200;

export function PlayerNotes() {
  const { t } = useAppCore();
  const [notes, setNotes] = useState<PlayerNoteRecord[]>([]);
  const [search, setSearch] = useState("");
  const [isLoading, setIsLoading] = useState(true);

  const load = useCallback(async (query: string) => {
    setIsLoading(true);
    try {
      const response = await listPlayerNotes({
        limit: PLAYER_NOTES_LIMIT,
        search: query.trim() || null,
      });
      setNotes(response.records);
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    const timer = window.setTimeout(() => void load(search), 300);
    return () => window.clearTimeout(timer);
  }, [load, search]);

  async function handleSave(record: PlayerNoteRecord, note: string, tags: string[]) {
    const saved = await updatePlayerNote({
      playerPuuid: record.playerPuuid,
      lastDisplayName: record.displayName,
      note: note || null,
      tags,
    });
    setNotes((prev) => prev.map((entry) => (entry.playerPuuid === saved.playerPuuid ? saved : entry)));
  }

  async function handleDelete(record: PlayerNoteRecord) {
    await deletePlayerNote({ playerPuuid: record.playerPuuid });
    setNotes((prev) => prev.filter((entry) => entry.playerPuuid !== record.playerPuuid));
  }

  return (
    <main className="min-h-0 flex-1 overflow-auto px-8 py-7">
      <div className="mx-auto flex w-full max-w-4xl flex-col gap-6">
        <header className="flex flex-wrap items-end justify-between gap-4">
          <div>
            <p className="text-sm font-medium uppercase tracking-wide text-rose-700">{t("playerNotes.eyebrow")}</p>
            <h1 className="mt-2 text-3xl font-semibold text-zinc-950 dark:text-zinc-50">{t("playerNotes.title")}</h1>
          </div>
          <button
            type="button"
            onClick={() => void load(search)}
            disabled={isLoading}
            className="inline-flex h-10 items-center justify-center rounded-md border border-zinc-300 dark:border-zinc-600 bg-white dark:bg-zinc-900 px-4 text-sm font-semibold text-zinc-700 dark:text-zinc-300 shadow-sm transition hover:bg-zinc-50 dark:hover:bg-zinc-800 disabled:cursor-not-allowed disabled:text-zinc-400"
          >
            {isLoading ? t("common.refreshing") : t("common.refresh")}
          </button>
        </header>

        <div className="flex items-center justify-between gap-3">
          <input
            type="search"
            value={search}
            onChange={(event) => setSearch(event.target.value)}
            placeholder={t("playerNotes.searchPlaceholder")}
            className="h-10 w-full max-w-sm rounded-md border border-zinc-300 dark:border-zinc-600 bg-white dark:bg-zinc-900 px-3 text-sm text-zinc-950 dark:text-zinc-50 outline-none focus:border-rose-700 focus:ring-2 focus:ring-rose-100"
          />
          <p className="shrink-0 text-xs font-semibold uppercase tracking-wide text-zinc-500 dark:text-zinc-400">
            {notes.length} {t("playerNotes.shown")}
          </p>
        </div>

        <section className="rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-900 shadow-sm">
          {isLoading && notes.length === 0 && (
            <div className="px-5 py-12 text-center text-sm text-zinc-500 dark:text-zinc-400">{t("common.loading")}</div>
          )}

          {!isLoading && notes.length === 0 && (
            <div className="px-5 py-12 text-center">
              <p className="text-sm font-medium text-zinc-600 dark:text-zinc-400">
                {search ? t("playerNotes.noMatches") : t("playerNotes.noEntries")}
              </p>
              {!search && <p className="mt-1 text-sm text-zinc-500 dark:text-zinc-400">{t("playerNotes.emptyHint")}</p>}
            </div>
          )}

          {notes.length > 0 && (
            <div className="divide-y divide-zinc-200 dark:divide-zinc-700">
              {notes.map((record) => (
                <PlayerNoteCard
                  key={record.playerPuuid}
                  record={record}
                  onSave={(note, tags) => void handleSave(record, note, tags)}
                  onDelete={() => void handleDelete(record)}
                />
              ))}
            </div>
          )}
        </section>
      </div>
    </main>
  );
}

function PlayerNoteCard({
  record,
  onSave,
  onDelete,
}: {
  record: PlayerNoteRecord;
  onSave: (note: string, tags: string[]) => void;
  onDelete: () => void;
}) {
  const { t } = useAppCore();
  const [isEditing, setIsEditing] = useState(false);
  const [note, setNote] = useState(record.note ?? "");
  const [tagInput, setTagInput] = useState("");
  const [tags, setTags] = useState<string[]>(record.tags);

  function beginEdit() {
    setNote(record.note ?? "");
    setTags(record.tags);
    setTagInput("");
    setIsEditing(true);
  }

  function handleAddTag() {
    const trimmed = tagInput.trim();
    if (trimmed && !tags.includes(trimmed) && tags.length < 8) {
      setTags([...tags, trimmed]);
      setTagInput("");
    }
  }

  return (
    <article className="px-5 py-4">
      <div className="flex items-start justify-between gap-3">
        <div className="min-w-0">
          <p className="truncate text-sm font-semibold text-zinc-950 dark:text-zinc-50">{record.displayName}</p>
          <p className="text-xs text-zinc-500 dark:text-zinc-400">
            {t("playerNotes.updatedAt")} {record.updatedAt}
          </p>
        </div>
        {!isEditing && (
          <button type="button" onClick={beginEdit} className="shrink-0 text-xs font-medium text-rose-700 hover:underline">
            {t("common.edit")}
          </button>
        )}
      </div>

      {!isEditing && (
        <>
          {record.tags.length > 0 && (
            <div className="mt-2 flex flex-wrap gap-1">
              {record.tags.map((tag) => (
                <span key={tag} className="rounded bg-zinc-100 dark:bg-zinc-800 px-1.5 py-0.5 text-xs text-zinc-600 dark:text-zinc-400">
                  {tag}
                </span>
              ))}
            </div>
          )}
          {record.note && <p className="mt-2 text-sm text-zinc-600 dark:text-zinc-400">{record.note}</p>}
        </>
      )}

      {isEditing && (
        <div className="mt-3 flex flex-col gap-2">
          <textarea
            rows={2}
            value={note}
            onChange={(event) => setNote(event.target.value)}
            placeholder={t("participant.note")}
            className="w-full resize-none rounded-md border border-zinc-300 dark:border-zinc-600 bg-white dark:bg-zinc-800 px-3 py-2 text-sm text-zinc-950 dark:text-zinc-50 placeholder:text-zinc-400 outline-none focus:border-rose-700 focus:ring-2 focus:ring-rose-100"
          />
          <div className="flex flex-wrap gap-1">
            {tags.map((tag) => (
              <button
                key={tag}
                type="button"
                onClick={() => setTags(tags.filter((value) => value !== tag))}
                className="rounded bg-zinc-100 dark:bg-zinc-800 px-1.5 py-0.5 text-xs text-zinc-600 dark:text-zinc-400 hover:bg-red-50 hover:text-red-600"
              >
                {tag} ×
              </button>
            ))}
          </div>
          <div className="flex flex-wrap items-center gap-2">
            <input
              type="text"
              value={tagInput}
              onChange={(event) => setTagInput(event.target.value)}
              onKeyDown={(event) => event.key === "Enter" && (event.preventDefault(), handleAddTag())}
              placeholder={t("participant.tagsPlaceholder")}
              className="h-8 flex-1 rounded-md border border-zinc-300 dark:border-zinc-600 bg-white dark:bg-zinc-800 px-3 text-sm text-zinc-950 dark:text-zinc-50 placeholder:text-zinc-400 outline-none focus:border-rose-700"
            />
            <button
              type="button"
              onClick={() => {
                onSave(note, tags);
                setIsEditing(false);
              }}
              className="inline-flex h-8 items-center rounded-md bg-rose-700 px-3 text-xs font-semibold text-white transition hover:bg-rose-800"
            >
              {t("common.save")}
            </button>
            <button
              type="button"
              onClick={() => setIsEditing(false)}
              className="inline-flex h-8 items-center rounded-md border border-zinc-300 dark:border-zinc-600 px-3 text-xs font-semibold text-zinc-700 dark:text-zinc-300 transition hover:bg-zinc-50 dark:hover:bg-zinc-800"
            >
              {t("common.cancel")}
            </button>
            <button
              type="button"
              onClick={() => {
                onDelete();
                setIsEditing(false);
              }}
              className="inline-flex h-8 items-center rounded-md border border-red-300 dark:border-red-800 px-3 text-xs font-semibold text-red-700 dark:text-red-400 transition hover:bg-red-50 dark:hover:bg-red-950"
            >
              {t("common.delete")}
            </button>
          </div>
        </div>
      )}
    </article>
  );
}
