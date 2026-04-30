import { useEffect, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  faChartLine,
  faFilter,
  faTrash,
} from "@fortawesome/free-solid-svg-icons";
import api from "../../lib/api";
import { jobsQueries } from "../../lib/queries";
import type { BuildJobResponse } from "../../lib/types";
import {
  HISTORY_BUILD_STATUS_LABELS,
  isHistoryBuildStatus,
  type HistoryBuildStatus,
} from "../../lib/job-status";
import type { JobViewMode } from "./types";
import JobListTable from "./components/JobListTable";
import ErrorMessage from "../../components/common/ErrorMessage";
import { useDialogs } from "../../components/common/DialogsProvider";
import { useServerHardware } from "../../components/common/ServerHardwareProvider";
import LoadingBlock from "../../components/ui/LoadingBlock";
import FaIcon from "../../components/ui/FaIcon";
import Button from "../../components/ui/Button";
import Select from "../../components/ui/Select";
import PageHeader from "../../components/ui/PageHeader";

const PAGE_SIZE = 50;
const USAGE_POLL_INTERVAL_MS = 1000;

interface JobsFilterState {
  mode: JobViewMode;
  filter: "all" | HistoryBuildStatus;
  offset: number;
  packageFilter: string;
  targetFilter: string;
}

function normalizeStatusFilter(value: string | null): "all" | HistoryBuildStatus {
  if (!value || value === "all") {
    return "all";
  }
  return isHistoryBuildStatus(value) ? value : "all";
}

function readInitialFilters(): JobsFilterState {
  if (typeof window === "undefined") {
    return {
      mode: "history",
      filter: "all",
      offset: 0,
      packageFilter: "",
      targetFilter: "",
    };
  }
  const params = new URLSearchParams(window.location.search);
  return {
    mode: params.get("mode") === "active" ? "active" : "history",
    filter: normalizeStatusFilter(params.get("status")),
    offset: Number(params.get("offset") || "0"),
    packageFilter: params.get("package") || "",
    targetFilter: params.get("target") || "",
  };
}

function syncUrl(state: JobsFilterState) {
  if (typeof window === "undefined") return;
  const params = new URLSearchParams();
  if (state.mode !== "history") params.set("mode", state.mode);
  if (state.filter !== "all" && state.mode === "history") {
    params.set("status", state.filter);
  }
  if (state.offset > 0) params.set("offset", String(state.offset));
  if (state.packageFilter.trim()) params.set("package", state.packageFilter.trim());
  if (state.targetFilter.trim()) params.set("target", state.targetFilter.trim());
  const query = params.toString();
  window.history.replaceState({}, "", `/jobs/${query ? `?${query}` : ""}`);
}

