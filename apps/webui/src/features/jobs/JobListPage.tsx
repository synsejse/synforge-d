import { useEffect, useState } from "react";
import {
  faChartLine,
  faFilter,
  faTrash,
} from "@fortawesome/free-solid-svg-icons";
import api from "../../lib/api";
import type {
  BuildJobResponse,
  JobResourceUsageSample,
} from "../../lib/types";
import {
  HISTORY_BUILD_STATUS_LABELS,
  isHistoryBuildStatus,
  type HistoryBuildStatus,
} from "../../lib/job-status";
import type { JobViewMode } from "./types";
import JobListTable from "./components/JobListTable";
import PageRoot from "../../components/common/PageRoot";
import ErrorMessage from "../../components/common/ErrorMessage";
import { useDialogs } from "../../components/common/DialogsProvider";
import PageVisibilityProvider, {
  usePageVisible,
} from "../../components/common/PageVisibilityProvider";
import ServerHardwareProvider, {
  useServerHardware,
} from "../../components/common/ServerHardwareProvider";
import LoadingBlock from "../../components/ui/LoadingBlock";
import FaIcon from "../../components/ui/FaIcon";
import Button from "../../components/ui/Button";
import Select from "../../components/ui/Select";
import PageHeader from "../../components/ui/PageHeader";

const USAGE_POLL_INTERVAL_MS = 1000;

function normalizeStatusFilter(value: string | null): "all" | HistoryBuildStatus {
  if (!value || value === "all") {
    return "all";
  }
  return isHistoryBuildStatus(value) ? value : "all";
}

function JobList() {
  const { confirm, notify } = useDialogs();
  const pageVisible = usePageVisible();
  const serverHardware = useServerHardware();
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
  const [usageByJob, setUsageByJob] = useState<Record<string, JobResourceUsageSample>>({});
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
    const ok = await confirm({
      title: "Delete job?",
      message: `Job ${job.job.id} will be removed.`,
      confirmLabel: "Delete",
      destructive: true,
    });
    if (!ok) return;
    try {
      await api.deleteJob(job.job.id);
      await load();
    } catch (e) {
      await notify({
        title: "Delete failed",
        message: e instanceof Error ? e.message : "Failed to delete job",
        variant: "error",
      });
    }
  }

  async function handlePruneFailed() {
    const failedCount = jobs.filter(
      (entry) => entry.job.status === "failed" || entry.job.status === "timed_out",
    ).length;
    if (failedCount === 0) return;
    const ok = await confirm({
      title: "Prune failed jobs?",
      message: `Delete ${failedCount} failed or timed out jobs?`,
      confirmLabel: "Delete",
      destructive: true,
    });
    if (!ok) return;
    try {
      setPruning(true);
      await api.pruneFailedJobs();
      await load();
    } catch (e) {
      await notify({
        title: "Prune failed",
        message: e instanceof Error ? e.message : "Failed to prune failed jobs",
        variant: "error",
      });
    } finally {
      setPruning(false);
    }
  }

  async function handleKill(job: BuildJobResponse) {
    const ok = await confirm({
      title: "Kill active job?",
      message: `${job.job.package_name} / ${job.job.mock_chroot} (${job.job.id})`,
      confirmLabel: "Kill",
      destructive: true,
    });
    if (!ok) return;
    try {
      setKillingJobId(job.job.id);
      await api.killJob(job.job.id);
      await load();
    } catch (e) {
      await notify({
        title: "Kill failed",
        message: e instanceof Error ? e.message : "Failed to kill job",
        variant: "error",
      });
    } finally {
      setKillingJobId(null);
    }
  }

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

        <JobListTable
          jobs={jobs}
          killingJobId={killingJobId}
          mode={mode}
          onDelete={(job) => void handleDelete(job)}
          onKill={(job) => void handleKill(job)}
          serverHardware={serverHardware}
          usageByJob={usageByJob}
        />

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

export default function JobListPage() {
  return (
    <PageRoot>
      <PageVisibilityProvider>
        <ServerHardwareProvider>
          <JobList />
        </ServerHardwareProvider>
      </PageVisibilityProvider>
    </PageRoot>
  );
}
