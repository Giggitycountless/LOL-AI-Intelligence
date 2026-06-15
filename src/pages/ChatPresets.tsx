import { useCallback, useEffect, useState } from "react";

import { deleteChatPreset, fetchChatPresets, saveChatPreset } from "../backend/leagueClient";
import { isCommandError } from "../backend/commands";
import type { ChatPreset } from "../backend/types";
import { useAppCore } from "../state/AppStateProvider";
import type { T } from "../utils/formatting";

type SlotState = {
  slot: number;
  label: string;
  message: string;
  hasSaved: boolean;
  dirty: boolean;
  saving: boolean;
  error: string | null;
};

function emptySlot(slot: number): SlotState {
  return {
    slot,
    label: "",
    message: "",
    hasSaved: false,
    dirty: false,
    saving: false,
    error: null,
  };
}

function slotFromPreset(preset: ChatPreset): SlotState {
  return {
    slot: preset.slot,
    label: preset.label,
    message: preset.message,
    hasSaved: true,
    dirty: false,
    saving: false,
    error: null,
  };
}

export function ChatPresets() {
  const { t } = useAppCore();
  const [slots, setSlots] = useState<SlotState[]>(
    Array.from({ length: 9 }, (_, i) => emptySlot(i + 1)),
  );
  const [loading, setLoading] = useState(true);
  const [loadError, setLoadError] = useState<string | null>(null);

  const reload = useCallback(async () => {
    setLoading(true);
    setLoadError(null);
    try {
      const presets = await fetchChatPresets();
      const bySlot = new Map(presets.map((p) => [p.slot, p]));
      setSlots(
        Array.from({ length: 9 }, (_, i) => {
          const slot = i + 1;
          const preset = bySlot.get(slot);
          return preset ? slotFromPreset(preset) : emptySlot(slot);
        }),
      );
    } catch (err: unknown) {
      const msg = isCommandError(err) ? err.message : err instanceof Error ? err.message : t("chatPresets.loadError");
      setLoadError(msg);
    } finally {
      setLoading(false);
    }
  }, [t]);

  useEffect(() => { void reload(); }, [reload]);

  const updateSlot = useCallback((slot: number, patch: Partial<SlotState>) => {
    setSlots((prev) => prev.map((s) => (s.slot === slot ? { ...s, ...patch } : s)));
  }, []);

  const handleSave = useCallback(async (slot: number) => {
    const current = slots.find((s) => s.slot === slot);
    if (!current) return;
    const label = current.label.trim();
    const message = current.message.trim();
    if (!label || !message) {
      updateSlot(slot, { error: t("chatPresets.emptyError") });
      return;
    }
    updateSlot(slot, { saving: true, error: null });
    try {
      const saved = await saveChatPreset(slot, label, message);
      setSlots((prev) => prev.map((s) => (s.slot === slot ? slotFromPreset(saved) : s)));
    } catch (err: unknown) {
      const msg = isCommandError(err) ? err.message : err instanceof Error ? err.message : t("chatPresets.saveFailed");
      updateSlot(slot, { saving: false, error: msg });
    }
  }, [slots, updateSlot, t]);

  const handleDelete = useCallback(async (slot: number) => {
    updateSlot(slot, { saving: true, error: null });
    try {
      await deleteChatPreset(slot);
      setSlots((prev) => prev.map((s) => (s.slot === slot ? emptySlot(slot) : s)));
    } catch (err: unknown) {
      const msg = isCommandError(err) ? err.message : err instanceof Error ? err.message : t("chatPresets.deleteFailed");
      updateSlot(slot, { saving: false, error: msg });
    }
  }, [updateSlot, t]);

  return (
    <main className="min-h-0 flex-1 overflow-auto px-8 py-7">
      <div className="mx-auto flex w-full max-w-4xl flex-col gap-6">
        <header>
          <p className="text-sm font-medium uppercase tracking-wide text-rose-700">{t("chatPresets.eyebrow")}</p>
          <h1 className="mt-2 text-3xl font-semibold text-zinc-950 dark:text-zinc-50">{t("chatPresets.title")}</h1>
          <p className="mt-1 text-sm text-zinc-500 dark:text-zinc-400">
            {t("chatPresets.subtitlePrefix")}
            <code className="rounded bg-zinc-200 dark:bg-zinc-700 px-1 text-xs">Ctrl+Shift+1</code>
            {t("chatPresets.subtitleTo")}
            <code className="rounded bg-zinc-200 dark:bg-zinc-700 px-1 text-xs">Ctrl+Shift+9</code>
            {t("chatPresets.subtitleMid")}
            <strong>{t("chatPresets.subtitleSendBold")}</strong>
            {t("chatPresets.subtitleSendRest")}
          </p>
          <p className="mt-2 text-xs text-amber-700 dark:text-amber-400">
            {t("chatPresets.warning")}
          </p>
        </header>

        <div className="flex items-start gap-2 rounded-lg border border-amber-200 dark:border-amber-800 bg-amber-50 dark:bg-amber-950 px-4 py-3 text-sm font-medium text-amber-800 dark:text-amber-300">
          <ShieldIcon />
          <span>{t("chatPresets.adminNote")}</span>
        </div>

        {loadError && (
          <div className="rounded-lg border border-rose-200 dark:border-rose-800 bg-rose-50 dark:bg-rose-950 px-4 py-3 text-sm font-medium text-rose-800 dark:text-rose-300">
            {loadError}
          </div>
        )}

        {loading ? (
          <div className="text-sm text-zinc-500 dark:text-zinc-400">{t("common.loading")}</div>
        ) : (
          <div className="grid gap-3">
            {slots.map((s) => (
              <SlotCard
                key={s.slot}
                state={s}
                t={t}
                onChangeLabel={(v) => updateSlot(s.slot, { label: v, dirty: true, error: null })}
                onChangeMessage={(v) => updateSlot(s.slot, { message: v, dirty: true, error: null })}
                onSave={() => void handleSave(s.slot)}
                onDelete={() => void handleDelete(s.slot)}
              />
            ))}
          </div>
        )}
      </div>
    </main>
  );
}