function JobList() {
  const queryClient = useQueryClient();
  const { confirm, notify } = useDialogs();
  const serverHardware = useServerHardware();

  const initial = readInitialFilters();
  const [filters, setFilters] = useState<JobsFilterState>(initial);
  const [packageInput, setPackageInput] = useState(initial.packageFilter);
  const [targetInput, setTargetInput] = useState(initial.targetFilter);

  useEffect(() => {
    syncUrl(filters);
  }, [filters]);

  const jobsQuery = useQuery(
    jobsQueries.list({
      scope: filters.mode === "active" ? "active" : "completed",
      limit: PAGE_SIZE,
      offset: filters.offset,
      status: filters.mode === "active" ? undefined : filters.filter,
      packageName: filters.packageFilter,
      mockChroot: filters.targetFilter,
    }),
  );

  const usageQuery = useQuery({
    ...jobsQueries.usageList(),
    enabled: filters.mode === "active",
    refetchInterval: filters.mode === "active" ? USAGE_POLL_INTERVAL_MS : false,
  });

  const usageByJob = (() => {
    if (filters.mode !== "active") return {};
    const map: Record<string, NonNullable<typeof usageQuery.data>["samples"][number]> =
      {};
    for (const sample of usageQuery.data?.samples ?? []) {
      map[sample.job_id] = sample;
    }
    return map;
  })();

  const invalidateJobs = () =>
    queryClient.invalidateQueries({ queryKey: ["jobs"] });

  const deleteMutation = useMutation({
    mutationFn: (jobId: string) => api.deleteJob(jobId),
    onSuccess: invalidateJobs,
    onError: (error) =>
      notify({
        title: "Delete failed",
        message: error instanceof Error ? error.message : "Failed to delete job",
        variant: "error",
      }),
  });

  const pruneMutation = useMutation({
    mutationFn: () => api.pruneFailedJobs(),
    onSuccess: invalidateJobs,
    onError: (error) =>
      notify({
        title: "Prune failed",
        message:
          error instanceof Error ? error.message : "Failed to prune failed jobs",
        variant: "error",
      }),
  });

  const killMutation = useMutation({
    mutationFn: (jobId: string) => api.killJob(jobId),
    onSuccess: invalidateJobs,
    onError: (error) =>
      notify({
        title: "Kill failed",
        message: error instanceof Error ? error.message : "Failed to kill job",
        variant: "error",
      }),
  });

  async function handleDelete(job: BuildJobResponse) {
    const ok = await confirm({
      title: "Delete job?",
      message: `Job ${job.job.id} will be removed.`,
      confirmLabel: "Delete",
      destructive: true,
    });
    if (!ok) return;
    deleteMutation.mutate(job.job.id);
  }

  async function handlePruneFailed() {
    const failedCount = (jobsQuery.data?.jobs ?? []).filter(
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
    pruneMutation.mutate();
  }

  async function handleKill(job: BuildJobResponse) {
    const ok = await confirm({
      title: "Kill active job?",
      message: `${job.job.package_name} / ${job.job.mock_chroot} (${job.job.id})`,
      confirmLabel: "Kill",
      destructive: true,
    });
    if (!ok) return;
    killMutation.mutate(job.job.id);
  }

  function setMode(mode: JobViewMode) {
    setFilters({ ...filters, mode, filter: "all", offset: 0 });
  }

  function setFilter(filter: "all" | HistoryBuildStatus) {
    setFilters({ ...filters, filter, offset: 0 });
  }

  function applyTextFilters() {
    setFilters({
      ...filters,
      packageFilter: packageInput,
      targetFilter: targetInput,
      offset: 0,
    });
  }

  function setOffset(offset: number) {
    setFilters({ ...filters, offset });
  }

  if (jobsQuery.isPending) {
    return <LoadingBlock label="Loading jobs…" lines={4} />;
  }

  if (jobsQuery.error) {
    return (
      <ErrorMessage
        message={
          jobsQuery.error instanceof Error
            ? jobsQuery.error.message
            : "Failed to load jobs"
        }
      />
    );
  }

  const killingJobId =
    killMutation.isPending && killMutation.variables
      ? killMutation.variables
      : null;
  const failedJobsCount = jobsQuery.data.jobs.filter(
    (entry) => entry.job.status === "failed" || entry.job.status === "timed_out",
  ).length;

  return (
    <div className="space-y-6">
      {/* Header */}
      <PageHeader
        eyebrow="JOB_ACTIVITY"
        title={filters.mode === "active" ? "Active Builds" : "Build Timeline"}
        description={
          filters.mode === "active"
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
                onClick={() => setMode("history")}
                className={`px-5 py-2.5 font-mono text-sm font-bold uppercase tracking-wider transition-all ${
                  filters.mode === "history"
                    ? "bg-[var(--theme-accent-lime)] text-black"
                    : "bg-black text-[var(--theme-text-muted)] hover:text-white"
                }`}
              >
                History
              </button>
              <button
                onClick={() => setMode("active")}
                className={`border-l-2 border-[var(--theme-border-strong)] px-5 py-2.5 font-mono text-sm font-bold uppercase tracking-wider transition-all ${
                  filters.mode === "active"
                    ? "bg-[var(--theme-terminal-green)] text-black"
                    : "bg-black text-[var(--theme-text-muted)] hover:text-white"
                }`}
              >
                Active
              </button>
            </div>

            {/* Status Filter (history only) */}
            {filters.mode === "history" && (
              <div className="w-full sm:flex-1 sm:min-w-[200px] sm:max-w-xs">
                <Select
                  options={[
                    { value: "all", label: "All Statuses" },
                    ...Object.entries(HISTORY_BUILD_STATUS_LABELS).map(
                      ([value, label]) => ({ value, label }),
                    ),
                  ]}
                  value={filters.filter}
                  onValueChange={(val) => setFilter(normalizeStatusFilter(val))}
                  placeholder="Filter status..."
                />
              </div>
            )}

            {/* Prune Button */}
            {filters.mode === "history" && (
              <Button
                variant="danger"
                size="sm"
                onClick={handlePruneFailed}
                disabled={pruneMutation.isPending || failedJobsCount === 0}
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
                value={packageInput}
                onChange={(e) => setPackageInput(e.target.value)}
                onKeyDown={(e) => e.key === "Enter" && applyTextFilters()}
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
                value={targetInput}
                onChange={(e) => setTargetInput(e.target.value)}
                onKeyDown={(e) => e.key === "Enter" && applyTextFilters()}
                placeholder="Filter by target..."
                className="w-full border-2 border-[var(--theme-border-strong)] bg-black px-4 py-2.5 font-mono text-sm text-white transition focus:border-[var(--theme-accent-lime)] focus:outline-none focus:ring-2 focus:ring-[var(--theme-accent-lime)] focus:ring-offset-2 focus:ring-offset-black"
              />
            </div>
            <div className="flex items-end">
              <Button variant="primary" className="w-full" onClick={applyTextFilters}>
                <FaIcon icon={faFilter} />
                Apply Filters
              </Button>
            </div>
          </div>
        </div>

        <JobListTable
          jobs={jobsQuery.data.jobs}
          killingJobId={killingJobId}
          mode={filters.mode}
          onDelete={(job) => void handleDelete(job)}
          onKill={(job) => void handleKill(job)}
          serverHardware={serverHardware}
          usageByJob={usageByJob}
        />

        {/* Pagination */}
        {(filters.offset > 0 || jobsQuery.data.page.has_more) && (
          <div className="border-t-4 border-[var(--theme-border-strong)] bg-zinc-950 px-6 py-4">
            <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
              <div className="font-mono text-sm text-zinc-500">
                Showing {filters.offset + 1}-{filters.offset + jobsQuery.data.jobs.length}
              </div>
              <div className="flex w-full gap-3 sm:w-auto">
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => setOffset(Math.max(0, filters.offset - PAGE_SIZE))}
                  disabled={filters.offset === 0}
                >
                  ← Previous
                </Button>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => setOffset(filters.offset + PAGE_SIZE)}
                  disabled={!jobsQuery.data.page.has_more}
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
  return <JobList />;
}
