import { useAppCore } from "../state/AppStateProvider";
import type { LeagueClientStatus } from "../backend/types";
import { Metric, RefreshIcon } from "../components/common";
import { formatTimestamp, formatLeaguePhase, type T } from "../utils/formatting";

export function Dashboard({ onOpenSettings }: { onOpenSettings: () => void }) {
  const { snapshot, isLoading, leagueSelfSnapshot, isLeagueClientLoading, refreshLeagueClient, t } = useAppCore();
  const health = snapshot?.health;
  const recentActivity = snapshot?.recentActivity ?? [];
  const settings = snapshot?.settings;
  const aiConfigured = Boolean(settings?.aiBaseUrl && settings?.aiApiKey && settings?.aiModel);
  const clientPhase = leagueSelfSnapshot?.status.phase;
  const clientReady = clientPhase === "connected" || clientPhase === "partialData";

  return (
    <main className="min-h-0 flex-1 overflow-auto px-8 py-7">
      <div className="mx-auto flex w-full max-w-6xl flex-col gap-7">
        <header className="flex flex-wrap items-end justify-between gap-4">
          <div>
            <p className="text-sm font-medium uppercase tracking-wide text-rose-700">{t("dashboard.eyebrow")}</p>
            <h1 className="mt-2 text-3xl font-semibold text-zinc-950 dark:text-zinc-50">{t("app.name")}</h1>
          </div>
          <HealthBadge status={health?.status ?? (isLoading ? "loading" : "degraded")} t={t} />
        </header>

        <SetupChecklist
          aiConfigured={aiConfigured}
          clientReady={clientReady}
          onOpenSettings={onOpenSettings}
          t={t}
        />

        <LeagueOverview
          averageKda={leagueSelfSnapshot?.recentPerformance.averageKda ?? null}
          isLoading={isLeagueClientLoading}
          onRefresh={() => refreshLeagueClient({ matchLimit: 6 })}
          refreshedAt={leagueSelfSnapshot?.refreshedAt ?? null}
          status={leagueSelfSnapshot?.status}
          summonerName={leagueSelfSnapshot?.summoner?.displayName ?? null}
          t={t}
        />

        <section className="rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-900 p-5 shadow-sm">
          <h2 className="text-base font-semibold text-zinc-950 dark:text-zinc-50">{t("dashboard.recentActivity")}</h2>
          <div className="mt-5 min-h-24 rounded-md border border-zinc-200 dark:border-zinc-700 bg-zinc-50 dark:bg-zinc-800 p-4 text-sm text-zinc-700 dark:text-zinc-300">
            {isLoading && t("dashboard.loadingActivity")}
            {!isLoading && recentActivity.length === 0 && t("dashboard.noActivity")}
            {!isLoading && recentActivity.length > 0 && (
              <div className="grid gap-3">
                {recentActivity.slice(0, 3).map((entry) => (
                  <div key={entry.id} className="min-w-0">
                    <p className="truncate font-semibold text-zinc-950 dark:text-zinc-50">{entry.title}</p>
                    <p className="mt-1 text-xs capitalize text-zinc-500 dark:text-zinc-400">{entry.kind}</p>
                  </div>
                ))}
              </div>
            )}
          </div>
        </section>

        <details className="overflow-hidden rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-900 shadow-sm">
          <summary className="cursor-pointer select-none px-5 py-4 text-sm font-semibold text-zinc-700 dark:text-zinc-300">
            {t("home.systemStatus")}
          </summary>
          <div className="grid gap-4 px-5 pb-5 md:grid-cols-3">
            <StatusTile label={t("dashboard.application")} value={health?.status ?? t("common.loading")} tone={health?.status === "ok" ? "good" : "warn"} />
            <StatusTile
              label={t("dashboard.database")}
              value={health?.databaseStatus ?? t("common.pending")}
              tone={health?.databaseStatus === "ok" ? "good" : "warn"}
            />
            <StatusTile
              label={t("dashboard.schema")}
              value={health ? String(health.schemaVersion ?? t("common.noData")) : t("common.pending")}
              tone={health?.schemaVersion ? "good" : "warn"}
            />
          </div>
          <div className="grid gap-3 border-t border-zinc-200 dark:border-zinc-700 px-5 py-5 sm:grid-cols-2">
            <Metric label={t("dashboard.startup")} value={snapshot?.settings.startupPage ?? t("common.loading")} />
            <Metric label={t("dashboard.density")} value={snapshot?.settings.compactMode ? t("dashboard.compact") : t("dashboard.standard")} />
            <Metric label={t("dashboard.activityLimit")} value={snapshot ? String(snapshot.settings.activityLimit) : t("common.loading")} />
            <Metric label={t("dashboard.activityEntries")} value={String(recentActivity.length)} />
          </div>
        </details>
      </div>
    </main>
  );
}

