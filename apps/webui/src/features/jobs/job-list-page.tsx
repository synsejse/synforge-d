import { useEffect, useState } from "react";
import { useDebounce } from "../../lib/hooks/use-debounce";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { getRouteApi } from "@tanstack/react-router";
import { faTrash } from "@fortawesome/free-solid-svg-icons";
import api from "../../lib/api";
import { jobsQueries } from "../../lib/queries";
import type { BuildJobResponse } from "../../lib/types";
import {
  HISTORY_BUILD_STATUS_LABELS,
  isHistoryBuildStatus,
  type HistoryBuildStatus,
} from "../../lib/job-status";
import type { JobViewMode } from "./types";

const route = getRouteApi("/_authed/jobs/");
import JobListTable from "./components/job-list-table";
import ErrorMessage from "../../components/common/error-message";
import { useDialogs } from "../../components/common/dialogs-context";
import { useToast } from "../../components/common/toast-context";
import LoadingBlock from "../../components/ui/loading-block";
import FaIcon from "../../components/ui/fa-icon";
import SegmentedControl from "../../components/ui/segmented-control";
import Select from "../../components/ui/select";
import PageHeader from "../../components/ui/page-header";
import PaginationControls from "../../components/common/pagination-controls";

const PAGE_SIZE = 50;
const USAGE_POLL_INTERVAL_MS = 1000;

