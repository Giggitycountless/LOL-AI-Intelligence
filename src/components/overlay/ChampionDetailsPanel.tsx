import { memo } from "react";

import type { LeagueChampionAbilityView, LeagueChampionDetailsView } from "../../state/AppStateProvider";
import { initials, type T } from "../../utils/formatting";
import {
  abilityStatDisplay,
  abilityStatText,
  abilityTooltipText,
  parseAbilityDescription,
  type DescriptionSection,
  type SpanKind,
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
      className="absolute right-2 top-12 z-20 flex max-h-[calc(100vh-3.5rem)] w-[38rem] max-w-[calc(100vw-1rem)] flex-col overflow-hidden rounded-lg border border-zinc-200 bg-white shadow-lg"
      onClick={(event) => event.stopPropagation()}
    >
      <div className="flex items-center gap-3 border-b border-zinc-200 bg-zinc-50 px-3 py-3">
        {details?.squarePortraitUrl ? (
          <img alt="" className="h-12 w-12 rounded-md border border-zinc-200 object-cover" src={details.squarePortraitUrl} />
        ) : (
          <div className="flex h-12 w-12 items-center justify-center rounded-md border border-zinc-200 bg-zinc-100 text-sm font-semibold text-zinc-500">
            {details ? initials(details.championName) : "?"}
          </div>
        )}
        <div className="min-w-0 flex-1">
          <h2 className="truncate text-base font-semibold text-zinc-950">{details?.championName ?? t("overlay.abilities")}</h2>
          <p className="truncate text-xs font-medium text-zinc-500">{details?.title ?? t("overlay.readingAbilities")}</p>
        </div>
        <button
          aria-label={t("overlay.close")}
          className="flex h-8 w-8 items-center justify-center rounded-md border border-zinc-300 bg-white text-zinc-500 transition hover:border-zinc-400 hover:bg-zinc-50 hover:text-zinc-950"
          onClick={onClose}
          title={t("overlay.close")}
          type="button"
        >
          <CloseIcon />
        </button>
      </div>

      <div className="min-h-0 flex-1 overflow-auto p-3">
        {isLoading && <p className="rounded-md bg-zinc-100 px-3 py-2 text-sm font-medium text-zinc-500">{t("overlay.loadingAbilities")}</p>}
        {hasError && !details && (
          <p className="rounded-md border border-rose-200 bg-rose-50 px-3 py-2 text-sm font-medium text-rose-700">
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

// ── Span styling ─────────────────────────────────────────────────────────────

const SPAN_CLASS: Record<SpanKind, string> = {
  text: "text-zinc-600",
  magic: "text-blue-600",
  physical: "text-orange-600",
  true: "text-amber-600",
  healing: "text-emerald-600",
  status: "text-rose-700",
  bold: "font-semibold text-zinc-900",
  unresolved: "font-mono text-red-600 underline decoration-dotted",
};

const LABEL_CLASS: Record<NonNullable<DescriptionSection["label"]>, string> = {
  Passive: "bg-zinc-100 text-zinc-500 border border-zinc-200",
  Active: "bg-rose-50 text-rose-700 border border-rose-200",
};

function StructuredDescription({ sections }: { sections: DescriptionSection[] }) {
  if (sections.length === 0) {
    return <p className="mt-2 text-xs text-zinc-400">-</p>;
  }

  return (
    <div className="mt-2 grid gap-1.5">
      {sections.map((section, i) => (
        <p key={i} className="text-xs leading-relaxed">
          {section.label && (
            <span className={`mr-1.5 inline-block rounded px-1.5 py-0.5 text-[10px] font-semibold ${LABEL_CLASS[section.label]}`}>
              {section.label}
            </span>
          )}
          {section.lines.map((spans, j) => (
            <span key={j}>
              {spans.map((span, k) => (
                <span key={k} className={SPAN_CLASS[span.kind]}>
                  {span.text}
                </span>
              ))}
              {j < section.lines.length - 1 && <br />}
            </span>
          ))}
        </p>
      ))}
    </div>
  );
}

// ── Ability card ─────────────────────────────────────────────────────────────

const AbilityCard = memo(function AbilityCard({ ability, t }: { ability: LeagueChampionAbilityView; t: T }) {
  const plainDescription = abilityTooltipText(ability.summaryDescription, ability.description);
  const sections = parseAbilityDescription(ability.description);
  const cooldown = abilityStatText(ability.cooldownValues, ability.cooldown);
  const cost = abilityStatText(ability.costValues, ability.cost);
  const range = abilityStatText(ability.rangeValues, ability.range);

  return (
    <section className="rounded-lg border border-zinc-200 bg-white p-3 shadow-sm">
      <div className="grid grid-cols-[4.75rem_minmax(0,1fr)] gap-3">
        {ability.iconUrl ? (
          <img alt="" className="h-12 w-12 rounded-md border border-zinc-200 object-cover shadow-sm" src={ability.iconUrl} />
        ) : (
          <div className="flex h-12 w-12 items-center justify-center rounded-md border border-zinc-200 bg-zinc-100 text-sm font-semibold text-zinc-500 shadow-sm">
            {ability.slot}
          </div>
        )}
        <div className="min-w-0">
          <div className="flex min-w-0 items-center gap-2">
            <span className="flex h-6 min-w-11 shrink-0 items-center justify-center rounded-md border border-rose-200 bg-rose-50 px-2 text-xs font-semibold text-rose-700">
              {ability.slot}
            </span>
            <h3 className="truncate text-sm font-semibold text-zinc-950" title={plainDescription}>
              {ability.name}
            </h3>
          </div>

          <StructuredDescription sections={sections} />

          <div className="mt-2 rounded-md bg-zinc-100 px-3 py-2 text-xs font-semibold leading-relaxed text-zinc-800">
            {ability.stats?.map((stat, i) => (
              <AbilityStatRow
                key={`${stat.label}-${i}`}
                label={stat.label}
                value={abilityStatDisplay(stat.values, stat.suffix)}
              />
            ))}
            <AbilityStatRow label={t("overlay.cooldown")} value={cooldown} />
            <AbilityStatRow label={t("overlay.cost")} value={cost} />
            <AbilityStatRow label={t("overlay.range")} value={range} />
          </div>
        </div>
      </div>
    </section>
  );
});

function AbilityStatRow({ label, value }: { label: string; value: string }) {
  return (
    <p className="grid grid-cols-[4.5rem_minmax(0,1fr)] gap-2">
      <span className="text-zinc-500">{label}:</span>
      <span className="min-w-0 truncate tabular-nums">{value}</span>
    </p>
  );
}
