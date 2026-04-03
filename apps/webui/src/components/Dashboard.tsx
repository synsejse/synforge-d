import { useEffect, useState } from "react";
import api from "../lib/api";
import { formatDateTime } from "../lib/datetime";
import type { BuildJobResponse, RepoSummaryResponse } from "../lib/types";
import EmptyState from "./EmptyState";
import FaIcon from "./FaIcon";
import LoadingBlock from "./LoadingBlock";
import MetricCard from "./MetricCard";
import PageHeader from "./PageHeader";
import StatusPill from "./StatusPill";
import {
  faBoxesStacked,
  faChartLine,
  faFolderTree,
} from "@fortawesome/free-solid-svg-icons";

export default function Dashboard() {
  const [jobs, setJobs] = useState<BuildJobResponse[]>([]);
  const [liveJobs, setLiveJobs] = useState<BuildJobResponse[]>([]);
  const [repoSummary, setRepoSummary] = useState<RepoSummaryResponse | null>(
    null,
  );
  const [packageCount, setPackageCount] = useState(0);
  const [enabledPackageCount, setEnabledPackageCount] = useState(0);
  const [activeJobCount, setActiveJobCount] = useState(0);
  const [jobCount, setJobCount] = useState(0);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    async function load() {
      try {
        const [
          packagesRes,
          enabledPackagesRes,
          recentJobsRes,
          pendingJobsRes,
          runningJobsRes,
          repositoryRes,
        ] = await Promise.all([
          api.listPackagesPage(1, 0),
          api.listPackagesPage(1, 0, { enabled: true }),
          api.listJobs({ limit: 6, offset: 0 }),
          api.listJobs({ limit: 4, offset: 0, status: "pending" }),
          api.listJobs({ limit: 4, offset: 0, status: "running" }),
          api.getRepoSummary(),
        ]);
        setPackageCount(packagesRes.page.total ?? packagesRes.packages.length);
        setEnabledPackageCount(
          enabledPackagesRes.page.total ?? enabledPackagesRes.packages.length,
        );
        setJobs(recentJobsRes.jobs);
        setJobCount(recentJobsRes.page.total ?? recentJobsRes.jobs.length);
        setActiveJobCount(
          (pendingJobsRes.page.total ?? pendingJobsRes.jobs.length) +
            (runningJobsRes.page.total ?? runningJobsRes.jobs.length),
        );
        setLiveJobs(
          [...runningJobsRes.jobs, ...pendingJobsRes.jobs].slice(0, 6),
        );
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
    return (
      <div className="border border-zinc-800 bg-black p-4 text-zinc-200">
        Error: {error}
      </div>
    );
  }

  return (
    <div className="space-y-8">
      <PageHeader
        eyebrow="System Overview"
        title="Operations at a glance"
        description="A high-signal snapshot of package state, active builds, and recent execution history."
        actions={[
          {
            href: "/packages/",
            label: "Manage Packages",
            icon: faBoxesStacked,
          },
          {
            href: "/jobs/",
            label: "Open Jobs",
            icon: faChartLine,
            variant: "primary",
          },
        ]}
      />

      <section className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
        <MetricCard
          label="Packages"
          value={packageCount}
          detail="Registered sources"
        />
        <MetricCard
          label="Enabled"
          value={enabledPackageCount}
          detail="Actively buildable"
        />
        <MetricCard
          label="Active Jobs"
          value={activeJobCount}
          detail="Pending or running"
        />
        <MetricCard
          label="Stored Size"
          value={formatBytes(repoSummary?.stored_bytes ?? 0)}
          detail="Published repository data"
        />
      </section>

      <section className="border border-zinc-800 bg-black p-6">
        <div className="flex items-end justify-between gap-4">
          <div>
            <div className="text-xs uppercase tracking-[0.2em] text-zinc-500">
              Recent Jobs
            </div>
            <h2 className="mt-2 text-2xl font-semibold text-white">
              Latest build runs
            </h2>
          </div>
          <a
            href="/jobs/"
            className="text-sm text-zinc-300 transition hover:text-white"
          >
            View all →
          </a>
        </div>

        {jobs.length === 0 ? (
          <div className="mt-5">
            <EmptyState>No jobs have run yet.</EmptyState>
          </div>
        ) : (
          <div className="mt-5 grid gap-3">
            {jobs.map((entry) => (
              <a
                key={entry.job.id}
                href={`/jobs/view/?id=${encodeURIComponent(entry.job.id)}`}
                className="grid gap-3 border border-zinc-800 bg-zinc-950/40 p-4 transition hover:bg-zinc-950 md:grid-cols-[minmax(0,210px)_minmax(0,120px)_minmax(0,1fr)_auto]"
              >
                <div className="min-w-0">
                  <div className="font-medium text-white">
                    {entry.job.package_name}
                  </div>
                  <div className="mt-1 truncate text-xs text-zinc-500">
                    {entry.job.id}
                  </div>
                </div>
                <div className="flex items-start">
                  <span className="inline-flex items-center border border-zinc-800 bg-black px-2.5 py-1 font-mono text-xs text-zinc-300">
                    {entry.job.mock_chroot}
                  </span>
                </div>
                <div className="min-w-0">
                  <div className="truncate font-mono text-sm text-zinc-300">
                    {entry.job.revision}
                  </div>
                  <div className="mt-1 text-xs text-zinc-500">
                    {formatDateTime(entry.job.created_at)}
                  </div>
                </div>
                <div className="flex items-start justify-start md:justify-end">
                  <StatusPill status={entry.job.status} />
                </div>
              </a>
            ))}
          </div>
        )}
      </section>

      <section className="grid gap-6 xl:grid-cols-2">
        <article className="flex h-full flex-col border border-zinc-800 bg-black p-6">
          <div className="flex items-center gap-2 text-xs uppercase tracking-[0.22em] text-zinc-500">
            <span className="inline-block h-2 w-2 bg-emerald-400"></span>
            Live Queue
          </div>
          <h2 className="mt-3 text-2xl font-semibold text-white">
            Builds in flight
          </h2>
          <p className="mt-2 text-sm leading-6 text-zinc-400">
            Pending and running jobs currently in the queue.
          </p>

          {liveJobs.length === 0 ? (
            <div className="mt-5 flex flex-1">
              <div className="flex min-h-[220px] flex-1 items-center justify-center border border-dashed border-zinc-800 bg-zinc-950/30 px-6 py-8 text-center text-zinc-400">
                Nothing is building right now.
              </div>
            </div>
          ) : (
            <div className="mt-5 grid gap-3">
              {liveJobs.map((entry) => (
                <a
                  key={entry.job.id}
                  href={`/jobs/view/?id=${encodeURIComponent(entry.job.id)}`}
                  className="block border border-zinc-800 bg-zinc-950/40 px-4 py-4 transition hover:bg-zinc-950"
                >
                  <div className="flex flex-col gap-3 md:flex-row md:items-start md:justify-between">
                    <div className="min-w-0">
                      <div className="flex flex-wrap items-center gap-2">
                        <div className="font-medium text-white">
                          {entry.job.package_name}
                        </div>
                        <span className="inline-flex items-center border border-zinc-800 bg-black px-2.5 py-1 font-mono text-xs text-zinc-300">
                          {entry.job.mock_chroot}
                        </span>
                      </div>
                      <div className="mt-1 truncate font-mono text-xs text-zinc-500">
                        {entry.job.revision}
                      </div>
                    </div>
                    <div className="flex md:justify-end">
                      <StatusPill status={entry.job.status} />
                    </div>
                  </div>
                </a>
              ))}
            </div>
          )}
        </article>

        <article className="border border-zinc-800 bg-black p-6">
          <div className="flex items-center gap-2 text-xs uppercase tracking-[0.22em] text-zinc-500">
            <FaIcon icon={faFolderTree} />
            Published Repository
          </div>
          <h2 className="mt-3 text-2xl font-semibold text-white">
            Repository snapshot
          </h2>
          <p className="mt-2 text-sm leading-6 text-zinc-400">
            Current published package, target, build, and file counts.
          </p>

          <div className="mt-5 grid gap-4 sm:grid-cols-2">
            <MiniMetric
              label="Published packages"
              value={repoSummary?.package_count ?? 0}
            />
            <MiniMetric
              label="Targets"
              value={repoSummary?.target_count ?? 0}
            />
            <MiniMetric label="Builds" value={repoSummary?.build_count ?? 0} />
            <MiniMetric
              label="Files"
              value={repoSummary?.published_file_count ?? 0}
            />
          </div>

          <div className="mt-5 border-t border-zinc-800 pt-4">
            <div className="text-xs uppercase tracking-[0.18em] text-zinc-500">
              Historical jobs
            </div>
            <div className="mt-2 text-sm text-zinc-200">{jobCount}</div>
          </div>
        </article>
      </section>
    </div>
  );
}

function MiniMetric({
  label,
  value,
}: {
  label: string;
  value: string | number;
}) {
  return (
    <div className="border border-zinc-800 bg-zinc-950/40 px-4 py-4">
      <div className="text-[10px] uppercase tracking-[0.18em] text-zinc-500">
        {label}
      </div>
      <div className="mt-2 text-2xl font-semibold tracking-tight text-white">
        {value}
      </div>
    </div>
  );
}

function formatBytes(bytes: number) {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KiB`;
  if (bytes < 1024 * 1024 * 1024)
    return `${(bytes / (1024 * 1024)).toFixed(1)} MiB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GiB`;
}
