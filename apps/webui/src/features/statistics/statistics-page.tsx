import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { getRouteApi } from "@tanstack/react-router";
import {
  faChartLine,
  faLayerGroup,
  faRotate,
  faRocket,
} from "@fortawesome/free-solid-svg-icons";
import { statisticsQueries } from "../../lib/queries";
import { formatDateTime } from "../../lib/datetime";
import type { TimeRange } from "../../lib/types";
import ErrorMessage from "../../components/common/error-message";
import { usePageVisible } from "../../components/common/page-visibility-provider";
import LoadingBlock from "../../components/ui/loading-block";
import FaIcon from "../../components/ui/fa-icon";
import PageHeader from "../../components/ui/page-header";
import RatioBar from "../../components/ui/ratio-bar";
import Tabs from "../../components/ui/tabs";
import TimeRangeSelector from "../../components/ui/time-range-selector";
import TimeSeriesChart from "../../components/ui/time-series-chart";

const STATS_REFRESH_INTERVAL_MS = 15_000;
const TIMESERIES_REFRESH_INTERVAL_MS = 30_000;

const route = getRouteApi("/_authed/statistics");

function formatSeconds(value: number | null | undefined): string {
  if (value == null) {
    return "-";
  }
  if (value < 60) {
    return `${value}s`;
  }
  if (value < 3600) {
    return `${Math.floor(value / 60)}m ${value % 60}s`;
  }
  const hours = Math.floor(value / 3600);
  const minutes = Math.floor((value % 3600) / 60);
  return `${hours}h ${minutes}m`;
}