function ShieldIcon() {
  return (
    <svg aria-hidden="true" className="mt-0.5 h-4 w-4 shrink-0" viewBox="0 0 24 24" fill="currentColor">
      <path d="M12 2 4 5v6c0 5 3.4 8.4 8 9 4.6-.6 8-4 8-9V5l-8-3Zm-1 13-3.5-3.5 1.4-1.4L11 12.2l4.1-4.1 1.4 1.4L11 15Z" />
    </svg>
  );
}

function SlotCard({
  state,
  t,
  onChangeLabel,
  onChangeMessage,
  onSave,
  onDelete,
}: {
  state: SlotState;
  t: T;
  onChangeLabel: (v: string) => void;
  onChangeMessage: (v: string) => void;
  onSave: () => void;
  onDelete: () => void;
}) {
  const hasContent = state.label.trim().length > 0 || state.message.trim().length > 0;

  return (
    <div className="rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-900 p-4 shadow-sm">
      <div className="flex flex-wrap items-start gap-3">
        <div className="flex h-10 w-24 shrink-0 items-center justify-center rounded-md bg-zinc-100 dark:bg-zinc-800 px-2 text-xs font-mono font-semibold text-zinc-700 dark:text-zinc-300">
          Ctrl+Shift+{state.slot}
        </div>

        <div className="flex min-w-0 flex-1 flex-col gap-2">
          <input
            type="text"
            placeholder={t("chatPresets.labelPlaceholder")}
            value={state.label}
            onChange={(e) => onChangeLabel(e.target.value)}
            maxLength={40}
            className="h-9 w-full rounded-md border border-zinc-300 dark:border-zinc-600 bg-white dark:bg-zinc-900 px-3 text-sm text-zinc-950 dark:text-zinc-50 outline-none focus:border-rose-700 focus:ring-2 focus:ring-rose-100 dark:focus:ring-rose-900"
          />
          <input
            type="text"
            placeholder={t("chatPresets.messagePlaceholder")}
            value={state.message}
            onChange={(e) => onChangeMessage(e.target.value)}
            maxLength={200}
            className="h-9 w-full rounded-md border border-zinc-300 dark:border-zinc-600 bg-white dark:bg-zinc-900 px-3 text-sm text-zinc-950 dark:text-zinc-50 outline-none focus:border-rose-700 focus:ring-2 focus:ring-rose-100 dark:focus:ring-rose-900"
          />
          {state.error && (
            <p className="text-xs font-medium text-rose-700 dark:text-rose-400">{state.error}</p>
          )}
        </div>

        <div className="flex shrink-0 flex-col gap-2">
          <button
            type="button"
            disabled={state.saving || !state.dirty || !hasContent}
            onClick={onSave}
            className="inline-flex h-9 items-center justify-center rounded-md bg-rose-700 px-3 text-xs font-semibold text-white transition hover:bg-rose-800 disabled:cursor-not-allowed disabled:bg-zinc-300 dark:disabled:bg-zinc-700"
          >
            {state.saving ? t("common.saving") : t("common.save")}
          </button>
          {state.hasSaved && (
            <button
              type="button"
              disabled={state.saving}
              onClick={onDelete}
              className="inline-flex h-9 items-center justify-center rounded-md border border-zinc-300 dark:border-zinc-600 px-3 text-xs font-semibold text-zinc-700 dark:text-zinc-300 transition hover:bg-zinc-50 dark:hover:bg-zinc-800 disabled:cursor-not-allowed"
            >
              {t("chatPresets.clear")}
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