function JobList() {
  const queryClient = useQueryClient();
  const { confirm } = useDialogs();
  const toast = useToast();
  const navigate = route.useNavigate();
  const search = route.useSearch();
  const filters = {
    mode: search.mode ?? "history",
    filter: search.filter ?? "all",
    offset: search.offset ?? 0,
    packageFilter: search.packageFilter ?? "",
    targetFilter: search.targetFilter ?? "",
    includeDeleted: search.includeDeleted ?? false,
  };
  const [packageInput, setPackageInput] = useState(filters.packageFilter);
  const [targetInput, setTargetInput] = useState(filters.targetFilter);
  const debouncedPackage = useDebounce(packageInput, 250);
  const debouncedTarget = useDebounce(targetInput, 250);

  const setFilters = (update: Partial<typeof search>) =>
    navigate({ search: (prev) => ({ ...prev, ...update }) });

  useEffect(() => {
    if (debouncedPackage !== filters.packageFilter || debouncedTarget !== filters.targetFilter) {
      setFilters({
        packageFilter: debouncedPackage,
        targetFilter: debouncedTarget,
        offset: 0,
      });
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [debouncedPackage, debouncedTarget]);

  const jobsQuery = useQuery({
    ...jobsQueries.list({
      scope: filters.mode === "active" ? "active" : "completed",
      limit: PAGE_SIZE,
      offset: filters.offset,
      status: filters.mode === "active" ? undefined : filters.filter,
      packageName: filters.packageFilter,
      mockChroot: filters.targetFilter,
      includeDeleted: filters.mode === "history" && filters.includeDeleted,
    }),
    // Active mode: refresh the row data every 2s so durations tick and
    // status badges flip live without manual refresh.
    refetchInterval:
      filters.mode === "active" ? USAGE_POLL_INTERVAL_MS * 2 : false,
  });

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
      toast.error(
        "Delete failed",
        error instanceof Error ? error.message : "Failed to delete job",
      ),
  });

  const pruneMutation = useMutation({
    mutationFn: () => api.pruneFailedJobs(),
    onSuccess: invalidateJobs,
    onError: (error) =>
      toast.error(
        "Prune failed",
        error instanceof Error ? error.message : "Failed to prune failed jobs",
      ),
  });

  const killMutation = useMutation({
    mutationFn: (jobId: string) => api.killJob(jobId),
    onSuccess: invalidateJobs,
    onError: (error) =>
      toast.error(
        "Kill failed",
        error instanceof Error ? error.message : "Failed to kill job",
      ),
  });

  const retryMutation = useMutation({
    mutationFn: (jobId: string) => api.retryJob(jobId),
    onSuccess: invalidateJobs,
    onError: (error) =>
      toast.error(
        "Retry failed",
        error instanceof Error ? error.message : "Failed to retry job",
      ),
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

  async function handleRetry(job: BuildJobResponse) {
    const ok = await confirm({
      title: "Retry build?",
      message: `${job.job.package_name} / ${job.job.mock_chroot} will be rebuilt.`,
      confirmLabel: "Retry",
    });
    if (!ok) return;
    retryMutation.mutate(job.job.id);
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
    setFilters({ mode, filter: "all", offset: 0 });
  }

  function setFilter(filter: "all" | HistoryBuildStatus) {
    setFilters({ filter, offset: 0 });
  }

  function setOffset(offset: number) {
    setFilters({ offset });
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

  const loading = jobsQuery.isPending;
  const data = jobsQuery.data;
  const jobs = data?.jobs ?? [];
  const killingJobId =
    killMutation.isPending && killMutation.variables
      ? killMutation.variables
      : null;
  const failedJobsCount = jobs.filter(
    (entry) => entry.job.status === "failed" || entry.job.status === "timed_out",
  ).length;

  const resultCount = data?.page.total ?? jobs.length;

  return (
    <div className="space-y-5">
      <PageHeader
        eyebrow={filters.mode === "active" ? "Active" : "History"}
        title="Jobs"
        description={
          filters.mode === "active"
            ? "Pending and running jobs currently in flight."
            : "Finished job history across all packages and targets."
        }
        color="orange"
      />

      {/* Control bar */}
      <div className="flex flex-wrap items-center gap-2.5 border border-edge bg-black p-4">
        <SegmentedControl<JobViewMode>
          value={filters.mode}
          onChange={setMode}
          ariaLabel="Job view mode"
          size="md"
          items={[
            { value: "history", label: "History", tone: "lime" },
            { value: "active", label: "Active", tone: "lime" },
          ]}
        />

        {filters.mode === "history" && (
          <div className="w-full sm:w-[190px]">
            <Select
              options={[
                { value: "all", label: "All Statuses" },
                ...Object.entries(HISTORY_BUILD_STATUS_LABELS).map(
                  ([value, label]) => ({ value, label }),
                ),
              ]}
              value={filters.filter}
              onValueChange={(val) =>
                setFilter(isHistoryBuildStatus(val) ? val : "all")
              }
              placeholder="Filter status..."
            />
          </div>
        )}

        {filters.mode === "history" && (
          <div className="flex flex-wrap items-center gap-2.5 sm:ml-auto">
            {/* Show-deleted toggle. Off by default — soft-deleted jobs are
                pruned of artifacts/logs but kept so statistics still see
                them; surface them on demand. */}
            <label className="inline-flex cursor-pointer items-center gap-2 border border-edge bg-black px-3 py-2 font-mono text-xs font-semibold uppercase tracking-[0.06em] text-soft transition-colors hover:text-strong">
              <input
                type="checkbox"
                checked={filters.includeDeleted}
                onChange={(e) =>
                  setFilters({
                    includeDeleted: e.target.checked || undefined,
                    offset: 0,
                  })
                }
              />
              Show deleted
            </label>

            <button
              type="button"
              onClick={handlePruneFailed}
              disabled={pruneMutation.isPending || failedJobsCount === 0}
              className="inline-flex items-center gap-2 border border-error bg-error/10 px-3 py-2 font-mono text-xs font-bold uppercase tracking-[0.06em] text-error transition-colors hover:bg-error/20 disabled:pointer-events-none disabled:opacity-40"
            >
              <FaIcon icon={faTrash} className="text-[12px]" />
              Prune Failed
            </button>
          </div>
        )}
      </div>

      {/* Filters (history only) */}
      {filters.mode === "history" && (
        <div className="grid gap-3.5 sm:grid-cols-2">
          <FilterBox
            label="Package"
            value={packageInput}
            onChange={setPackageInput}
            placeholder="Filter by package ..."
          />
          <FilterBox
            label="Target"
            value={targetInput}
            onChange={setTargetInput}
            placeholder="Filter by target ..."
          />
        </div>
      )}

      {/* Result count */}
      {!loading && filters.mode === "history" && (
        <div className="font-mono text-xs font-semibold uppercase tracking-[0.16em] text-[#6b6b73]">
          {resultCount} {resultCount === 1 ? "result" : "results"}
        </div>
      )}

      {loading ? (
        <LoadingBlock label="Loading jobs…" lines={4} />
      ) : (
        <JobListTable
          jobs={jobs}
          killingJobId={killingJobId}
          mode={filters.mode}
          onDelete={(job) => void handleDelete(job)}
          onRetry={(job) => void handleRetry(job)}
          onKill={(job) => void handleKill(job)}
          usageByJob={usageByJob}
        />
      )}

      {data && jobs.length > 0 && (
        <PaginationControls
          offset={filters.offset}
          pageSize={PAGE_SIZE}
          count={jobs.length}
          hasMore={data.page.has_more}
          total={data.page.total}
          isFetching={jobsQuery.isFetching}
          onOffsetChange={setOffset}
        />
      )}
    </div>
  );
}

function FilterBox({
  label,
  value,
  onChange,
  placeholder,
}: {
  label: string;
  value: string;
  onChange: (value: string) => void;
  placeholder: string;
}) {
  return (
    <div className="border border-edge bg-black px-4 py-3.5">
      <div className="font-mono text-xs font-semibold uppercase tracking-[0.22em] text-soft">
        {label}
      </div>
      <input
        type="text"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder={placeholder}
        className="mt-2.5 w-full border border-edge bg-black px-3 py-2.5 font-mono text-xs text-white outline-none transition-colors focus:border-accent-lime"
      />
    </div>
  );
}

export default function JobListPage() {
  return <JobList />;
}
