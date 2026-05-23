import { useCallback, useEffect, useState } from "react";

import { deleteChatPreset, fetchChatPresets, saveChatPreset } from "../backend/leagueClient";
import { isCommandError } from "../backend/commands";
import type { ChatPreset } from "../backend/types";

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
      const msg = isCommandError(err) ? err.message : err instanceof Error ? err.message : "加载预设失败";
      setLoadError(msg);
    } finally {
      setLoading(false);
    }
  }, []);

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
      updateSlot(slot, { error: "标签和消息不能为空" });
      return;
    }
    updateSlot(slot, { saving: true, error: null });
    try {
      const saved = await saveChatPreset(slot, label, message);
      setSlots((prev) => prev.map((s) => (s.slot === slot ? slotFromPreset(saved) : s)));
    } catch (err: unknown) {
      const msg = isCommandError(err) ? err.message : err instanceof Error ? err.message : "保存失败";
      updateSlot(slot, { saving: false, error: msg });
    }
  }, [slots, updateSlot]);

  const handleDelete = useCallback(async (slot: number) => {
    updateSlot(slot, { saving: true, error: null });
    try {
      await deleteChatPreset(slot);
      setSlots((prev) => prev.map((s) => (s.slot === slot ? emptySlot(slot) : s)));
    } catch (err: unknown) {
      const msg = isCommandError(err) ? err.message : err instanceof Error ? err.message : "删除失败";
      updateSlot(slot, { saving: false, error: msg });
    }
  }, [updateSlot]);

  return (
    <main className="min-h-0 flex-1 overflow-auto px-8 py-7">
      <div className="mx-auto flex w-full max-w-4xl flex-col gap-6">
        <header>
          <p className="text-sm font-medium uppercase tracking-wide text-rose-700">快捷喊话</p>
          <h1 className="mt-2 text-3xl font-semibold text-zinc-950 dark:text-zinc-50">聊天预设</h1>
          <p className="mt-1 text-sm text-zinc-500 dark:text-zinc-400">
            最多 9 条预设，分别绑定 <code className="rounded bg-zinc-200 dark:bg-zinc-700 px-1 text-xs">Ctrl+Shift+1</code> 到 <code className="rounded bg-zinc-200 dark:bg-zinc-700 px-1 text-xs">Ctrl+Shift+9</code>。游戏内按热键自动打开聊天框并键入消息，<strong>由你自己按 Enter 发送</strong>（或 Esc 取消、Shift+Enter 改成全聊）。
          </p>
          <p className="mt-2 text-xs text-amber-700 dark:text-amber-400">
            ⚠️ 仅在 League 游戏窗口前台时触发。请勿用于刷屏或骚扰队友。
          </p>
        </header>

        {loadError && (
          <div className="rounded-lg border border-rose-200 dark:border-rose-800 bg-rose-50 dark:bg-rose-950 px-4 py-3 text-sm font-medium text-rose-800 dark:text-rose-300">
            {loadError}
          </div>
        )}

        {loading ? (
          <div className="text-sm text-zinc-500 dark:text-zinc-400">加载中…</div>
        ) : (
          <div className="grid gap-3">
            {slots.map((s) => (
              <SlotCard
                key={s.slot}
                state={s}
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

function SlotCard({
  state,
  onChangeLabel,
  onChangeMessage,
  onSave,
  onDelete,
}: {
  state: SlotState;
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
            placeholder={`标签，例如「上路 miss」`}
            value={state.label}
            onChange={(e) => onChangeLabel(e.target.value)}
            maxLength={40}
            className="h-9 w-full rounded-md border border-zinc-300 dark:border-zinc-600 bg-white dark:bg-zinc-900 px-3 text-sm text-zinc-950 dark:text-zinc-50 outline-none focus:border-rose-700 focus:ring-2 focus:ring-rose-100 dark:focus:ring-rose-900"
          />
          <input
            type="text"
            placeholder="消息内容（按热键时自动发到队聊；加 /all 前缀发全频道）"
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
            {state.saving ? "保存中…" : "保存"}
          </button>
          {state.hasSaved && (
            <button
              type="button"
              disabled={state.saving}
              onClick={onDelete}
              className="inline-flex h-9 items-center justify-center rounded-md border border-zinc-300 dark:border-zinc-600 px-3 text-xs font-semibold text-zinc-700 dark:text-zinc-300 transition hover:bg-zinc-50 dark:hover:bg-zinc-800 disabled:cursor-not-allowed"
            >
              清空
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
