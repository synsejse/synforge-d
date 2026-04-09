import { useEffect, useState } from "react";
import {
  faChartLine,
  faFilter,
  faStop,
  faTrash,
} from "@fortawesome/free-solid-svg-icons";
import api from "../../lib/api";
import type {
  BuildJobResponse,
  JobResourceUsageSample,
  ServerHardwareResponse,
} from "../../lib/types";
import {
  HISTORY_BUILD_STATUS_LABELS,
  isHistoryBuildStatus,
  type HistoryBuildStatus,
} from "../../lib/job-status";
import { formatDateTime, formatDurationBetween } from "../../lib/datetime";
import ErrorMessage from "../common/ErrorMessage";
import LoadingBlock from "../ui/LoadingBlock";
import FaIcon from "../ui/FaIcon";
import Button from "../ui/Button";
import Badge from "../ui/Badge";
import Select from "../ui/Select";
import PageHeader from "../ui/PageHeader";

type JobViewMode = "active" | "history";

const USAGE_POLL_INTERVAL_MS = 1000;

function normalizeStatusFilter(value: string | null): "all" | HistoryBuildStatus {
  if (!value || value === "all") {
    return "all";
  }
  return isHistoryBuildStatus(value) ? value : "all";
}

export default function JobList() {
  const [jobs, setJobs] = useState<BuildJobResponse[]>([]);
  const [hasMore, setHasMore] = useState(false);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [mode, setMode] = useState<JobViewMode>(() => {
    if (typeof window === "undefined") return "history";
    const value = new URLSearchParams(window.location.search).get("mode");
    return value === "active" ? "active" : "history";
  });
  const [filter, setFilter] = useState<"all" | HistoryBuildStatus>(() => {
    if (typeof window === "undefined") return "all";
    const value = new URLSearchParams(window.location.search).get("status");
    return normalizeStatusFilter(value);
  });
  const [offset, setOffset] = useState<number>(() => {
    if (typeof window === "undefined") return 0;
    return Number(new URLSearchParams(window.location.search).get("offset") || "0");
  });
  const [packageFilter, setPackageFilter] = useState("");
  const [targetFilter, setTargetFilter] = useState("");
  const [pruning, setPruning] = useState(false);
  const [killingJobId, setKillingJobId] = useState<string | null>(null);
  const [pageVisible, setPageVisible] = useState(() => {
    if (typeof document === "undefined") return true;
    return document.visibilityState === "visible";
  });
  const [usageByJob, setUsageByJob] = useState<Record<string, JobResourceUsageSample>>({});
  const [serverHardware, setServerHardware] = useState<ServerHardwareResponse | null>(null);
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
          : await api.listCompletedJobs({
              limit: pageSize,
              offset: nextOffset,
              status: nextFilter,
              packageName: nextPackageFilter,
              mockChroot: nextTargetFilter,
            });
      setJobs(res.jobs);
      setHasMore(res.page.has_more);
      setMode(nextMode);
      setOffset(nextOffset);
      setFilter(nextFilter);
      setPackageFilter(nextPackageFilter);
      setTargetFilter(nextTargetFilter);
      if (nextMode !== "active") {
        setUsageByJob({});
      }
      setError(null);

      // Update URL
      if (typeof window !== "undefined") {
        const params = new URLSearchParams();
        if (nextMode !== "history") params.set("mode", nextMode);
        if (nextFilter !== "all" && nextMode === "history") params.set("status", nextFilter);
        if (nextOffset > 0) params.set("offset", String(nextOffset));
        if (nextPackageFilter.trim()) params.set("package", nextPackageFilter.trim());
        if (nextTargetFilter.trim()) params.set("target", nextTargetFilter.trim());
        const query = params.toString();
        window.history.replaceState({}, "", `/jobs/${query ? `?${query}` : ""}`);
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

  useEffect(() => {
    api
      .getServerHardware()
      .then((hardware) => setServerHardware(hardware))
      .catch(() => undefined);
  }, []);

  useEffect(() => {
    if (typeof document === "undefined") return;
    const updateVisibility = () => {
      setPageVisible(document.visibilityState === "visible");
    };
    document.addEventListener("visibilitychange", updateVisibility);
    return () => {
      document.removeEventListener("visibilitychange", updateVisibility);
    };
  }, []);

  useEffect(() => {
    if (mode !== "active" || !pageVisible) return;
    let cancelled = false;
    const pollUsage = async () => {
      try {
        const response = await api.listJobUsage();
        if (cancelled) return;
        const next: Record<string, JobResourceUsageSample> = {};
        for (const sample of response.samples) {
          next[sample.job_id] = sample;
        }
        setUsageByJob(next);
      } catch {
        // Keep existing values when a poll fails.
      }
    };
    void pollUsage();
    const timer = window.setInterval(() => {
      void pollUsage();
    }, USAGE_POLL_INTERVAL_MS);
    return () => {
      cancelled = true;
      window.clearInterval(timer);
    };
  }, [mode, pageVisible]);

  async function handleDelete(job: BuildJobResponse) {
    if (!confirm(`Delete job ${job.job.id}?`)) return;
    try {
      await api.deleteJob(job.job.id);
      await load();
    } catch (e) {
      alert(e instanceof Error ? e.message : "Failed to delete job");
    }
  }

  async function handlePruneFailed() {
    const failedCount = jobs.filter(
      (entry) => entry.job.status === "failed" || entry.job.status === "timed_out",
    ).length;
    if (failedCount === 0) return;
    if (!confirm(`Delete ${failedCount} failed or timed out jobs?`)) return;
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

  async function handleKill(job: BuildJobResponse) {
    if (
      !confirm(
        `Kill active job ${job.job.id} (${job.job.package_name} / ${job.job.mock_chroot})?`,
      )
    ) {
      return;
    }
    try {
      setKillingJobId(job.job.id);
      await api.killJob(job.job.id);
      await load();
    } catch (e) {
      alert(e instanceof Error ? e.message : "Failed to kill job");
    } finally {
      setKillingJobId(null);
    }
  }

  const getStatusVariant = (status: string) => {
    if (status === "succeeded") return "success";
    if (status === "failed" || status === "timed_out") return "error";
    if (status === "running") return "lime";
    if (status === "pending") return "warning";
    return "default";
  };

  const formatMemory = (bytes: number) => {
    if (bytes >= 1024 * 1024 * 1024) {
      return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GiB`;
    }
    return `${(bytes / (1024 * 1024)).toFixed(0)} MiB`;
  };

  if (loading && jobs.length === 0) {
    return <LoadingBlock label="Loading jobs…" lines={4} />;
  }

  if (error) {
    return <ErrorMessage message={error} />;
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <PageHeader
        eyebrow="JOB_ACTIVITY"
        title={mode === "active" ? "Active Builds" : "Build Timeline"}
        description={
          mode === "active"
            ? "Pending and running jobs currently in flight."
            : "Finished job history across all packages and targets."
        }
        color="orange"
        actions={[{ href: "/", label: "Overview", icon: faChartLine }]}
      />

      {/* Mode Toggle + Filters */}
      <div className="border-4 border-[var(--theme-border-strong)] bg-black shadow-[4px_4px_0_rgba(255,255,255,0.1)]">
        <div className="border-b-4 border-[var(--theme-border-strong)] bg-gradient-to-r from-zinc-900 to-black px-6 py-4">
          <div className="flex flex-wrap items-center gap-4">
            {/* Mode Toggle */}
            <div className="flex border-2 border-[var(--theme-border-strong)]">
              <button
                onClick={() => load("history", "all", 0, packageFilter, targetFilter)}
                className={`px-5 py-2.5 font-mono text-sm font-bold uppercase tracking-wider transition-all ${
                  mode === "history"
                    ? "bg-[var(--theme-accent-lime)] text-black"
                    : "bg-black text-[var(--theme-text-muted)] hover:text-white"
                }`}
              >
                History
              </button>
              <button
                onClick={() => load("active", "all", 0, packageFilter, targetFilter)}
                className={`border-l-2 border-[var(--theme-border-strong)] px-5 py-2.5 font-mono text-sm font-bold uppercase tracking-wider transition-all ${
                  mode === "active"
                    ? "bg-[var(--theme-terminal-green)] text-black"
                    : "bg-black text-[var(--theme-text-muted)] hover:text-white"
                }`}
              >
                Active
              </button>
            </div>

            {/* Status Filter (history only) */}
            {mode === "history" && (
              <div className="w-full sm:flex-1 sm:min-w-[200px] sm:max-w-xs">
                <Select
                  options={[
                    { value: "all", label: "All Statuses" },
                    ...(Object.entries(HISTORY_BUILD_STATUS_LABELS).map(
                      ([value, label]) => ({ value, label }),
                    )),
                  ]}
                  value={filter}
                  onValueChange={(val) =>
                    load(mode, normalizeStatusFilter(val), 0, packageFilter, targetFilter)
                  }
                  placeholder="Filter status..."
                />
              </div>
            )}

            {/* Prune Button */}
            {mode === "history" && (
              <Button
                variant="danger"
                size="sm"
                onClick={handlePruneFailed}
                disabled={pruning || jobs.filter((e) => e.job.status === "failed" || e.job.status === "timed_out").length === 0}
              >
                <FaIcon icon={faTrash} />
                Prune Failed
              </Button>
            )}
          </div>
        </div>

        {/* Search Filters */}
        <div className="border-b-4 border-[var(--theme-border-strong)] bg-black px-6 py-4">
          <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
            <div>
              <label className="mb-2 block font-mono text-xs font-bold uppercase tracking-wider text-zinc-500">
                Package
              </label>
              <input
                type="text"
                value={packageFilter}
                onChange={(e) => setPackageFilter(e.target.value)}
                onKeyDown={(e) => e.key === "Enter" && load(mode, filter, 0, packageFilter, targetFilter)}
                placeholder="Filter by package..."
                className="w-full border-2 border-[var(--theme-border-strong)] bg-black px-4 py-2.5 font-mono text-sm text-white transition focus:border-[var(--theme-accent-lime)] focus:outline-none focus:ring-2 focus:ring-[var(--theme-accent-lime)] focus:ring-offset-2 focus:ring-offset-black"
              />
            </div>
            <div>
              <label className="mb-2 block font-mono text-xs font-bold uppercase tracking-wider text-zinc-500">
                Target
              </label>
              <input
                type="text"
                value={targetFilter}
                onChange={(e) => setTargetFilter(e.target.value)}
                onKeyDown={(e) => e.key === "Enter" && load(mode, filter, 0, packageFilter, targetFilter)}
                placeholder="Filter by target..."
                className="w-full border-2 border-[var(--theme-border-strong)] bg-black px-4 py-2.5 font-mono text-sm text-white transition focus:border-[var(--theme-accent-lime)] focus:outline-none focus:ring-2 focus:ring-[var(--theme-accent-lime)] focus:ring-offset-2 focus:ring-offset-black"
              />
            </div>
            <div className="flex items-end">
              <Button
                variant="primary"
                className="w-full"
                onClick={() => load(mode, filter, 0, packageFilter, targetFilter)}
              >
                <FaIcon icon={faFilter} />
                Apply Filters
              </Button>
            </div>
          </div>
        </div>

        {/* Jobs Table */}
        <div>
          {jobs.length === 0 ? (
            <div className="flex min-h-[300px] items-center justify-center px-6 py-12">
              <div className="text-center">
                <div className="font-mono text-sm text-zinc-500">
                  {mode === "active" ? "No active jobs" : "No jobs found"}
                </div>
              </div>
            </div>
          ) : (
            <>
              <div className="space-y-3 p-4 md:hidden">
                {jobs.map((entry) => {
                  const isLive = entry.job.status === "pending" || entry.job.status === "running";
                  const latestUsage = usageByJob[entry.job.id] ?? null;
                  return (
                    <article key={`mobile:${entry.job.id}`} className="border-2 border-zinc-700 bg-black p-4">
                      <div className="flex items-start justify-between gap-3">
                        <div className="min-w-0">
                          <a
                            href={`/jobs/view/?id=${encodeURIComponent(entry.job.id)}`}
                            className="font-display font-bold text-white transition hover:text-[var(--theme-accent-lime)]"
                          >
                            {entry.job.package_name}
                          </a>
                          <div className="mt-1 break-all font-mono text-xs text-zinc-600">
                            {entry.job.id}
                          </div>
                        </div>
                        <Badge variant={getStatusVariant(entry.job.status)} pulse={isLive}>
                          {entry.job.status}
                        </Badge>
                      </div>
                      <div className="mt-3 space-y-2 font-mono text-xs text-zinc-400">
                        <div>
                          <span className="text-zinc-500">Target:</span>{" "}
                          <Badge variant="ghost">{entry.job.mock_chroot}</Badge>
                        </div>
                        <div className="break-all">
                          <span className="text-zinc-500">Revision:</span>{" "}
                          {entry.job.revision}
                        </div>
                        <div>
                          <span className="text-zinc-500">Duration:</span>{" "}
                          {formatDurationBetween(entry.job.created_at, entry.job.finished_at)}
                        </div>
                        <div>
                          <span className="text-zinc-500">Created:</span>{" "}
                          {formatDateTime(entry.job.created_at)}
                        </div>
                        {mode === "active" && isLive && (
                          <div className="space-y-3 border-2 border-zinc-700 bg-zinc-950 px-3 py-3">
                            <UsageBarRow
                              label="RAM"
                              value={formatMemoryUsage(latestUsage, serverHardware, formatMemory)}
                              percent={memoryUsagePercent(latestUsage, serverHardware)}
                              fillClass="bg-amber-400"
                              valueClass="text-amber-300"
                            />
                          </div>
                        )}
                      </div>
                      <div className="mt-4 grid gap-2 sm:flex sm:flex-wrap">
                        <Button
                          variant="ghost"
                          size="sm"
                          className="w-full sm:w-auto"
                          onClick={() => (window.location.href = `/jobs/view/?id=${encodeURIComponent(entry.job.id)}`)}
                        >
                          Open
                        </Button>
                        {mode === "active" && isLive && (
                          <Button
                            variant="warning"
                            size="sm"
                            className="w-full sm:w-auto"
                            onClick={() => handleKill(entry)}
                            disabled={killingJobId === entry.job.id}
                          >
                            <FaIcon icon={faStop} />
                            Kill Active
                          </Button>
                        )}
                        {mode !== "active" && (
                          <Button
                            variant="danger"
                            size="sm"
                            className="w-full sm:w-auto"
                            onClick={() => handleDelete(entry)}
                            disabled={isLive}
                          >
                            <FaIcon icon={faTrash} />
                            Delete
                          </Button>
                        )}
                      </div>
                    </article>
                  );
                })}
              </div>
              <div className="hidden overflow-x-auto md:block">
                <table className="w-full min-w-[640px] lg:min-w-[980px]">
                  <thead className="border-b-2 border-[var(--theme-border-strong)] bg-zinc-950">
                    <tr>
                      <th scope="col" className="px-5 py-4 text-left font-mono text-xs font-bold uppercase tracking-[0.2em] text-zinc-500">
                        Package
                      </th>
                      <th scope="col" className="px-5 py-4 text-left font-mono text-xs font-bold uppercase tracking-[0.2em] text-zinc-500">
                        Target
                      </th>
                      <th scope="col" className="px-5 py-4 text-left font-mono text-xs font-bold uppercase tracking-[0.2em] text-zinc-500">
                        Revision
                      </th>
                      <th scope="col" className="px-5 py-4 text-left font-mono text-xs font-bold uppercase tracking-[0.2em] text-zinc-500">
                        Status
                      </th>
                      <th scope="col" className="px-5 py-4 text-left font-mono text-xs font-bold uppercase tracking-[0.2em] text-zinc-500">
                        Duration
                      </th>
                      <th scope="col" className="px-5 py-4 text-left font-mono text-xs font-bold uppercase tracking-[0.2em] text-zinc-500">
                        Created
                      </th>
                      {mode === "active" && (
                        <th scope="col" className="px-5 py-4 text-left font-mono text-xs font-bold uppercase tracking-[0.2em] text-zinc-500">
                          Live Usage
                        </th>
                      )}
                      <th scope="col" className="px-5 py-4 text-left font-mono text-xs font-bold uppercase tracking-[0.2em] text-zinc-500">
                        Actions
                      </th>
                    </tr>
                  </thead>
                  <tbody>
                    {jobs.map((entry, idx) => {
                      const isLive = entry.job.status === "pending" || entry.job.status === "running";
                      const latestUsage = usageByJob[entry.job.id] ?? null;
                      return (
                        <tr
                          key={entry.job.id}
                          className={`border-b-2 border-[var(--theme-border)] transition-all hover:bg-zinc-950 ${
                            idx % 2 === 0 ? "bg-black" : "bg-zinc-950/40"
                          }`}
                        >
                          <td className="px-5 py-4">
                            <div className="min-w-[160px]">
                              <a
                                href={`/jobs/view/?id=${encodeURIComponent(entry.job.id)}`}
                                className="font-display font-bold text-white transition hover:text-[var(--theme-accent-lime)]"
                              >
                                {entry.job.package_name}
                              </a>
                              <div className="mt-1 max-w-[200px] truncate font-mono text-xs text-zinc-600">
                                {entry.job.id}
                              </div>
                            </div>
                          </td>
                          <td className="px-5 py-4">
                            <Badge variant="ghost">{entry.job.mock_chroot}</Badge>
                          </td>
                          <td className="px-5 py-4">
                            <div className="max-w-[300px] truncate font-mono text-sm text-zinc-300">
                              {entry.job.revision}
                            </div>
                          </td>
                          <td className="px-5 py-4">
                            <Badge variant={getStatusVariant(entry.job.status)} pulse={isLive}>
                              {entry.job.status}
                            </Badge>
                          </td>
                          <td className="px-5 py-4 font-mono text-sm text-zinc-400">
                            {formatDurationBetween(entry.job.created_at, entry.job.finished_at)}
                          </td>
                          <td className="px-5 py-4 font-mono text-sm text-zinc-400">
                            {formatDateTime(entry.job.created_at)}
                          </td>
                          {mode === "active" && (
                            <td className="px-5 py-4">
                              {isLive && latestUsage ? (
                                <div className="min-w-[320px] border-2 border-zinc-700 bg-zinc-950 p-3">
                                  <UsageBarRow
                                    label="RAM"
                                    value={formatMemoryUsage(latestUsage, serverHardware, formatMemory)}
                                    percent={memoryUsagePercent(latestUsage, serverHardware)}
                                    fillClass="bg-amber-400"
                                    valueClass="text-amber-300"
                                  />
                                </div>
                              ) : (
                                <span className="font-mono text-xs text-zinc-600">-</span>
                              )}
                            </td>
                          )}
                          <td className="px-5 py-4">
                            <div className="flex flex-wrap gap-2">
                              <Button
                                variant="ghost"
                                size="sm"
                                onClick={() => (window.location.href = `/jobs/view/?id=${encodeURIComponent(entry.job.id)}`)}
                              >
                                Open
                              </Button>
                              {mode === "active" && isLive && (
                                <Button
                                  variant="warning"
                                  size="sm"
                                  onClick={() => handleKill(entry)}
                                  disabled={killingJobId === entry.job.id}
                                >
                                  <FaIcon icon={faStop} />
                                  Kill Active
                                </Button>
                              )}
                              {mode !== "active" && (
                                <Button
                                  variant="danger"
                                  size="sm"
                                  onClick={() => handleDelete(entry)}
                                  disabled={isLive}
                                >
                                  <FaIcon icon={faTrash} />
                                  Delete
                                </Button>
                              )}
                            </div>
                          </td>
                        </tr>
                      );
                    })}
                  </tbody>
                </table>
              </div>
            </>
          )}
        </div>

        {/* Pagination */}
        {(offset > 0 || hasMore) && (
          <div className="border-t-4 border-[var(--theme-border-strong)] bg-zinc-950 px-6 py-4">
            <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
              <div className="font-mono text-sm text-zinc-500">
                Showing {offset + 1}-{offset + jobs.length}
              </div>
              <div className="flex w-full gap-3 sm:w-auto">
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => load(mode, filter, Math.max(0, offset - pageSize), packageFilter, targetFilter)}
                  disabled={offset === 0}
                >
                  ← Previous
                </Button>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => load(mode, filter, offset + pageSize, packageFilter, targetFilter)}
                  disabled={!hasMore}
                >
                  Next →
                </Button>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

function clampPercent(value: number): number {
  if (!Number.isFinite(value)) return 0;
  return Math.max(0, Math.min(100, value));
}

function resolveMemoryCapacityBytes(
  sample: JobResourceUsageSample,
  hardware: ServerHardwareResponse | null,
): number | null {
  if (sample.memory_limit_bytes > 0) {
    return sample.memory_limit_bytes;
  }
  if (hardware && hardware.total_memory_mb > 0) {
    return hardware.total_memory_mb * 1024 * 1024;
  }
  return null;
}

function memoryUsagePercent(
  sample: JobResourceUsageSample | null,
  hardware: ServerHardwareResponse | null,
): number {
  if (!sample) return 0;
  const memoryCapacityBytes = resolveMemoryCapacityBytes(sample, hardware);
  if (!memoryCapacityBytes || memoryCapacityBytes <= 0) return 0;
  return clampPercent((sample.memory_usage_bytes / memoryCapacityBytes) * 100);
}

function formatMemoryUsage(
  sample: JobResourceUsageSample | null,
  hardware: ServerHardwareResponse | null,
  formatter: (bytes: number) => string,
): string {
  if (!sample) return "-";
  const memoryCapacityBytes = resolveMemoryCapacityBytes(sample, hardware);
  if (memoryCapacityBytes && memoryCapacityBytes > 0) {
    return `${formatter(sample.memory_usage_bytes)} / ${formatter(memoryCapacityBytes)}`;
  }
  return formatter(sample.memory_usage_bytes);
}

function UsageBarRow({
  label,
  value,
  percent,
  fillClass,
  valueClass,
}: {
  label: string;
  value: string;
  percent: number;
  fillClass: string;
  valueClass: string;
}) {
  const hasSample = value !== "-";
  return (
    <div className="space-y-2">
      <div className="flex items-center justify-between gap-3 font-mono text-xs uppercase tracking-[0.12em]">
        <span className="text-zinc-500">{label}</span>
        <span className={valueClass}>{value}</span>
      </div>
      <div className="h-5 border-2 border-zinc-700 bg-black p-[3px]">
        <div
          className={`h-full transition-all duration-700 ${fillClass}`}
          style={{ width: `${hasSample ? percent : 0}%` }}
        />
      </div>
    </div>
  );
}
