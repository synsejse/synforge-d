import { useQuery } from "@tanstack/react-query";
import { getRouteApi } from "@tanstack/react-router";
import {
  faBoxesStacked,
  faChartLine,
  faCircleCheck,
  faClockRotateLeft,
  faDatabase,
  faHardDrive,
  faLayerGroup,
  faRotate,
  faRocket,
  faTriangleExclamation,
} from "@fortawesome/free-solid-svg-icons";
import { statisticsQueries } from "../../lib/queries";
import { formatDateTime } from "../../lib/datetime";
import type { TimeRange } from "../../lib/types";
import ErrorMessage from "../../components/common/error-message";
import { usePageVisible } from "../../components/common/page-visibility-provider";
import LoadingBlock from "../../components/ui/loading-block";
import FaIcon from "../../components/ui/fa-icon";
import MetricCard from "../../components/ui/metric-card";
import PageHeader from "../../components/ui/page-header";
import RatioBar from "../../components/ui/ratio-bar";
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

  const { data, isPending, error } = useQuery({
    ...statisticsQueries.overview(),
    refetchInterval: visible ? STATS_REFRESH_INTERVAL_MS : false,
  });

  const timeseriesQuery = useQuery({
    ...statisticsQueries.timeseries(range),
    refetchInterval: visible ? TIMESERIES_REFRESH_INTERVAL_MS : false,
  });

  if (isPending) {
    return <LoadingBlock label="Loading statistics…" lines={4} />;
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
    <div className="space-y-8">
      <PageHeader
        eyebrow="SYSTEM_TELEMETRY"
        title="Statistics"
        description="Dedicated operational metrics for system throughput, sync health, and cache behavior."
        color="cyan"
        actions={[
          { to: "/", label: "Overview", icon: faLayerGroup },
          { to: "/jobs", label: "Jobs", icon: faRocket },
        ]}
      />

      <section className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
        <MetricCard
          label="Packages"
          value={data.packageCount}
          detail={`${data.enabledPackageCount} enabled`}
          icon={<FaIcon icon={faBoxesStacked} />}
        />
        <MetricCard
          label="Active_Jobs"
          value={data.activeJobCount}
          detail="Pending + running"
          variant="terminal"
          icon={<FaIcon icon={faRocket} />}
        />
        <MetricCard
          label="Sync_Succeeded_24h"
          value={data.syncMetrics.succeeded_24h}
          detail="Successful source syncs"
          variant="success"
          icon={<FaIcon icon={faCircleCheck} />}
        />
        <MetricCard
          label="Sync_Failures_24h"
          value={data.syncMetrics.failed_24h}
          detail={
            data.syncMetrics.last_failure_at
              ? `Last ${formatDateTime(data.syncMetrics.last_failure_at)}`
              : "No recent failures"
          }
          variant={data.syncMetrics.failed_24h > 0 ? "error" : "success"}
          icon={<FaIcon icon={faTriangleExclamation} />}
        />
        <MetricCard
          label="Stored_Bytes"
          value={data.repoSummary.stored_bytes}
          detail={`${data.repoSummary.published_file_count} published files`}
          icon={<FaIcon icon={faHardDrive} />}
        />
        <MetricCard
          label="Git_Mirrors"
          value={data.cacheStats.git_mirror_cache.tracked_mirrors}
          detail={`${data.cacheStats.git_mirror_cache.mirror_directories} directories on disk`}
          icon={<FaIcon icon={faDatabase} />}
        />
        <MetricCard
          label="Cached_Chroots"
          value={data.cacheStats.mock_chroot_cache.cached_chroot_count}
          detail={
            data.cacheStats.mock_chroot_cache.age_seconds != null
              ? `Age ${formatSeconds(data.cacheStats.mock_chroot_cache.age_seconds)}`
              : "No cached entry"
          }
          icon={<FaIcon icon={faRotate} />}
        />
        <MetricCard
          label="Collected_At"
          value={formatDateTime(data.cacheStats.collected_at, "n/a")}
          detail="API snapshot timestamp"
          icon={<FaIcon icon={faClockRotateLeft} />}
        />
      </section>

      {/* Time-series — sync and build activity over a selectable window. */}
      <section className="space-y-4">
        <div className="flex flex-col gap-3 sm:flex-row sm:items-end sm:justify-between">
          <div>
            <h2 className="font-mono text-sm font-bold uppercase tracking-[0.2em] text-white">
              Activity_Over_Time
            </h2>
            <p className="mt-1 font-mono text-xs text-soft">
              Stacked succeeded vs failed counts. Switch the window to zoom out.
            </p>
          </div>
          <TimeRangeSelector value={range} onChange={setRange} />
        </div>

        <div className="grid gap-6 xl:grid-cols-2">
          <article className="border-4 border-edge-strong bg-black p-5">
            <div className="mb-3 flex items-center gap-2 font-mono text-xs font-bold uppercase tracking-[0.2em] text-accent-cyan">
              <FaIcon icon={faRotate} />
              Sync_Operations
            </div>
            <TimeSeriesChart
              data={timeseriesQuery.data?.sync}
              isLoading={timeseriesQuery.isPending}
              emptyLabel="No sync activity in this window."
            />
          </article>

          <article className="border-4 border-edge-strong bg-black p-5">
            <div className="mb-3 flex items-center gap-2 font-mono text-xs font-bold uppercase tracking-[0.2em] text-accent-lime">
              <FaIcon icon={faChartLine} />
              Build_Jobs
            </div>
            <TimeSeriesChart
              data={timeseriesQuery.data?.jobs}
              isLoading={timeseriesQuery.isPending}
              emptyLabel="No completed builds in this window."
            />
          </article>
        </div>
      </section>

      {/* Ratio visualizations — point-in-time distributions to complement
          the time-series above. */}
      <section className="grid gap-6 md:grid-cols-2 xl:grid-cols-3">
        <article className="border-4 border-edge-strong bg-black p-6">
          <h3 className="font-mono text-sm font-bold uppercase tracking-[0.2em] text-white">
            Sync_Outcomes_24h
          </h3>
          <p className="mt-1 font-mono text-xs text-soft">
            Distribution of source-sync attempts in the last 24 hours.
          </p>
          <div className="mt-4">
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

        <article className="border-4 border-edge-strong bg-black p-6">
          <h3 className="font-mono text-sm font-bold uppercase tracking-[0.2em] text-white">
            Mock_Chroot_Cache
          </h3>
          <p className="mt-1 font-mono text-xs text-soft">
            Hit / miss / stale-served counts since daemon start.
          </p>
          <div className="mt-4">
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

        <article className="border-4 border-edge-strong bg-black p-6">
          <h3 className="font-mono text-sm font-bold uppercase tracking-[0.2em] text-white">
            Git_Mirror_Health
          </h3>
          <p className="mt-1 font-mono text-xs text-soft">
            Healthy vs stale tracked mirrors right now.
          </p>
          <div className="mt-4">
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

      <section className="grid gap-6 xl:grid-cols-2">
        <article className="border-4 border-edge-strong bg-black">
          <header className="border-b-4 border-edge-strong bg-surface-alt px-6 py-4">
            <h2 className="font-mono text-sm font-bold uppercase tracking-[0.2em] text-white">
              Mock_Chroot_Cache
            </h2>
          </header>
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
        </article>

        <article className="border-4 border-edge-strong bg-black">
          <header className="border-b-4 border-edge-strong bg-surface-alt px-6 py-4">
            <h2 className="font-mono text-sm font-bold uppercase tracking-[0.2em] text-white">
              Git_Mirror_Cache
            </h2>
          </header>
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
        </article>
      </section>
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
