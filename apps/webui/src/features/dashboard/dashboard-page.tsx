import { useEffect, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { dashboardQueries, jobsQueries } from "../../lib/queries";
import { formatBytes } from "../../lib/bytes";
import type { JobResourceUsageSample } from "../../lib/types";
import ErrorMessage from "../../components/common/error-message";
import { usePageVisible } from "../../components/common/page-visibility-context";
import FaIcon from "../../components/ui/fa-icon";
import LoadingBlock from "../../components/ui/loading-block";
import MetricCard from "../../components/ui/metric-card";
import PageHeader from "../../components/ui/page-header";
import BuildRunRow from "./build-run-row";
import ActiveBuildsSection from "./active-builds-section";
import SyncScheduleStrip from "./sync-schedule-strip";
import {
  faBoxesStacked,
  faCircleCheck,
  faFolderTree,
  faRocket,
} from "@fortawesome/free-solid-svg-icons";

const DASHBOARD_REFRESH_INTERVAL_MS = 10_000;
const USAGE_REFRESH_INTERVAL_MS = 4_000;

function Dashboard() {
  const visible = usePageVisible();
  const { data, isPending, error } = useQuery({
    ...dashboardQueries.overview(),
    refetchInterval: visible ? DASHBOARD_REFRESH_INTERVAL_MS : false,
  });
  const liveJobs = data?.liveJobs ?? [];
  const hasRunning = liveJobs.some((j) => j.job.status === "running");
  const hasLiveJobs = liveJobs.length > 0;

  // Live CPU/MEM samples for the in-flight panel — only polled while builds
  // are actually running and the tab is visible. Queued jobs have no
  // container yet, so there is nothing to sample.
  const usageQuery = useQuery({
    ...jobsQueries.usageList(),
    enabled: hasRunning,
    refetchInterval: visible && hasRunning ? USAGE_REFRESH_INTERVAL_MS : false,
  });
  const usageByJob = new Map<string, JobResourceUsageSample>(
    (usageQuery.data?.samples ?? []).map((s) => [s.job_id, s]),
  );

  // Ticks queued and running elapsed counters once a second while work exists.
  const [now, setNow] = useState(() => Date.now());
  useEffect(() => {
    if (!visible || !hasLiveJobs) return;
    const id = window.setInterval(() => setNow(Date.now()), 1000);
    return () => window.clearInterval(id);
  }, [visible, hasLiveJobs]);

  if (error) {
    return (
      <ErrorMessage
        message={error instanceof Error ? error.message : "Failed to load dashboard"}
      />
    );
  }

  const loading = isPending;
  const jobs = data?.jobs ?? [];
  const queued = liveJobs.filter((j) => j.job.status === "pending").length;
  const building = liveJobs.filter((j) => j.job.status === "running").length;

  return (
    <div className="space-y-6">
      <PageHeader
        title="Dashboard"
        description="High-signal snapshot of package state, active builds, and execution history."
        color="lime"
      />

      {loading ? (
        <LoadingBlock label="Loading metrics…" lines={2} />
      ) : (
        <section className="grid grid-cols-2 gap-3 sm:gap-4 xl:grid-cols-4">
          <MetricCard
            label="Packages"
            value={data?.packageCount ?? 0}
            detail="Registered sources"
            icon={<FaIcon icon={faBoxesStacked} />}
          />
          <MetricCard
            label="Enabled"
            value={data?.enabledPackageCount ?? 0}
            detail="Actively buildable"
            icon={<FaIcon icon={faCircleCheck} />}
          />
          <MetricCard
            label="Active jobs"
            value={data?.activeJobCount ?? 0}
            detail="Pending or running"
            live={(data?.activeJobCount ?? 0) > 0}
            icon={<FaIcon icon={faRocket} />}
          />
          <MetricCard
            label="Stored"
            value={formatBytes(data?.repoSummary.stored_bytes ?? 0)}
            detail="Published repository data"
            icon={<FaIcon icon={faFolderTree} />}
          />
        </section>
      )}

      <ActiveBuildsSection
        loading={loading}
        jobs={liveJobs}
        usageByJob={usageByJob}
        now={now}
      />

      {!loading && (
        <PipelineStrip
          queued={queued}
          building={building}
          recentDone={jobs.length}
        />
      )}

      <SyncScheduleStrip />

      <section className="border border-edge bg-black">
        <div className="flex items-center justify-between gap-4 border-b border-edge px-[18px] py-[15px]">
          <h2 className="font-mono text-[13px] font-bold uppercase tracking-[0.06em] text-white">
            Latest build runs
          </h2>
          <Link
            to="/jobs"
            className="font-mono text-xs font-bold uppercase tracking-[0.08em] text-accent-lime transition-all duration-100 ease-linear hover:underline"
          >
            View All →
          </Link>
        </div>

        {loading ? (
          <div className="p-[18px]">
            <LoadingBlock label="Loading recent builds…" lines={3} />
          </div>
        ) : jobs.length === 0 ? (
          <div className="p-[18px]">
            <div className="flex min-h-[160px] items-center justify-center border border-dashed border-edge px-6 py-8">
              <div className="font-mono text-sm text-[#52525b]">
                No jobs have run yet.
              </div>
            </div>
          </div>
        ) : (
          <div>
            {jobs.map((entry, i) => (
              <BuildRunRow
                key={entry.job.id}
                entry={entry}
                last={i === jobs.length - 1}
              />
            ))}
          </div>
        )}
      </section>
    </div>
  );
}

export default function DashboardPage() {
  return <Dashboard />;
}

interface PipelineStripProps {
  queued: number;
  building: number;
  recentDone: number;
}

/**
 * ASCII-style flow:  [ QUEUED ] ─→ [ BUILDING ] ─→ RECENT DONE  view all →
 * One neutral container; each stage stacks a bracketed mono label over a
 * big tabular count. The building lane carries the brand accent. Arrows are
 * mono glyphs between stages on sm:+; below sm the stages stack.
 */
function PipelineStrip({ queued, building, recentDone }: PipelineStripProps) {
  return (
    <section
      aria-label="Build pipeline"
      className="grid grid-cols-1 border border-edge bg-black sm:grid-cols-[1fr_auto_1fr_auto_1.4fr]"
    >
      <PipelineStage label="Queued" count={queued} />
      <PipelineArrow />
      <PipelineStage label="Building" count={building} accent />
      <PipelineArrow />
      <Link
        to="/jobs"
        className="group flex items-center justify-between gap-4 border-t border-edge px-5 py-[18px] transition-colors hover:bg-surface-hover sm:border-t-0"
      >
        <div>
          <div className="font-mono text-xs font-bold uppercase tracking-[0.2em] text-soft">
            Recent done
          </div>
          <div className="mt-3.5 font-mono text-3xl font-extrabold leading-none tabular-nums text-white">
            {recentDone}
          </div>
        </div>
        <span className="font-mono text-xs font-bold uppercase tracking-[0.08em] text-soft transition-colors group-hover:text-accent-lime">
          View all →
        </span>
      </Link>
    </section>
  );
}

function PipelineStage({
  label,
  count,
  accent = false,
}: {
  label: string;
  count: number;
  accent?: boolean;
}) {
  const live = accent && count > 0;
  return (
    <div
      className={`px-5 py-[18px] ${accent ? "border-t border-edge sm:border-x sm:border-t-0 sm:border-[#161618]" : ""}`}
    >
      <div
        className={`font-mono text-xs font-bold uppercase tracking-[0.2em] ${live ? "text-accent-lime" : "text-soft"}`}
      >
        [ {label} ]
      </div>
      <div
        className={`mt-3.5 font-mono text-3xl font-extrabold leading-none tabular-nums ${live ? "text-accent-lime" : "text-white"}`}
      >
        {count}
      </div>
    </div>
  );
}

function PipelineArrow() {
  return (
    <span
      aria-hidden="true"
      className="hidden items-center justify-center px-1.5 font-mono text-lg text-edge-strong sm:flex"
    >
      →
    </span>
  );
}
