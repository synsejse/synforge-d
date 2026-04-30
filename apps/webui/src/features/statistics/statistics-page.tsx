import { useQuery } from "@tanstack/react-query";
import {
  faBoxesStacked,
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
import ErrorMessage from "../../components/common/error-message";
import LoadingBlock from "../../components/ui/loading-block";
import FaIcon from "../../components/ui/fa-icon";
import MetricCard from "../../components/ui/metric-card";
import PageHeader from "../../components/ui/page-header";

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
  const { data, isPending, error } = useQuery(statisticsQueries.overview());

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

      <section className="grid gap-6 xl:grid-cols-2">
        <article className="border-4 border-[var(--theme-border-strong)] bg-black">
          <header className="border-b-4 border-[var(--theme-border-strong)] bg-zinc-950 px-6 py-4">
            <h2 className="font-mono text-sm font-bold uppercase tracking-[0.2em] text-white">
              Mock_Chroot_Cache
            </h2>
          </header>
          <div className="grid gap-px bg-zinc-800">
            <StatRow label="Worker image" value={data.cacheStats.mock_chroot_cache.worker_image ?? "-"} />
            <StatRow label="TTL" value={formatSeconds(data.cacheStats.mock_chroot_cache.ttl_seconds)} />
            <StatRow label="Hit count" value={String(data.cacheStats.mock_chroot_cache.hit_count)} />
            <StatRow label="Miss count" value={String(data.cacheStats.mock_chroot_cache.miss_count)} />
            <StatRow
              label="Stale served"
              value={String(data.cacheStats.mock_chroot_cache.stale_served_count)}
            />
            <StatRow
              label="Last refresh"
              value={formatDateTime(data.cacheStats.mock_chroot_cache.last_refresh_at, "-")}
            />
          </div>
        </article>

        <article className="border-4 border-[var(--theme-border-strong)] bg-black">
          <header className="border-b-4 border-[var(--theme-border-strong)] bg-zinc-950 px-6 py-4">
            <h2 className="font-mono text-sm font-bold uppercase tracking-[0.2em] text-white">
              Git_Mirror_Cache
            </h2>
          </header>
          <div className="grid gap-px bg-zinc-800">
            <StatRow label="Mirror root" value={data.cacheStats.git_mirror_cache.mirror_root} />
            <StatRow
              label="Refresh TTL"
              value={formatSeconds(data.cacheStats.git_mirror_cache.refresh_ttl_seconds)}
            />
            <StatRow
              label="Max unused"
              value={formatSeconds(data.cacheStats.git_mirror_cache.max_unused_seconds)}
            />
            <StatRow
              label="Tracked mirrors"
              value={String(data.cacheStats.git_mirror_cache.tracked_mirrors)}
            />
            <StatRow
              label="Stale mirrors"
              value={String(data.cacheStats.git_mirror_cache.stale_mirrors)}
            />
            <StatRow
              label="Latest fetched"
              value={formatDateTime(data.cacheStats.git_mirror_cache.latest_fetched_at, "-")}
            />
            <StatRow
              label="Latest used"
              value={formatDateTime(data.cacheStats.git_mirror_cache.latest_used_at, "-")}
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
      <div className="font-mono text-xs font-bold uppercase tracking-[0.15em] text-zinc-500">
        {label}
      </div>
      <div className="break-words font-mono text-sm text-zinc-200 sm:truncate">{value}</div>
    </div>
  );
}

export default function StatisticsPage() {
  return <Statistics />;
}
