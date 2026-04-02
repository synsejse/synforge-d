import { useEffect, useState } from "react";
import api from "../lib/api";
import { formatDateTime } from "../lib/datetime";
import type { BuildJobResponse, PackageResponse } from "../lib/types";
import EmptyState from "./EmptyState";
import MetricCard from "./MetricCard";
import PageHeader from "./PageHeader";
import StatusPill from "./StatusPill";
import { faBoxesStacked, faChartLine } from "@fortawesome/free-solid-svg-icons";

export default function Dashboard() {
  const [packages, setPackages] = useState<PackageResponse[]>([]);
  const [jobs, setJobs] = useState<BuildJobResponse[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    async function load() {
      try {
        const [pkgRes, jobRes] = await Promise.all([api.listPackages(), api.listJobs()]);
        setPackages(pkgRes.packages);
        setJobs(jobRes.jobs);
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
    return <div className="text-zinc-400">Loading overview…</div>;
  }

  if (error) {
    return <div className="border border-zinc-800 bg-black p-4 text-zinc-200">Error: {error}</div>;
  }

  const activeJobs = jobs.filter((entry) => entry.job.status === "pending" || entry.job.status === "running");
  const recentJobs = jobs.slice(0, 6);
  const stats = [
    { label: "Packages", value: packages.length, detail: "Registered sources" },
    { label: "Enabled", value: packages.filter((entry) => entry.package.enabled).length, detail: "Actively buildable" },
    { label: "Active Jobs", value: activeJobs.length, detail: "Live builds now" },
    { label: "Total Jobs", value: jobs.length, detail: "Historical runs" },
  ];

  return (
    <div className="space-y-8">
      <PageHeader
        eyebrow="System Overview"
        title="Operations at a glance"
        description="A high-signal snapshot of package state, active builds, and recent execution history."
        actions={[
          { href: "/packages/", label: "Manage Packages", icon: faBoxesStacked },
          { href: "/jobs/", label: "Open Jobs", icon: faChartLine, variant: "primary" },
        ]}
      />

      <section className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
        {stats.map((stat) => (
          <MetricCard key={stat.label} label={stat.label} value={stat.value} detail={stat.detail} />
        ))}
      </section>

      <section className="grid gap-6 xl:grid-cols-[minmax(0,1.1fr)_minmax(320px,0.9fr)]">
        <div className="border border-zinc-800 bg-black p-6">
          <div className="mb-5 flex items-end justify-between">
            <div>
              <h2 className="text-xl font-semibold text-white">Recent Jobs</h2>
              <p className="mt-2 text-sm text-zinc-400">Latest build runs across every package.</p>
            </div>
            <a href="/jobs/" className="text-sm text-zinc-300 transition hover:text-white">
              View all →
            </a>
          </div>

          {recentJobs.length === 0 ? (
            <EmptyState>No jobs have run yet.</EmptyState>
          ) : (
            <div className="grid gap-3">
              {recentJobs.map((entry) => (
                <a
                  key={entry.job.id}
                  href={`/jobs/view/?id=${encodeURIComponent(entry.job.id)}`}
                  className="grid gap-3 border border-zinc-800 bg-black p-4 transition hover:bg-zinc-950 md:grid-cols-[minmax(0,180px)_minmax(0,88px)_minmax(0,1fr)_auto]"
                >
                  <div className="min-w-0">
                    <div className="font-medium text-white">{entry.job.package_name}</div>
                    <div className="mt-1 truncate text-xs text-zinc-500">{entry.job.id}</div>
                  </div>
                  <div className="flex items-start md:justify-start">
                    <span className="inline-flex items-center border border-zinc-800 bg-black px-2.5 py-1 font-mono text-xs text-zinc-300">
                      {entry.job.mock_chroot}
                    </span>
                  </div>
                  <div className="min-w-0">
                    <div className="truncate font-mono text-sm text-zinc-300">{entry.job.revision}</div>
                    <div className="mt-1 text-xs text-zinc-500">{formatDateTime(entry.job.created_at)}</div>
                  </div>
                  <div className="flex items-start md:justify-end">
                    <StatusPill status={entry.job.status} />
                  </div>
                </a>
              ))}
            </div>
          )}
        </div>

        <div className="space-y-4 border border-zinc-800 bg-black p-6">
          <div>
            <h2 className="text-xl font-semibold text-white">Active Builds</h2>
            <p className="mt-2 text-sm text-zinc-400">Jobs currently pending or running.</p>
          </div>
          {activeJobs.length === 0 ? (
            <EmptyState>Nothing is building right now.</EmptyState>
          ) : (
            activeJobs.map((entry) => (
              <a
                key={entry.job.id}
                href={`/jobs/view/?id=${encodeURIComponent(entry.job.id)}`}
                className="block border border-zinc-800 bg-black px-4 py-4 transition hover:bg-zinc-950"
              >
                <div className="flex flex-col gap-3 md:flex-row md:items-start md:justify-between">
                  <div className="min-w-0">
                    <div className="flex flex-wrap items-center gap-2">
                      <div className="font-medium text-white">{entry.job.package_name}</div>
                      <span className="inline-flex items-center border border-zinc-800 bg-black px-2.5 py-1 font-mono text-xs text-zinc-300">
                        {entry.job.mock_chroot}
                      </span>
                    </div>
                    <div className="mt-1 truncate font-mono text-xs text-zinc-500">{entry.job.revision}</div>
                  </div>
                  <div className="flex md:justify-end">
                    <StatusPill status={entry.job.status} />
                  </div>
                </div>
              </a>
            ))
          )}
        </div>
      </section>
    </div>
  );
}
