import { useQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { dashboardQueries, statisticsQueries } from "../../lib/queries";
import { formatBytes } from "../../lib/bytes";
import ErrorMessage from "../../components/common/error-message";
import { usePageVisible } from "../../components/common/page-visibility-provider";
import FaIcon from "../../components/ui/fa-icon";
import LoadingBlock from "../../components/ui/loading-block";
import MetricCard from "../../components/ui/metric-card";
import PageHeader from "../../components/ui/page-header";
import MiniJobRow from "./mini-job-row";
import SyncScheduleStrip from "./sync-schedule-strip";
import {
  faBoxesStacked,
  faChartLine,
  faCircleCheck,
  faFolderTree,
  faRocket,
} from "@fortawesome/free-solid-svg-icons";

const DASHBOARD_REFRESH_INTERVAL_MS = 10_000;

function Dashboard() {
  const visible = usePageVisible();
  const { data, isPending, error } = useQuery({
    ...dashboardQueries.overview(),
    refetchInterval: visible ? DASHBOARD_REFRESH_INTERVAL_MS : false,
  });
  const timeseriesQuery = useQuery({
    ...statisticsQueries.timeseries("24h"),
    refetchInterval: visible ? 60_000 : false,
  });
  const jobsSparkline = (timeseriesQuery.data?.jobs.points ?? []).map(
    (p) => p.succeeded + p.failed,
  );
  const syncSparkline = (timeseriesQuery.data?.sync.points ?? []).map(
    (p) => p.succeeded + p.failed,
  );

  if (error) {
    return (
      <ErrorMessage
        message={error instanceof Error ? error.message : "Failed to load dashboard"}
      />
    );
  }

  const loading = isPending;
  const jobs = data?.jobs ?? [];
  const liveJobs = data?.liveJobs ?? [];
  const queued = liveJobs.filter((j) => j.job.status === "pending").length;
  const building = liveJobs.filter((j) => j.job.status === "running").length;

  return (
    <div className="space-y-8">
      <PageHeader
        title="Dashboard"
        description="High-signal snapshot of package state, active builds, and execution history."
        color="cyan"
        actions={[
          { to: "/packages", label: "Packages", icon: faBoxesStacked },
          { to: "/statistics", label: "Statistics", icon: faChartLine },
          { to: "/jobs", label: "Open Jobs", icon: faChartLine, variant: "primary" },
        ]}
      />

      {loading ? (
        <LoadingBlock label="Loading metrics…" lines={2} />
      ) : (
        <section className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
          <MetricCard
            label="Packages"
            value={data?.packageCount ?? 0}
            detail="Registered sources"
            icon={<FaIcon icon={faBoxesStacked} />}
            sparkline={syncSparkline.length > 0 ? syncSparkline : null}
          />
          <MetricCard
            label="Enabled"
            value={data?.enabledPackageCount ?? 0}
            detail="Actively buildable"
            variant="accent"
            icon={<FaIcon icon={faCircleCheck} />}
          />
          <MetricCard
            label="Active jobs"
            value={data?.activeJobCount ?? 0}
            detail="Pending or running"
            variant="terminal"
            icon={<FaIcon icon={faRocket} />}
            sparkline={jobsSparkline.length > 0 ? jobsSparkline : null}
          />
          <MetricCard
            label="Stored"
            value={formatBytes(data?.repoSummary.stored_bytes ?? 0)}
            detail="Published repository data"
            icon={<FaIcon icon={faFolderTree} />}
          />
        </section>
      )}

      <section className="border-2 border-edge-strong bg-black shadow-card-sm">
        <div className="flex items-baseline justify-between gap-4 border-b-2 border-edge-strong app-section-band px-5 py-4">
          <h2 className="text-xl font-bold text-white">Latest build runs</h2>
          <Link
            to="/jobs"
            className="font-mono text-sm font-semibold text-accent-lime transition-all duration-100 ease-linear hover:underline"
          >
            View All →
          </Link>
        </div>

        <div className="p-5">
          {loading ? (
            <LoadingBlock label="Loading recent builds…" lines={3} />
          ) : jobs.length === 0 ? (
            <div className="flex min-h-[200px] items-center justify-center border-2 border-dashed border-edge bg-surface-alt/30 px-6 py-8">
              <div className="text-center">
                <div className="font-mono text-sm text-soft">
                  No jobs have run yet.
                </div>
              </div>
            </div>
          ) : (
            <div className="space-y-2">
              {jobs.map((entry) => (
                <MiniJobRow key={entry.job.id} entry={entry} />
              ))}
            </div>
          )}
        </div>
      </section>

      {!loading && (
        <PipelineStrip
          queued={queued}
          building={building}
          recentDone={jobs.length}
        />
      )}

      <SyncScheduleStrip />

      <section className="border-2 border-success bg-black shadow-card-sm">
        <div className="flex items-baseline justify-between gap-4 border-b-2 border-success bg-black px-5 py-4">
          <div className="flex items-center gap-2">
            <span className="relative flex h-2 w-2">
              <span className="absolute inline-flex h-full w-full animate-ping bg-success opacity-75" />
              <span className="relative inline-flex h-2 w-2 bg-success" />
            </span>
            <h2 className="text-xl font-bold text-white">Builds in flight</h2>
          </div>
          {!loading && (
            <span className="font-mono text-xs uppercase tracking-[0.18em] text-soft">
              {liveJobs.length} active
            </span>
          )}
        </div>
        <div className="p-5">
          {loading ? (
            <LoadingBlock label="Loading active builds…" lines={2} />
          ) : liveJobs.length === 0 ? (
            <div className="flex min-h-[140px] items-center justify-center border-2 border-dashed border-edge bg-surface-alt/30 px-6 py-8">
              <div className="font-mono text-sm text-soft">
                Nothing is building right now.
              </div>
            </div>
          ) : (
            <div className="space-y-2">
              {liveJobs.map((entry) => (
                <MiniJobRow key={entry.job.id} entry={entry} />
              ))}
            </div>
          )}
        </div>
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

function PipelineStrip({ queued, building, recentDone }: PipelineStripProps) {
  return (
    <section
      aria-label="Build pipeline"
      className="border-2 border-edge-strong bg-black px-4 py-3 sm:px-5"
    >
      <div className="flex flex-col items-stretch gap-3 md:flex-row md:items-center md:gap-4">
        <PipelineStage
          label="Queued"
          count={queued}
          tone={queued > 0 ? "orange" : "muted"}
        />
        <PipelineConnector />
        <PipelineStage
          label="Building"
          count={building}
          tone={building > 0 ? "lime" : "muted"}
          live={building > 0}
        />
        <PipelineConnector />
        <Link
          to="/jobs"
          className="group flex flex-1 items-center justify-between border-2 border-edge bg-surface-alt/40 px-4 py-3 transition-colors hover:border-success hover:bg-surface-alt"
        >
          <div className="flex items-center gap-3">
            <span className="font-mono text-[10px] font-bold uppercase tracking-[0.22em] text-soft">
              Recent done
            </span>
            <span className="font-display text-2xl font-black tabular-nums text-white">
              {recentDone}
            </span>
          </div>
          <span className="font-mono text-xs uppercase tracking-[0.18em] text-soft transition-colors group-hover:text-success">
            view all →
          </span>
        </Link>
      </div>
    </section>
  );
}

function PipelineStage({
  label,
  count,
  tone,
  live = false,
}: {
  label: string;
  count: number;
  tone: "orange" | "lime" | "muted";
  live?: boolean;
}) {
  const toneClasses =
    tone === "orange"
      ? "border-accent-orange text-accent-orange"
      : tone === "lime"
        ? "border-accent-lime text-accent-lime"
        : "border-edge-strong text-soft";
  return (
    <div
      className={`flex flex-1 items-center justify-between border-2 bg-black px-4 py-3 ${toneClasses}`}
    >
      <span className="font-mono text-[10px] font-bold uppercase tracking-[0.22em]">
        [ {label} ]
      </span>
      <span className="flex items-center gap-2">
        {live ? (
          <span
            aria-hidden="true"
            className="inline-block h-2 w-2 animate-pulse bg-accent-lime"
          />
        ) : null}
        <span className="font-display text-2xl font-black tabular-nums">
          {count}
        </span>
      </span>
    </div>
  );
}

function PipelineConnector() {
  return (
    <span
      aria-hidden="true"
      className="flex shrink-0 items-center justify-center font-mono text-soft md:text-base"
    >
      <span className="hidden md:inline">──→</span>
      <span className="md:hidden">↓</span>
    </span>
  );
}
