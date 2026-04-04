import { useEffect, useState } from "react";
import api from "../lib/api";
import ActionButton from "./ActionButton";
import EmptyState from "./EmptyState";
import FaIcon from "./FaIcon";
import LoadingBlock from "./LoadingBlock";
import PageHeader from "./PageHeader";
import StatusPill from "./StatusPill";
import { formatDateTime, formatDurationBetween } from "../lib/datetime";
import type { BuildJobResponse, BuildStatus } from "../lib/types";
import {
  faChartLine,
  faFolderOpen,
  faMagnifyingGlass,
  faTrash,
} from "@fortawesome/free-solid-svg-icons";

const FILTERS: Array<"all" | BuildStatus> = [
  "all",
  "pending",
  "running",
  "succeeded",
  "failed",
  "timed_out",
];

type JobViewMode = "history" | "active";

export default function JobList() {
  const [jobs, setJobs] = useState<BuildJobResponse[]>([]);
  const [hasMore, setHasMore] = useState(false);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [mode, setMode] = useState<JobViewMode>(() => {
    if (typeof window === "undefined") {
      return "history";
    }
    const value = new URLSearchParams(window.location.search).get("mode");
    return value === "active" ? "active" : "history";
  });
  const [filter, setFilter] = useState<string>(() => {
    if (typeof window === "undefined") {
      return "all";
    }
    return new URLSearchParams(window.location.search).get("status") || "all";
  });
  const [offset, setOffset] = useState<number>(() => {
    if (typeof window === "undefined") {
      return 0;
    }
    return Number(
      new URLSearchParams(window.location.search).get("offset") || "0",
    );
  });
  const [packageFilter, setPackageFilter] = useState(() => {
    if (typeof window === "undefined") {
      return "";
    }
    return new URLSearchParams(window.location.search).get("package") || "";
  });
  const [targetFilter, setTargetFilter] = useState(() => {
    if (typeof window === "undefined") {
      return "";
    }
    return new URLSearchParams(window.location.search).get("target") || "";
  });
  const [pruning, setPruning] = useState(false);
  const pageSize = 50;

  async function load(
    nextMode = mode,
    nextFilter = filter,
    nextOffset = offset,
    nextPackageFilter = packageFilter,
    nextTargetFilter = targetFilter,
  ) {
    try {
      setLoading(true);
      const res =
        nextMode === "active"
          ? await api.listActiveJobs({
              limit: pageSize,
              offset: nextOffset,
              packageName: nextPackageFilter,
              mockChroot: nextTargetFilter,
            })
          : await api.listJobs({
              limit: pageSize,
              offset: nextOffset,
              status: nextFilter,
              packageName: nextPackageFilter,
              mockChroot: nextTargetFilter,
              completedOnly: true,
            });
      setJobs(res.jobs);
      setHasMore(res.page.has_more);
      setMode(nextMode);
      setOffset(nextOffset);
      setFilter(nextFilter);
      setPackageFilter(nextPackageFilter);
      setTargetFilter(nextTargetFilter);
      setError(null);
      if (typeof window !== "undefined") {
        const params = new URLSearchParams(window.location.search);
        if (nextMode === "history") {
          params.delete("mode");
        } else {
          params.set("mode", nextMode);
        }
        if (nextMode === "active" || nextFilter === "all") {
          params.delete("status");
        } else {
          params.set("status", nextFilter);
        }
        if (nextOffset === 0) {
          params.delete("offset");
        } else {
          params.set("offset", String(nextOffset));
        }
        if (nextPackageFilter.trim()) {
          params.set("package", nextPackageFilter.trim());
        } else {
          params.delete("package");
        }
        if (nextTargetFilter.trim()) {
          params.set("target", nextTargetFilter.trim());
        } else {
          params.delete("target");
        }
        const query = params.toString();
        window.history.replaceState(
          {},
          "",
          `/jobs/${query ? `?${query}` : ""}`,
        );
      }
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

  async function handlePruneFailed() {
    const failedCount = jobs.filter(
      (entry) =>
        entry.job.status === "failed" || entry.job.status === "timed_out",
    ).length;
    if (failedCount === 0) {
      return;
    }
    if (!confirm(`Delete ${failedCount} failed or timed out jobs?`)) {
      return;
    }
    try {
      setPruning(true);
      await api.pruneFailedJobs();
      await load();
    } catch (e) {
      alert(e instanceof Error ? e.message : "Failed to prune failed jobs");
    } finally {
      setPruning(false);
    }
  }

  if (loading) {
    return <LoadingBlock label="Loading jobs…" lines={4} />;
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
        eyebrow="Job Activity"
        title={mode === "active" ? "Active Builds" : "Build Timeline"}
        description={
          mode === "active"
            ? "Pending and running jobs currently in flight."
            : "Finished job history across all packages and targets."
        }
        actions={[{ href: "/", label: "Overview", icon: faChartLine }]}
      />

      <section className="flex flex-wrap gap-2 border border-zinc-800 bg-black p-4">
        {(["history", "active"] as JobViewMode[]).map((value) => (
          <button
            key={value}
            onClick={() => load(value, "all", 0, packageFilter, targetFilter)}
            aria-pressed={mode === value}
            className={`border px-4 py-2 text-sm transition ${
              mode === value
                ? "border-zinc-200 bg-zinc-100 text-black"
                : "border-zinc-800 bg-black text-zinc-300 hover:border-zinc-600 hover:bg-zinc-950"
            }`}
          >
            {value === "history" ? "History" : "Active"}
          </button>
        ))}
        {mode === "history" ? (
          <>
            <ActionButton
              onClick={handlePruneFailed}
              disabled={
                pruning ||
                !jobs.some(
                  (entry) =>
                    entry.job.status === "failed" ||
                    entry.job.status === "timed_out",
                )
              }
              icon={faTrash}
              className="text-zinc-300 disabled:cursor-not-allowed disabled:opacity-50"
            >
              {pruning ? "Pruning…" : "Prune Failed"}
            </ActionButton>
            {FILTERS.map((value) => (
              <button
                key={value}
                onClick={() => load(mode, value, 0, packageFilter, targetFilter)}
                aria-pressed={filter === value}
                className={`border px-4 py-2 text-sm transition ${
                  filter === value
                    ? "border-zinc-200 bg-zinc-100 text-black"
                    : "border-zinc-800 bg-black text-zinc-300 hover:border-zinc-600 hover:bg-zinc-950"
                }`}
              >
                {value}
              </button>
            ))}
          </>
        ) : null}
      </section>

      <section className="grid gap-3 border border-zinc-800 bg-black p-4 md:grid-cols-[minmax(0,1fr)_240px_auto]">
        <label className="block">
          <span className="sr-only">Filter jobs by package</span>
          <input
            type="search"
            value={packageFilter}
            onChange={(event) => setPackageFilter(event.target.value)}
            placeholder="Filter by package"
            className="w-full border border-zinc-800 bg-black px-4 py-2.5 text-sm text-white outline-none transition focus:border-zinc-600"
          />
        </label>
        <label className="block">
          <span className="sr-only">Filter jobs by target</span>
          <input
            type="search"
            value={targetFilter}
            onChange={(event) => setTargetFilter(event.target.value)}
            placeholder="Filter by target"
            className="w-full border border-zinc-800 bg-black px-4 py-2.5 text-sm text-white outline-none transition focus:border-zinc-600"
          />
        </label>
        <button
          type="button"
          onClick={() => load(mode, filter, 0, packageFilter, targetFilter)}
          className="border border-zinc-800 bg-black px-4 py-2.5 text-sm text-zinc-200 transition hover:border-zinc-600 hover:bg-zinc-950"
        >
          <FaIcon icon={faMagnifyingGlass} className="mr-2" />
          Apply
        </button>
      </section>

      {jobs.length === 0 ? (
        <EmptyState>
          {mode === "active"
            ? "No active jobs match the current filter."
            : "No finished jobs match the current filter."}
        </EmptyState>
      ) : (
        <div className="overflow-x-auto border border-zinc-800 bg-black">
          <table className="min-w-[980px] w-full">
            <caption className="sr-only">
              Build jobs with status, target, revision, and row actions.
            </caption>
            <thead className="bg-zinc-950 text-left text-xs uppercase tracking-[0.2em] text-zinc-500">
              <tr>
                <th scope="col" className="px-4 py-3">
                  Package
                </th>
                <th scope="col" className="px-4 py-3">
                  Target
                </th>
                <th scope="col" className="px-4 py-3">
                  Revision
                </th>
                <th scope="col" className="px-4 py-3">
                  Status
                </th>
                <th scope="col" className="px-4 py-3">
                  Trigger
                </th>
                <th scope="col" className="px-4 py-3">
                  Duration
                </th>
                <th scope="col" className="px-4 py-3">
                  Created
                </th>
                <th scope="col" className="px-4 py-3">
                  Actions
                </th>
              </tr>
            </thead>
            <tbody className="divide-y divide-white/8">
              {jobs.map((entry) => {
                const isLive =
                  entry.job.status === "pending" ||
                  entry.job.status === "running";
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
                        <div className="max-w-[180px] break-all text-xs text-zinc-500">
                          {entry.job.id}
                        </div>
                      </div>
                    </td>
                    <td className="px-4 py-3 text-sm font-mono text-zinc-300">
                      {entry.job.mock_chroot}
                    </td>
                    <td className="px-4 py-3">
                      <div className="max-w-[420px] break-all font-mono text-sm text-zinc-300">
                        {entry.job.revision}
                      </div>
                    </td>
                    <td className="px-4 py-3">
                      <StatusPill status={entry.job.status} />
                    </td>
                    <td className="px-4 py-3 text-sm text-zinc-400">
                      {entry.job.trigger}
                    </td>
                    <td className="px-4 py-3 text-sm text-zinc-400">
                      {formatDurationBetween(
                        entry.job.created_at,
                        entry.job.finished_at,
                      )}
                    </td>
                    <td className="px-4 py-3 text-sm text-zinc-400">
                      {formatDateTime(entry.job.created_at)}
                    </td>
                    <td className="px-4 py-3">
                      <div className="flex flex-wrap gap-2">
                        <ActionButton
                          href={`/jobs/view/?id=${encodeURIComponent(entry.job.id)}`}
                          icon={faFolderOpen}
                          aria-label={`Open job ${entry.job.id}`}
                        >
                          Open
                        </ActionButton>
                        <ActionButton
                          onClick={() => handleDelete(entry)}
                          disabled={isLive}
                          icon={faTrash}
                          aria-label={`Delete job ${entry.job.id}`}
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

      {jobs.length > 0 ? (
        <div className="flex items-center justify-between gap-3">
          <button
            onClick={() =>
              load(
                mode,
                filter,
                Math.max(0, offset - pageSize),
                packageFilter,
                targetFilter,
              )
            }
            disabled={loading || offset === 0}
            className="border border-zinc-800 px-4 py-2 text-sm text-zinc-300 disabled:cursor-not-allowed disabled:opacity-50"
          >
            Previous
          </button>
          <button
            onClick={() => load(mode, filter, offset + pageSize, packageFilter, targetFilter)}
            disabled={loading || !hasMore}
            className="border border-zinc-800 px-4 py-2 text-sm text-zinc-300 disabled:cursor-not-allowed disabled:opacity-50"
          >
            Next
          </button>
        </div>
      ) : null}
    </div>
  );
}
