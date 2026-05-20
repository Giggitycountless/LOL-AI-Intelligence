import { memo } from "react";

import type { LeagueChampionAbilityView, LeagueChampionDetailsView } from "../../state/AppStateProvider";
import { initials, type T } from "../../utils/formatting";
import {
  abilityStatDisplay,
  abilityStatText,
  abilityTooltipText,
} from "../../pages/selfHistoryOverlayAbilityView";
import { CloseIcon } from "./Icons";

export function ChampionDetailsPanel({
  details,
  hasError,
  isLoading,
  onClose,
  t,
}: {
  details: LeagueChampionDetailsView | undefined;
  hasError: boolean;
  isLoading: boolean;
  onClose: () => void;
  t: T;
}) {
  return (
    <aside
      className="absolute right-2 top-12 z-20 flex max-h-[calc(100vh-3.5rem)] w-[42rem] max-w-[calc(100vw-1rem)] flex-col overflow-hidden rounded-xl border border-zinc-200 bg-zinc-50 shadow-xl"
      onClick={(event) => event.stopPropagation()}
    >
      {/* Header */}
      <div className="flex items-center gap-4 border-b border-zinc-200 bg-white px-4 py-3">
        {details?.squarePortraitUrl ? (
          <img
            alt=""
            className="h-14 w-14 shrink-0 rounded-lg border border-zinc-200 object-cover shadow-sm"
            src={details.squarePortraitUrl}
          />
        ) : (
          <div className="flex h-14 w-14 shrink-0 items-center justify-center rounded-lg border border-zinc-200 bg-zinc-100 text-base font-bold text-zinc-500">
            {details ? initials(details.championName) : "?"}
          </div>
        )}
        <div className="min-w-0 flex-1">
          <h2 className="truncate text-lg font-bold text-zinc-950 leading-tight">
            {details?.championName ?? t("overlay.abilities")}
          </h2>
          <p className="truncate text-xs text-zinc-400 mt-0.5">
            {details?.title ?? t("overlay.readingAbilities")}
          </p>
        </div>
        <button
          aria-label={t("overlay.close")}
          className="flex h-8 w-8 shrink-0 items-center justify-center rounded-lg border border-zinc-200 bg-white text-zinc-400 transition hover:border-zinc-300 hover:bg-zinc-50 hover:text-zinc-700"
          onClick={onClose}
          title={t("overlay.close")}
          type="button"
        >
          <CloseIcon />
        </button>
      </div>

      {/* Body */}
      <div className="min-h-0 flex-1 overflow-auto p-3">
        {isLoading && (
          <p className="rounded-lg bg-white px-4 py-3 text-sm font-medium text-zinc-400 text-center border border-zinc-200">
            {t("overlay.loadingAbilities")}
          </p>
        )}
        {hasError && !details && (
          <p className="rounded-lg border border-rose-200 bg-rose-50 px-4 py-3 text-sm font-medium text-rose-700">
            {t("overlay.abilitiesUnavailable")}
          </p>
        )}
        {details && (
          <div className="grid gap-2">
            {details.abilities.map((ability) => (
              <AbilityCard ability={ability} key={`${ability.slot}-${ability.name}`} t={t} />
            ))}
          </div>
        )}
      </div>
    </aside>
  );
}

// ── Slot accent colors ────────────────────────────────────────────────────────

const SLOT_BORDER: Record<string, string> = {
  Passive: "border-l-zinc-300",
  Q:       "border-l-sky-400",
  W:       "border-l-emerald-400",
  E:       "border-l-amber-400",
  R:       "border-l-rose-500",
};

const SLOT_BADGE: Record<string, string> = {
  Passive: "bg-zinc-100 text-zinc-500 border-zinc-200",
  Q:       "bg-sky-50 text-sky-700 border-sky-200",
  W:       "bg-emerald-50 text-emerald-700 border-emerald-200",
  E:       "bg-amber-50 text-amber-700 border-amber-200",
  R:       "bg-rose-50 text-rose-700 border-rose-200",
};

// ── Ability card ─────────────────────────────────────────────────────────────

const AbilityCard = memo(function AbilityCard({
  ability,
  t,
}: {
  ability: LeagueChampionAbilityView;
  t: T;
}) {
  const slot = ability.slot.toUpperCase();
  const borderColor = SLOT_BORDER[slot] ?? SLOT_BORDER.Q;
  const badgeColor = SLOT_BADGE[slot] ?? SLOT_BADGE.Q;

  const tooltip = abilityTooltipText(ability.summaryDescription, ability.description);
  const cooldown = abilityStatText(ability.cooldownValues, ability.cooldown);
  const cost = abilityStatText(ability.costValues, ability.cost);
  const range = abilityStatText(ability.rangeValues, ability.range);

  return (
    <section
      className={`flex gap-3 rounded-lg border border-zinc-200 border-l-4 bg-white px-3 py-3 shadow-sm transition hover:border-zinc-300 hover:shadow ${borderColor}`}
      title={tooltip || undefined}
    >
      {/* Icon */}
      <div className="shrink-0">
        {ability.iconUrl ? (
          <img
            alt=""
            className="h-12 w-12 rounded-lg border border-zinc-200 object-cover shadow-sm"
            src={ability.iconUrl}
          />
        ) : (
          <div className="flex h-12 w-12 items-center justify-center rounded-lg border border-zinc-200 bg-zinc-100 text-sm font-bold text-zinc-500">
            {slot}
          </div>
        )}
      </div>

      {/* Content */}
      <div className="min-w-0 flex-1">
        {/* Name row */}
        <div className="flex items-center gap-2">
          <span className={`inline-flex h-5 items-center rounded border px-1.5 text-[10px] font-bold ${badgeColor}`}>
            {slot}
          </span>
          <span className="truncate text-sm font-semibold text-zinc-900">
            {ability.name}
          </span>
        </div>

        {/* Stats row */}
        <div className="mt-2 flex flex-wrap gap-x-4 gap-y-1">
          {cooldown && <Stat label={t("overlay.cooldown")} value={cooldown} />}
          {cost && <Stat label={t("overlay.cost")} value={cost} />}
          {range && <Stat label={t("overlay.range")} value={range} />}
          {ability.stats?.map((stat, i) => (
            <Stat
              key={`${stat.label}-${i}`}
              label={stat.label}
              value={abilityStatDisplay(stat.values, stat.suffix)}
            />
          ))}
        </div>
      </div>
    </section>
  );
});

function Stat({ label, value }: { label: string; value: string }) {
  return (
    <p className="flex items-baseline gap-1 text-xs">
      <span className="text-zinc-400">{label}</span>
      <span className="font-semibold tabular-nums text-zinc-800">{value}</span>
    </p>
  );
}