function SetupChecklist({
  aiConfigured,
  clientReady,
  onOpenSettings,
  t,
}: {
  aiConfigured: boolean;
  clientReady: boolean;
  onOpenSettings: () => void;
  t: T;
}) {
  if (clientReady && aiConfigured) {
    return (
      <section className="flex items-center gap-2 rounded-lg border border-emerald-200 dark:border-emerald-800 bg-emerald-50 dark:bg-emerald-950 px-5 py-3 text-sm font-medium text-emerald-800 dark:text-emerald-300">
        <CheckIcon />
        {t("home.allSet")}
      </section>
    );
  }

  return (
    <section className="rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-900 p-5 shadow-sm">
      <h2 className="text-base font-semibold text-zinc-950 dark:text-zinc-50">{t("home.setupTitle")}</h2>
      <div className="mt-4 grid gap-3">
        <SetupStep
          done={clientReady}
          title={t("home.stepClient")}
          detail={clientReady ? t("common.connected") : t("home.stepClientPending")}
        />
        <SetupStep
          done={aiConfigured}
          title={t("home.stepAi")}
          detail={aiConfigured ? t("home.stepAiReady") : t("home.stepAiPending")}
          action={aiConfigured ? undefined : { label: t("home.configureAi"), onClick: onOpenSettings }}
        />
        <SetupStep info title={t("home.overlayTip")} detail={t("home.overlayTipBody")} />
      </div>
    </section>
  );
}

function SetupStep({
  done = false,
  info = false,
  title,
  detail,
  action,
}: {
  done?: boolean;
  info?: boolean;
  title: string;
  detail: string;
  action?: { label: string; onClick: () => void };
}) {
  return (
    <div className="flex items-center gap-3 rounded-md border border-zinc-200 dark:border-zinc-700 bg-zinc-50 dark:bg-zinc-800 px-4 py-3">
      <StepMarker done={done} info={info} />
      <div className="min-w-0 flex-1">
        <p className="text-sm font-semibold text-zinc-950 dark:text-zinc-50">{title}</p>
        <p className="mt-0.5 text-xs text-zinc-500 dark:text-zinc-400">{detail}</p>
      </div>
      {action && (
        <button
          type="button"
          onClick={action.onClick}
          className="inline-flex h-8 shrink-0 items-center rounded-md bg-rose-700 px-3 text-xs font-semibold text-white transition hover:bg-rose-800"
        >
          {action.label}
        </button>
      )}
    </div>
  );
}

function StepMarker({ done, info }: { done: boolean; info: boolean }) {
  if (info) {
    return (
      <span className="flex h-5 w-5 shrink-0 items-center justify-center rounded-full bg-sky-100 text-xs font-bold text-sky-700 dark:bg-sky-950 dark:text-sky-300">
        i
      </span>
    );
  }
  if (done) {
    return (
      <span className="flex h-5 w-5 shrink-0 items-center justify-center rounded-full bg-emerald-100 text-emerald-700 dark:bg-emerald-950 dark:text-emerald-300">
        <CheckIcon />
      </span>
    );
  }
  return <span className="h-2.5 w-2.5 shrink-0 rounded-full bg-amber-500" />;
}

function CheckIcon() {
  return (
    <svg aria-hidden="true" className="h-3.5 w-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3">
      <path d="M5 13l4 4L19 7" strokeLinecap="round" strokeLinejoin="round" />
    </svg>
  );
}

