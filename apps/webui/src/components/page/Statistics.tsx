import { useEffect, useState } from "react";
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
import api from "../../lib/api";
import { formatDateTime } from "../../lib/datetime";
import type {
  CacheStatsResponse,
  RepoSummaryResponse,
  SyncMetricsResponse,
} from "../../lib/types";
import ErrorMessage from "../common/ErrorMessage";
import LoadingBlock from "../ui/LoadingBlock";
import FaIcon from "../ui/FaIcon";
import MetricCard from "../ui/MetricCard";
import PageHeader from "../ui/PageHeader";

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

export default function Statistics() {
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [cacheStats, setCacheStats] = useState<CacheStatsResponse | null>(null);
  const [syncMetrics, setSyncMetrics] = useState<SyncMetricsResponse | null>(null);
  const [repoSummary, setRepoSummary] = useState<RepoSummaryResponse | null>(null);
  const [packageCount, setPackageCount] = useState(0);
  const [enabledPackageCount, setEnabledPackageCount] = useState(0);
  const [activeJobCount, setActiveJobCount] = useState(0);

  useEffect(() => {
    async function load() {
      try {
        const [
          cacheStatsRes,
          syncMetricsRes,
          repoSummaryRes,
          packagesRes,
          enabledPackagesRes,
          activeJobsRes,
        ] = await Promise.all([
          api.getCacheStats(),
          api.getSyncMetrics(),
          api.getRepoSummary(),
          api.listPackagesPage(1, 0),
          api.listPackagesPage(1, 0, { enabled: true }),
          api.listActiveJobs({ limit: 1, offset: 0 }),
        ]);
        setCacheStats(cacheStatsRes);
        setSyncMetrics(syncMetricsRes);
        setRepoSummary(repoSummaryRes);
        setPackageCount(packagesRes.page.total ?? packagesRes.packages.length);
        setEnabledPackageCount(
          enabledPackagesRes.page.total ?? enabledPackagesRes.packages.length,
        );
        setActiveJobCount(activeJobsRes.page.total ?? activeJobsRes.jobs.length);
        setError(null);
      } catch (e) {
        setError(e instanceof Error ? e.message : "Failed to load statistics");
      } finally {
        setLoading(false);
      }
    }
    load();
  }, []);

  if (loading) {
    return <LoadingBlock label="Loading statistics…" lines={4} />;
  }

  if (error || !cacheStats || !syncMetrics || !repoSummary) {
    return <ErrorMessage message={error || "Failed to load statistics"} />;
  }

  return (
    <div className="space-y-8">
      <PageHeader
        eyebrow="SYSTEM_TELEMETRY"
        title="Statistics"
        description="Dedicated operational metrics for system throughput, sync health, and cache behavior."
        color="cyan"
        actions={[
          { href: "/", label: "Overview", icon: faLayerGroup },
          { href: "/jobs/", label: "Jobs", icon: faRocket },
        ]}
      />

      <section className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
        <MetricCard
          label="Packages"
          value={packageCount}
          detail={`${enabledPackageCount} enabled`}
          icon={<FaIcon icon={faBoxesStacked} />}
        />
        <MetricCard
          label="Active_Jobs"
          value={activeJobCount}
          detail="Pending + running"
          variant="terminal"
          icon={<FaIcon icon={faRocket} />}
        />
        <MetricCard
          label="Sync_Succeeded_24h"
          value={syncMetrics.succeeded_24h}
          detail="Successful source syncs"
          variant="success"
          icon={<FaIcon icon={faCircleCheck} />}
        />
        <MetricCard
          label="Sync_Failures_24h"
          value={syncMetrics.failed_24h}
          detail={
            syncMetrics.last_failure_at
              ? `Last ${formatDateTime(syncMetrics.last_failure_at)}`
              : "No recent failures"
          }
          variant={syncMetrics.failed_24h > 0 ? "error" : "success"}
          icon={<FaIcon icon={faTriangleExclamation} />}
        />
        <MetricCard
          label="Stored_Bytes"
          value={repoSummary.stored_bytes}
          detail={`${repoSummary.published_file_count} published files`}
          icon={<FaIcon icon={faHardDrive} />}
        />
        <MetricCard
          label="Git_Mirrors"
          value={cacheStats.git_mirror_cache.tracked_mirrors}
          detail={`${cacheStats.git_mirror_cache.mirror_directories} directories on disk`}
          icon={<FaIcon icon={faDatabase} />}
        />
        <MetricCard
          label="Cached_Chroots"
          value={cacheStats.mock_chroot_cache.cached_chroot_count}
          detail={
            cacheStats.mock_chroot_cache.age_seconds != null
              ? `Age ${formatSeconds(cacheStats.mock_chroot_cache.age_seconds)}`
              : "No cached entry"
          }
          icon={<FaIcon icon={faRotate} />}
        />
        <MetricCard
          label="Collected_At"
          value={formatDateTime(cacheStats.collected_at, "n/a")}
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
            <StatRow label="Worker image" value={cacheStats.mock_chroot_cache.worker_image ?? "-"} />
            <StatRow label="TTL" value={formatSeconds(cacheStats.mock_chroot_cache.ttl_seconds)} />
            <StatRow label="Hit count" value={String(cacheStats.mock_chroot_cache.hit_count)} />
            <StatRow label="Miss count" value={String(cacheStats.mock_chroot_cache.miss_count)} />
            <StatRow
              label="Stale served"
              value={String(cacheStats.mock_chroot_cache.stale_served_count)}
            />
            <StatRow
              label="Last refresh"
              value={formatDateTime(cacheStats.mock_chroot_cache.last_refresh_at, "-")}
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
            <StatRow label="Mirror root" value={cacheStats.git_mirror_cache.mirror_root} />
            <StatRow
              label="Refresh TTL"
              value={formatSeconds(cacheStats.git_mirror_cache.refresh_ttl_seconds)}
            />
            <StatRow
              label="Max unused"
              value={formatSeconds(cacheStats.git_mirror_cache.max_unused_seconds)}
            />
            <StatRow
              label="Tracked mirrors"
              value={String(cacheStats.git_mirror_cache.tracked_mirrors)}
            />
            <StatRow
              label="Stale mirrors"
              value={String(cacheStats.git_mirror_cache.stale_mirrors)}
            />
            <StatRow
              label="Latest fetched"
              value={formatDateTime(cacheStats.git_mirror_cache.latest_fetched_at, "-")}
            />
            <StatRow
              label="Latest used"
              value={formatDateTime(cacheStats.git_mirror_cache.latest_used_at, "-")}
            />
          </div>
        </article>
      </section>
    </div>
  );
}

function StatRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="grid grid-cols-[220px_minmax(0,1fr)] gap-3 bg-black px-5 py-3">
      <div className="font-mono text-xs font-bold uppercase tracking-[0.15em] text-zinc-500">
        {label}
      </div>
      <div className="truncate font-mono text-sm text-zinc-200">{value}</div>
    </div>
  );
}
