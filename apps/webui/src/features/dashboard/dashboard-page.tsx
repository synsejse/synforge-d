import { useQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { dashboardQueries } from "../../lib/queries";
import { formatBytes } from "../../lib/bytes";
import { formatDateTime } from "../../lib/datetime";
import ErrorMessage from "../../components/common/error-message";
import { usePageVisible } from "../../components/common/page-visibility-provider";
import LoadingBlock from "../../components/ui/loading-block";
import FaIcon from "../../components/ui/fa-icon";
import MetricCard from "../../components/ui/metric-card";
import Badge from "../../components/ui/badge";
import PageHeader from "../../components/ui/page-header";
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

  if (isPending) {
    return <LoadingBlock label="Loading overview…" lines={4} />;
  }

  if (error) {
    return (
      <ErrorMessage
        message={error instanceof Error ? error.message : "Failed to load dashboard"}
      />
    );
  }

  const getStatusVariant = (status: string) => {
    if (status === "succeeded") return "success";
    if (status === "failed" || status === "timed_out") return "error";
    if (status === "running") return "lime";
    if (status === "pending") return "warning";
    return "default";
  };

  return (
    <div className="space-y-8">
      {/* Hero Header */}
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

      {/* Metrics Grid */}
      <section className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
        <MetricCard
          label="Packages"
          value={data.packageCount}
          detail="Registered sources"
          icon={<FaIcon icon={faBoxesStacked} />}
        />
        <MetricCard
          label="Enabled"
          value={data.enabledPackageCount}
          detail="Actively buildable"
          variant="accent"
          icon={<FaIcon icon={faCircleCheck} />}
        />
        <MetricCard
          label="Active_Jobs"
          value={data.activeJobCount}
          detail="Pending or running"
          variant="terminal"
          icon={<FaIcon icon={faRocket} />}
        />
        <MetricCard
          label="Stored"
          value={formatBytes(data.repoSummary.stored_bytes)}
          detail="Published repository data"
          icon={<FaIcon icon={faFolderTree} />}
        />
      </section>

      {/* Recent Jobs Table */}
      <section className="border-2 border-edge-strong bg-black shadow-card-sm">
        <div className="flex items-end justify-between gap-4 border-b-2 border-edge-strong app-section-band px-5 py-4">
          <h2 className="text-xl font-bold text-white">
            Latest build runs
          </h2>
          <Link
            to="/jobs"
            className="font-mono text-sm font-semibold text-accent-lime transition-all duration-100 ease-linear hover:underline"
          >
            View All →
          </Link>
        </div>

        <div className="p-5">
          {data.jobs.length === 0 ? (
            <div className="flex min-h-[200px] items-center justify-center border-2 border-dashed border-edge bg-surface-alt/30 px-6 py-8">
              <div className="text-center">
                <div className="font-mono text-sm text-soft">
                  No jobs have run yet.
                </div>
              </div>
            </div>
          ) : (
            <div className="grid gap-2">
              {data.jobs.map((entry) => (
                <Link
                  key={entry.job.id}
                  to="/jobs/view"
                  search={{ id: entry.job.id }}
                  className="grid gap-4 border-2 border-edge bg-surface-alt/40 p-5 transition-all duration-100 hover:translate-x-[-1px] hover:translate-y-[-1px] hover:border-edge-strong hover:bg-surface-alt hover:shadow-[3px_3px_0_rgba(255,255,255,0.08)] md:grid-cols-[minmax(0,220px)_minmax(0,130px)_minmax(0,1fr)_auto]"
                >
                  <div className="min-w-0">
                    <div className="font-display text-base font-bold text-white">
                      {entry.job.package_name}
                    </div>
                    <div className="mt-1 truncate font-mono text-xs text-soft">
                      {entry.job.id}
                    </div>
                  </div>
                  <div className="flex items-start">
                    <Badge variant="ghost">
                      {entry.job.mock_chroot}
                    </Badge>
                  </div>
                  <div className="min-w-0">
                    <div className="truncate font-mono text-sm text-muted">
                      {entry.job.revision}
                    </div>
                    <div className="mt-1 font-mono text-xs text-soft">
                      {formatDateTime(entry.job.created_at)}
                    </div>
                  </div>
                  <div className="flex items-start justify-start md:justify-end">
                    <Badge variant={getStatusVariant(entry.job.status)} pulse={entry.job.status === "running"}>
                      {entry.job.status}
                    </Badge>
                  </div>
                </Link>
              ))}
            </div>
          )}
        </div>
      </section>

      {/* Live Queue — full width */}
      <section className="border-2 border-success bg-black shadow-card-sm">
        <div className="flex items-end justify-between gap-4 border-b-2 border-success bg-black px-5 py-4">
          <div className="flex items-center gap-2">
            <span className="relative flex h-2 w-2">
              <span className="absolute inline-flex h-full w-full animate-ping bg-success opacity-75" />
              <span className="relative inline-flex h-2 w-2 bg-success" />
            </span>
            <h2 className="text-xl font-bold text-white">
              Builds in flight
            </h2>
          </div>
          <span className="font-mono text-xs uppercase tracking-[0.18em] text-soft">
            {data.liveJobs.length} active
          </span>
        </div>
        <div className="p-5">
          {data.liveJobs.length === 0 ? (
            <div className="flex min-h-[140px] items-center justify-center border-2 border-dashed border-edge bg-surface-alt/30 px-6 py-8">
              <div className="font-mono text-sm text-soft">
                Nothing is building right now.
              </div>
            </div>
          ) : (
            <div className="grid gap-3 md:grid-cols-2">
              {data.liveJobs.map((entry) => (
                <Link
                  key={entry.job.id}
                  to="/jobs/view"
                  search={{ id: entry.job.id }}
                  className="group relative block overflow-hidden border-2 border-edge bg-surface-alt/40 px-4 py-3 transition-all duration-100 ease-linear hover:border-success hover:bg-surface-alt"
                >
                  <span
                    aria-hidden="true"
                    className="absolute inset-y-0 left-0 w-1 bg-success opacity-0 transition-opacity duration-100 group-hover:opacity-100"
                  />
                  <div className="flex flex-col gap-2 md:flex-row md:items-center md:justify-between">
                    <div className="min-w-0">
                      <div className="flex flex-wrap items-center gap-2">
                        <div className="font-display font-bold text-white">
                          {entry.job.package_name}
                        </div>
                        <Badge variant="ghost">{entry.job.mock_chroot}</Badge>
                      </div>
                      <div className="mt-1 truncate font-mono text-xs text-soft">
                        {entry.job.revision}
                      </div>
                    </div>
                    <div className="flex md:justify-end">
                      <Badge variant={getStatusVariant(entry.job.status)} pulse>
                        {entry.job.status}
                      </Badge>
                    </div>
                  </div>
                </Link>
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
