import { useEffect, useState } from "react";
import api from "../../lib/api";
import { formatBytes } from "../../lib/bytes";
import { formatDateTime } from "../../lib/datetime";
import type {
  BuildJobResponse,
  RepoSummaryResponse,
} from "../../lib/types";
import ErrorMessage from "../common/ErrorMessage";
import LoadingBlock from "../ui/LoadingBlock";
import FaIcon from "../ui/FaIcon";
import MetricCard from "../ui/MetricCard";
import Badge from "../ui/Badge";
import PageHeader from "../ui/PageHeader";
import {
  faBoxesStacked,
  faChartLine,
  faCircleCheck,
  faFolderTree,
  faRocket,
} from "@fortawesome/free-solid-svg-icons";

const EMPTY_REPO_SUMMARY: RepoSummaryResponse = {
  package_count: 0,
  target_count: 0,
  build_count: 0,
  published_file_count: 0,
  stored_bytes: 0,
  recent_files: [],
  targets: [],
};

export default function Dashboard() {
  const [jobs, setJobs] = useState<BuildJobResponse[]>([]);
  const [liveJobs, setLiveJobs] = useState<BuildJobResponse[]>([]);
  const [repoSummary, setRepoSummary] = useState<RepoSummaryResponse>(
    EMPTY_REPO_SUMMARY,
  );
  const [packageCount, setPackageCount] = useState(0);
  const [enabledPackageCount, setEnabledPackageCount] = useState(0);
  const [activeJobCount, setActiveJobCount] = useState(0);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    async function load() {
      try {
        const [
          packagesRes,
          enabledPackagesRes,
          recentJobsRes,
          activeJobsRes,
          repositoryRes,
        ] = await Promise.all([
          api.listPackagesPage(1, 0),
          api.listPackagesPage(1, 0, { enabled: true }),
          api.listCompletedJobs({ limit: 6, offset: 0 }),
          api.listActiveJobs({ limit: 6, offset: 0 }),
          api.getRepoSummary(),
        ]);
        setPackageCount(packagesRes.page.total ?? packagesRes.packages.length);
        setEnabledPackageCount(
          enabledPackagesRes.page.total ?? enabledPackagesRes.packages.length,
        );
        setJobs(recentJobsRes.jobs);
        setActiveJobCount(activeJobsRes.page.total ?? activeJobsRes.jobs.length);
        setLiveJobs(activeJobsRes.jobs);
        setRepoSummary(repositoryRes);
        setError(null);
      } catch (e) {
        setError(e instanceof Error ? e.message : "Failed to load dashboard");
      } finally {
        setLoading(false);
      }
    }

    load();
  }, []);

  if (loading) {
    return <LoadingBlock label="Loading overview…" lines={4} />;
  }

  if (error) {
    return <ErrorMessage message={error} />;
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
        eyebrow="SYSTEM_OVERVIEW"
        title="Dashboard"
        description="High-signal snapshot of package state, active builds, and execution history."
        color="cyan"
        actions={[
          { href: "/packages/", label: "Packages", icon: faBoxesStacked },
          { href: "/statistics/", label: "Statistics", icon: faChartLine },
          { href: "/jobs/", label: "Open Jobs", icon: faChartLine, variant: "primary" },
        ]}
      />

      {/* Metrics Grid */}
      <section className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
        <MetricCard
          label="Packages"
          value={packageCount}
          detail="Registered sources"
          icon={<FaIcon icon={faBoxesStacked} />}
        />
        <MetricCard
          label="Enabled"
          value={enabledPackageCount}
          detail="Actively buildable"
          variant="accent"
          icon={<FaIcon icon={faCircleCheck} />}
        />
        <MetricCard
          label="Active_Jobs"
          value={activeJobCount}
          detail="Pending or running"
          variant="terminal"
          icon={<FaIcon icon={faRocket} />}
        />
        <MetricCard
          label="Stored"
          value={formatBytes(repoSummary.stored_bytes)}
          detail="Published repository data"
          icon={<FaIcon icon={faFolderTree} />}
        />
      </section>

      {/* Recent Jobs Table */}
      <section className="border-4 border-[var(--theme-border-strong)] bg-black shadow-[4px_4px_0_rgba(255,255,255,0.1)]">
        <div className="flex items-end justify-between gap-4 border-b-4 border-[var(--theme-border-strong)] bg-gradient-to-r from-zinc-900 to-black px-6 py-5">
          <div>
            <div className="font-mono text-xs font-bold uppercase tracking-[0.25em] text-zinc-500">
              Recent_Jobs
            </div>
            <h2 className="font-display mt-2 text-2xl font-bold uppercase tracking-tight text-white">
              Latest_Build_Runs
            </h2>
          </div>
          <a
            href="/jobs/"
            className="font-mono text-sm font-semibold text-[var(--theme-accent-lime)] transition-all duration-100 ease-linear hover:underline"
          >
            View_All →
          </a>
        </div>

        <div className="p-6">
          {jobs.length === 0 ? (
            <div className="flex min-h-[200px] items-center justify-center border-2 border-dashed border-[var(--theme-border)] bg-zinc-950/30 px-6 py-8">
              <div className="text-center">
                <div className="font-mono text-sm text-zinc-500">
                  No jobs have run yet.
                </div>
              </div>
            </div>
          ) : (
            <div className="grid gap-2">
              {jobs.map((entry) => (
                <a
                  key={entry.job.id}
                  href={`/jobs/view/?id=${encodeURIComponent(entry.job.id)}`}
                  className="grid gap-4 border-2 border-[var(--theme-border)] bg-zinc-950/40 p-5 transition-all duration-100 hover:translate-x-[-1px] hover:translate-y-[-1px] hover:border-[var(--theme-border-strong)] hover:bg-zinc-950 hover:shadow-[3px_3px_0_rgba(255,255,255,0.08)] md:grid-cols-[minmax(0,220px)_minmax(0,130px)_minmax(0,1fr)_auto]"
                >
                  <div className="min-w-0">
                    <div className="font-display text-base font-bold text-white">
                      {entry.job.package_name}
                    </div>
                    <div className="mt-1 truncate font-mono text-xs text-zinc-600">
                      {entry.job.id}
                    </div>
                  </div>
                  <div className="flex items-start">
                    <Badge variant="ghost">
                      {entry.job.mock_chroot}
                    </Badge>
                  </div>
                  <div className="min-w-0">
                    <div className="truncate font-mono text-sm text-zinc-300">
                      {entry.job.revision}
                    </div>
                    <div className="mt-1 font-mono text-xs text-zinc-600">
                      {formatDateTime(entry.job.created_at)}
                    </div>
                  </div>
                  <div className="flex items-start justify-start md:justify-end">
                    <Badge variant={getStatusVariant(entry.job.status)} pulse={entry.job.status === "running"}>
                      {entry.job.status}
                    </Badge>
                  </div>
                </a>
              ))}
            </div>
          )}
        </div>
      </section>

      {/* Two Column Layout */}
      <section className="grid gap-6 xl:grid-cols-2">
        {/* Live Queue */}
        <article className="flex h-full flex-col border-4 border-[var(--theme-terminal-green)] bg-black shadow-[4px_4px_0_rgba(0,255,65,0.2)]">
          <div className="border-b-4 border-[var(--theme-terminal-green)] bg-black px-6 py-5">
            <div className="flex items-center gap-2 font-mono text-xs font-bold uppercase tracking-[0.25em] text-[var(--theme-terminal-green)]">
              <span className="relative flex h-2 w-2">
                <span className="absolute inline-flex h-full w-full animate-ping bg-[var(--theme-terminal-green)] opacity-75"></span>
                <span className="relative inline-flex h-2 w-2 bg-[var(--theme-terminal-green)]"></span>
              </span>
              Live_Queue
            </div>
            <h2 className="font-display mt-2 text-xl font-bold uppercase tracking-tight text-white">
              Builds_In_Flight
            </h2>
          </div>

          <div className="flex-1 p-6">
            {liveJobs.length === 0 ? (
              <div className="flex min-h-[240px] items-center justify-center border-2 border-dashed border-[var(--theme-border)] bg-zinc-950/30 px-6 py-8">
                <div className="font-mono text-sm text-zinc-500">
                  Nothing is building right now.
                </div>
              </div>
            ) : (
              <div className="grid gap-3">
                {liveJobs.map((entry) => (
                  <a
                    key={entry.job.id}
                    href={`/jobs/view/?id=${encodeURIComponent(entry.job.id)}`}
                    className="block border-2 border-[var(--theme-border)] bg-zinc-950/40 px-5 py-4 transition-all duration-100 ease-linear hover:border-[var(--theme-terminal-green)] hover:bg-zinc-950"
                  >
                    <div className="flex flex-col gap-3 md:flex-row md:items-start md:justify-between">
                      <div className="min-w-0">
                        <div className="flex flex-wrap items-center gap-2">
                          <div className="font-display font-bold text-white">
                            {entry.job.package_name}
                          </div>
                          <Badge variant="ghost">
                            {entry.job.mock_chroot}
                          </Badge>
                        </div>
                        <div className="mt-1 truncate font-mono text-xs text-zinc-500">
                          {entry.job.revision}
                        </div>
                      </div>
                      <div className="flex md:justify-end">
                        <Badge variant={getStatusVariant(entry.job.status)} pulse>
                          {entry.job.status}
                        </Badge>
                      </div>
                    </div>
                  </a>
                ))}
              </div>
            )}
          </div>
        </article>

        {/* Repository Snapshot */}
        <article className="border-4 border-[var(--theme-border-strong)] bg-black shadow-[4px_4px_0_rgba(255,255,255,0.1)]">
          <div className="border-b-4 border-[var(--theme-border-strong)] bg-gradient-to-r from-zinc-900 to-black px-6 py-5">
            <div className="flex items-center gap-2 font-mono text-xs font-bold uppercase tracking-[0.25em] text-zinc-500">
              <FaIcon icon={faFolderTree} />
              Published_Repository
            </div>
            <h2 className="font-display mt-2 text-xl font-bold uppercase tracking-tight text-white">
              Repository_Snapshot
            </h2>
          </div>

          <div className="p-6">
            <div className="grid gap-4 sm:grid-cols-2">
              <div className="border-l-4 border-[var(--theme-accent-lime)] bg-zinc-950/30 pl-4 pr-3 py-4">
                <div className="font-mono text-xs font-bold uppercase tracking-wider text-zinc-500">
                  Packages
                </div>
                <div className="font-display mt-2 text-3xl font-black text-white">
                  {repoSummary.package_count}
                </div>
              </div>
              <div className="border-l-4 border-[var(--theme-accent-lime)] bg-zinc-950/30 pl-4 pr-3 py-4">
                <div className="font-mono text-xs font-bold uppercase tracking-wider text-zinc-500">
                  Targets
                </div>
                <div className="font-display mt-2 text-3xl font-black text-white">
                  {repoSummary.target_count}
                </div>
              </div>
              <div className="border-l-4 border-zinc-700 bg-zinc-950/30 pl-4 pr-3 py-4">
                <div className="font-mono text-xs font-bold uppercase tracking-wider text-zinc-500">
                  Builds
                </div>
                <div className="font-display mt-2 text-3xl font-black text-white">
                  {repoSummary.build_count}
                </div>
              </div>
              <div className="border-l-4 border-zinc-700 bg-zinc-950/30 pl-4 pr-3 py-4">
                <div className="font-mono text-xs font-bold uppercase tracking-wider text-zinc-500">
                  Files
                </div>
                <div className="font-display mt-2 text-3xl font-black text-white">
                  {repoSummary.published_file_count}
                </div>
              </div>
            </div>
          </div>
        </article>
      </section>
    </div>
  );
}
