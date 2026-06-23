import { useAppCore } from "../../state/AppStateProvider";
import type { MatchResult } from "../../backend/types";
import { formatResult } from "../../utils/formatting";

export function ResultBadge({ result }: { result: MatchResult }) {
  const { t } = useAppCore();
  const tone =
    result === "win"
      ? "border-emerald-300 bg-emerald-100 text-emerald-800 dark:border-emerald-700 dark:bg-emerald-950 dark:text-emerald-300"
      : result === "loss"
        ? "border-rose-300 bg-rose-100 text-rose-800 dark:border-rose-700 dark:bg-rose-950 dark:text-rose-300"
        : "border-zinc-300 bg-zinc-100 text-zinc-600 dark:border-zinc-600 dark:bg-zinc-800 dark:text-zinc-300";

  return <span className={["rounded-md border px-2 py-0.5 text-xs font-semibold", tone].join(" ")}>{formatResult(result, t)}</span>;
}
