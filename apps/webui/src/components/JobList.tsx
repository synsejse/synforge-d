import { useEffect, useState } from "react";
import api from "../lib/api";
import ActionButton from "./ActionButton";
import EmptyState from "./EmptyState";
import PageHeader from "./PageHeader";
import StatusPill from "./StatusPill";
import { formatDateTime, formatDurationBetween } from "../lib/datetime";
import type { BuildJobResponse, BuildStatus } from "../lib/types";
import { faChartLine, faFolderOpen, faTrash } from "@fortawesome/free-solid-svg-icons";

const FILTERS: Array<"all" | BuildStatus> = [
  "all",
  "pending",
  "running",
  "succeeded",
  "failed",
  "timed_out",
];

export default function JobList() {
  const [jobs, setJobs] = useState<BuildJobResponse[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [filter, setFilter] = useState<string>("all");

  async function load() {
    try {
      setLoading(true);
      const res = await api.listJobs();
      setJobs(res.jobs);
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to load jobs");
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    load();
  }, []);

  async function handleDelete(job: BuildJobResponse) {
    if (!confirm(`Delete job ${job.job.id}?`)) {
      return;
    }
    try {
      await api.deleteJob(job.job.id);
      await load();
    } catch (e) {
      alert(e instanceof Error ? e.message : "Failed to delete job");
    }
  }

  if (loading) {
    return <div className="text-zinc-400">Loading jobs…</div>;
  }

  if (error) {
    return <div className="border border-zinc-800 bg-black p-4 text-zinc-200">Error: {error}</div>;
  }

  const filteredJobs =
    filter === "all" ? jobs : jobs.filter((entry) => entry.job.status === filter);

  const statusCounts = jobs.reduce(
    (acc, entry) => {
      acc[entry.job.status] = (acc[entry.job.status] || 0) + 1;
      return acc;
    },
    {} as Record<string, number>
  );

  return (
    <div className="space-y-8">
      <PageHeader
        eyebrow="Job Activity"
        title="Build Timeline"
        description="Queued, running, and finished jobs."
        actions={[{ href: "/", label: "Overview", icon: faChartLine }]}
      />

      <section className="flex flex-wrap gap-2 border border-zinc-800 bg-black p-4">
        {FILTERS.map((value) => (
          <button
            key={value}
            onClick={() => setFilter(value)}
            className={`border px-4 py-2 text-sm transition ${
              filter === value
                ? "border-zinc-200 bg-zinc-100 text-black"
                : "border-zinc-800 bg-black text-zinc-300 hover:border-zinc-600 hover:bg-zinc-950"
            }`}
          >
            {value} {value !== "all" && statusCounts[value] ? `(${statusCounts[value]})` : ""}
          </button>
        ))}
      </section>

      {filteredJobs.length === 0 ? (
        <EmptyState>No jobs match the current filter.</EmptyState>
      ) : (
        <div className="overflow-x-auto border border-zinc-800 bg-black">
          <table className="min-w-[980px] w-full">
            <thead className="bg-zinc-950 text-left text-xs uppercase tracking-[0.2em] text-zinc-500">
              <tr>
                <th className="px-4 py-3">Package</th>
                <th className="px-4 py-3">Target</th>
                <th className="px-4 py-3">Revision</th>
                <th className="px-4 py-3">Status</th>
                <th className="px-4 py-3">Trigger</th>
                <th className="px-4 py-3">Duration</th>
                <th className="px-4 py-3">Created</th>
                <th className="px-4 py-3">Actions</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-white/8">
              {filteredJobs.map((entry) => {
                const isLive = entry.job.status === "pending" || entry.job.status === "running";
                return (
                  <tr key={entry.job.id} className="hover:bg-zinc-950">
                    <td className="px-4 py-3">
                      <div className="min-w-[160px] space-y-1">
                        <a
                          href={`/jobs/view/?id=${encodeURIComponent(entry.job.id)}`}
                          className="font-medium text-white transition hover:text-zinc-300"
                        >
                          {entry.job.package_name}
                        </a>
                        <div className="max-w-[180px] break-all text-xs text-zinc-500">{entry.job.id}</div>
                      </div>
                    </td>
                    <td className="px-4 py-3 text-sm font-mono text-zinc-300">{entry.job.mock_chroot}</td>
                    <td className="px-4 py-3">
                      <div className="max-w-[420px] break-all font-mono text-sm text-zinc-300">{entry.job.revision}</div>
                    </td>
                    <td className="px-4 py-3">
                      <StatusPill status={entry.job.status} />
                    </td>
                    <td className="px-4 py-3 text-sm text-zinc-400">{entry.job.trigger}</td>
                    <td className="px-4 py-3 text-sm text-zinc-400">
                      {formatDurationBetween(entry.job.created_at, entry.job.finished_at)}
                    </td>
                    <td className="px-4 py-3 text-sm text-zinc-400">
                      {formatDateTime(entry.job.created_at)}
                    </td>
                    <td className="px-4 py-3">
                      <div className="flex flex-wrap gap-2">
                        <ActionButton
                          href={`/jobs/view/?id=${encodeURIComponent(entry.job.id)}`}
                          icon={faFolderOpen}
                        >
                          Open
                        </ActionButton>
                        <ActionButton
                          onClick={() => handleDelete(entry)}
                          disabled={isLive}
                          icon={faTrash}
                          className="text-zinc-300 disabled:cursor-not-allowed disabled:opacity-50"
                        >
                          Delete
                        </ActionButton>
                      </div>
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