function LeagueOverview({
  averageKda,
  isLoading,
  onRefresh,
  refreshedAt,
  status,
  summonerName,
  t,
}: {
  averageKda: number | null;
  isLoading: boolean;
  onRefresh: () => void;
  refreshedAt: string | null;
  status: LeagueClientStatus | undefined;
  summonerName: string | null;
  t: T;
}) {
  return (
    <section className="rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-900 p-5 shadow-sm">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div>
          <h2 className="text-base font-semibold text-zinc-950 dark:text-zinc-50">{t("dashboard.leagueClient")}</h2>
          <p className="mt-1 text-sm text-zinc-500 dark:text-zinc-400">{leagueStatusMessage(status, isLoading, t)}</p>
        </div>
        <div className="flex items-center gap-2">
          <LeagueStatusBadge isLoading={isLoading} status={status} t={t} />
          <button
            className="inline-flex h-10 items-center gap-2 rounded-md border border-zinc-300 dark:border-zinc-600 bg-white dark:bg-zinc-900 px-3 text-sm font-medium text-zinc-800 dark:text-zinc-200 transition hover:border-zinc-400 hover:bg-zinc-50 dark:hover:bg-zinc-800 disabled:cursor-not-allowed disabled:opacity-60"
            disabled={isLoading}
            onClick={onRefresh}
            type="button"
          >
            <RefreshIcon />
            {t("common.refresh")}
          </button>
        </div>
      </div>

      <div className="mt-5 grid gap-3 md:grid-cols-3">
        <Metric label={t("dashboard.summoner")} value={summonerName ?? (isLoading ? t("common.loading") : t("common.unavailable"))} />
        <Metric label={t("dashboard.recentKda")} value={averageKda === null ? t("common.unavailable") : averageKda.toFixed(1)} />
        <Metric label={t("dashboard.updated")} value={formatTimestamp(refreshedAt, t)} />
      </div>
    </section>
  );
}

function LeagueStatusBadge({ status, isLoading, t }: { status: LeagueClientStatus | undefined; isLoading: boolean; t: T }) {
  const isReady = status?.connection === "connected" && status.phase === "connected";
  const isPartial = status?.phase === "partialData";
  const label = isLoading ? (status ? t("common.refreshing") : t("common.loading")) : isReady ? t("common.connected") : formatLeaguePhase(status?.phase, t);

  return (
    <div
      className={[
        "inline-flex h-10 items-center gap-2 rounded-md border px-3 text-sm font-medium",
        isReady
          ? "border-emerald-200 bg-emerald-50 text-emerald-800"
          : isPartial
            ? "border-sky-200 bg-sky-50 text-sky-800"
            : "border-amber-200 bg-amber-50 text-amber-800",
      ].join(" ")}
    >
      <span className={["h-2.5 w-2.5 rounded-full", isReady ? "bg-emerald-600" : isPartial ? "bg-sky-600" : "bg-amber-500"].join(" ")} />
      {label}
    </div>
  );
}

function HealthBadge({ status, t }: { status: "ok" | "degraded" | "loading"; t: T }) {
  const isReady = status === "ok";

  return (
    <div
      className={[
        "inline-flex h-10 items-center gap-2 rounded-md border px-3 text-sm font-medium",
        isReady ? "border-emerald-200 bg-emerald-50 text-emerald-800" : "border-amber-200 bg-amber-50 text-amber-800",
      ].join(" ")}
    >
      <span className={["h-2.5 w-2.5 rounded-full", isReady ? "bg-emerald-600" : "bg-amber-500"].join(" ")} />
      {isReady ? t("common.ready") : t("common.pending")}
    </div>
  );
}

function StatusTile({ label, value, tone }: { label: string; value: string; tone: "good" | "warn" }) {
  return (
    <div className="rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-900 p-5 shadow-sm">
      <p className="text-sm font-medium text-zinc-500 dark:text-zinc-400">{label}</p>
      <p className={["mt-3 text-2xl font-semibold capitalize", tone === "good" ? "text-emerald-700" : "text-amber-700"].join(" ")}>
        {value}
      </p>
    </div>
  );
}

function leagueStatusMessage(status: LeagueClientStatus | undefined, isLoading: boolean, t: T) {
  if (isLoading && !status) {
    return t("dashboard.checkingClient");
  }

  if (isLoading) {
    return t("dashboard.refreshingClient");
  }

  if (status?.message) {
    return status.message;
  }

  if (status?.connection === "connected") {
    return t("dashboard.clientReady");
  }

  return t("dashboard.clientUnavailable");
}

