import type { PlaystyleProfile, PlaystyleTag, PlaystyleTone } from "../backend/types";
import type { TranslationKey } from "../i18n";

type T = (key: TranslationKey) => string;

function toneClass(tone: PlaystyleTone): string {
  switch (tone) {
    case "good":
      return "border-emerald-200 bg-emerald-50 text-emerald-800 dark:border-emerald-900 dark:bg-emerald-950 dark:text-emerald-300";
    case "warn":
      return "border-amber-200 bg-amber-50 text-amber-800 dark:border-amber-900 dark:bg-amber-950 dark:text-amber-300";
    case "info":
      return "border-zinc-200 bg-zinc-50 text-zinc-700 dark:border-zinc-700 dark:bg-zinc-800 dark:text-zinc-300";
  }
}

function withValue(text: string, value: string | null): string {
  return value ? text.replace("{n}", value) : text;
}

function tagLabel(tag: PlaystyleTag, t: T): string {
  return withValue(t(`playstyleTag.${tag.kind}` as TranslationKey), tag.value);
}

function tagDescription(tag: PlaystyleTag, t: T): string {
  return withValue(t(`playstyleTag.${tag.kind}.desc` as TranslationKey), tag.value);
}

export function PlaystyleTags({ profile, t }: { profile: PlaystyleProfile | null; t: T }) {
  if (!profile || profile.tags.length === 0) {
    return <p className="text-sm text-zinc-500 dark:text-zinc-400">{t("profile.playstyle.empty")}</p>;
  }

  return (
    <div className="flex flex-wrap gap-2">
      {profile.tags.map((tag) => (
        <span
          className={["rounded-md border px-2.5 py-1 text-sm font-semibold", toneClass(tag.tone)].join(" ")}
          key={tag.kind}
          title={tagDescription(tag, t)}
        >
          {tagLabel(tag, t)}
        </span>
      ))}
    </div>
  );
}