function Statistics() {
  const visible = usePageVisible();
  const navigate = route.useNavigate();
  const search = route.useSearch();
  const range: TimeRange = search.range ?? "24h";
  const setRange = (next: TimeRange) =>
    navigate({ search: (prev) => ({ ...prev, range: next }) });
  const [cacheTab, setCacheTab] = useState<"mock" | "git">("mock");

  const { data, isPending, error } = useQuery({
    ...statisticsQueries.overview(),
    refetchInterval: visible ? STATS_REFRESH_INTERVAL_MS : false,
  });

  const timeseriesQuery = useQuery({
    ...statisticsQueries.timeseries(range),
    refetchInterval: visible ? TIMESERIES_REFRESH_INTERVAL_MS : false,
  });

  if (isPending) {
    return (
      <div className="space-y-6">
        <PageHeader
          title="Statistics"
          description="Dedicated operational metrics for system throughput, sync health, and cache behavior."
          color="cyan"
          actions={[
            { to: "/", label: "Overview", icon: faLayerGroup },
            { to: "/jobs", label: "Jobs", icon: faRocket },
          ]}
        />
        <LoadingBlock label="Loading statistics…" lines={4} />
      </div>
    );
  }

  if (error) {
    return (
      <ErrorMessage
        message={error instanceof Error ? error.message : "Failed to load statistics"}
      />
    );
  }

  const chrootCache = data.cacheStats.mock_chroot_cache;
  const mirrorCache = data.cacheStats.git_mirror_cache;
  const healthyMirrors = Math.max(
    0,
    mirrorCache.tracked_mirrors - mirrorCache.stale_mirrors,
  );

  return (
    <div className="space-y-6">
      <PageHeader
        title="Statistics"
        description="Dedicated operational metrics for system throughput, sync health, and cache behavior."
        color="cyan"
        actions={[
          { to: "/", label: "Overview", icon: faLayerGroup },
          { to: "/jobs", label: "Jobs", icon: faRocket },
        ]}
      />

      {/* Time-series — sync and build activity over a selectable window. */}
      <section className="space-y-4">
        <div className="flex flex-col gap-3 sm:flex-row sm:items-end sm:justify-between">
          <div>
            <h2 className="text-lg font-bold text-white">Activity over time</h2>
            <p className="mt-1 text-xs text-soft">
              Stacked succeeded vs failed counts. Switch the window to zoom out.
            </p>
          </div>
          <TimeRangeSelector value={range} onChange={setRange} />
        </div>

        <div className="grid gap-6 xl:grid-cols-2">
          <article className="border-2 border-edge-strong bg-black p-5">
            <div className="mb-3 flex items-center gap-2 font-mono text-xs font-bold uppercase tracking-[0.18em] text-accent-cyan">
              <FaIcon icon={faRotate} />
              Sync operations
            </div>
            <TimeSeriesChart
              data={timeseriesQuery.data?.sync}
              isLoading={timeseriesQuery.isPending}
              emptyLabel="No sync activity in this window."
            />
          </article>

          <article className="border-2 border-edge-strong bg-black p-5">
            <div className="mb-3 flex items-center gap-2 font-mono text-xs font-bold uppercase tracking-[0.18em] text-accent-lime">
              <FaIcon icon={faChartLine} />
              Build jobs
            </div>
            <TimeSeriesChart
              data={timeseriesQuery.data?.jobs}
              isLoading={timeseriesQuery.isPending}
              emptyLabel="No completed builds in this window."
            />
          </article>
        </div>
      </section>

      {/* Ratio visualizations — point-in-time distributions */}
      <section className="grid gap-6 md:grid-cols-2 xl:grid-cols-3">
        <article className="border-2 border-edge-strong bg-black p-5">
          <h3 className="text-sm font-semibold text-white">
            Sync outcomes (24h)
          </h3>
          <div className="mt-3">
            <RatioBar
              segments={[
                {
                  label: "Succeeded",
                  value: data.syncMetrics.succeeded_24h,
                  color: "var(--theme-terminal-green)",
                },
                {
                  label: "Failed",
                  value: data.syncMetrics.failed_24h,
                  color: "var(--theme-error-red)",
                },
              ]}
            />
          </div>
        </article>

        <article className="border-2 border-edge-strong bg-black p-5">
          <h3 className="text-sm font-semibold text-white">
            Mock chroot cache
          </h3>
          <div className="mt-3">
            <RatioBar
              segments={[
                {
                  label: "Hit",
                  value: chrootCache.hit_count,
                  color: "var(--theme-accent-lime)",
                },
                {
                  label: "Miss",
                  value: chrootCache.miss_count,
                  color: "var(--theme-accent-orange)",
                },
                {
                  label: "Stale served",
                  value: chrootCache.stale_served_count,
                  color: "var(--theme-accent-cyan)",
                },
              ]}
            />
          </div>
        </article>

        <article className="border-2 border-edge-strong bg-black p-5">
          <h3 className="text-sm font-semibold text-white">
            Git mirror health
          </h3>
          <div className="mt-3">
            <RatioBar
              segments={[
                {
                  label: "Healthy",
                  value: healthyMirrors,
                  color: "var(--theme-terminal-green)",
                },
                {
                  label: "Stale",
                  value: mirrorCache.stale_mirrors,
                  color: "var(--theme-accent-orange)",
                },
              ]}
            />
          </div>
        </article>
      </section>

      {/* Cache detail — one panel at a time */}
      <Tabs
        ariaLabel="Cache details"
        value={cacheTab}
        onChange={setCacheTab}
        items={[
          { value: "mock", label: "Mock chroot cache" },
          { value: "git", label: "Git mirror cache" },
        ]}
      >
        {cacheTab === "mock" ? (
          <div className="grid gap-px bg-edge">
            <StatRow label="Worker image" value={chrootCache.worker_image ?? "-"} />
            <StatRow label="TTL" value={formatSeconds(chrootCache.ttl_seconds)} />
            <StatRow label="Hit count" value={String(chrootCache.hit_count)} />
            <StatRow label="Miss count" value={String(chrootCache.miss_count)} />
            <StatRow
              label="Stale served"
              value={String(chrootCache.stale_served_count)}
            />
            <StatRow
              label="Last refresh"
              value={formatDateTime(chrootCache.last_refresh_at, "-")}
            />
          </div>
        ) : null}
        {cacheTab === "git" ? (
          <div className="grid gap-px bg-edge">
            <StatRow label="Mirror root" value={mirrorCache.mirror_root} />
            <StatRow
              label="Refresh TTL"
              value={formatSeconds(mirrorCache.refresh_ttl_seconds)}
            />
            <StatRow
              label="Max unused"
              value={formatSeconds(mirrorCache.max_unused_seconds)}
            />
            <StatRow
              label="Tracked mirrors"
              value={String(mirrorCache.tracked_mirrors)}
            />
            <StatRow
              label="Stale mirrors"
              value={String(mirrorCache.stale_mirrors)}
            />
            <StatRow
              label="Latest fetched"
              value={formatDateTime(mirrorCache.latest_fetched_at, "-")}
            />
            <StatRow
              label="Latest used"
              value={formatDateTime(mirrorCache.latest_used_at, "-")}
            />
          </div>
        ) : null}
      </Tabs>

      <p className="font-mono text-[11px] text-soft">
        Snapshot collected at{" "}
        {formatDateTime(data.cacheStats.collected_at, "n/a")}.
      </p>
    </div>
  );
}

function StatRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="grid gap-2 bg-black px-4 py-3 sm:grid-cols-[180px_minmax(0,1fr)] sm:gap-3 sm:px-5">
      <div className="font-mono text-xs font-bold uppercase tracking-[0.15em] text-soft">
        {label}
      </div>
      <div className="break-words font-mono text-sm text-strong sm:truncate">{value}</div>
    </div>
  );
}

export default function StatisticsPage() {
  return <Statistics />;
}
