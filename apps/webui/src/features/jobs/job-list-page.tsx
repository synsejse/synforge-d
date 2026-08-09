import { useEffect, useState } from "react";
import { useDebounce } from "../../lib/hooks/use-debounce";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { getRouteApi } from "@tanstack/react-router";
import api from "../../lib/api";
import { jobsQueries } from "../../lib/queries";
import type { BuildJobResponse } from "../../lib/types";
import type { HistoryBuildStatus } from "../../lib/job-status";
import type { JobViewMode } from "./types";

const route = getRouteApi("/_authed/jobs/");
import JobListTable from "./components/job-list-table";
import ErrorMessage from "../../components/common/error-message";
import { useDialogs } from "../../components/common/dialogs-context";
import { useToast } from "../../components/common/toast-context";
import LoadingBlock from "../../components/ui/loading-block";
import SegmentedControl from "../../components/ui/segmented-control";
import PageHeader from "../../components/ui/page-header";
import PaginationControls from "../../components/common/pagination-controls";
import EmptyState from "../../components/ui/empty-state";
import Button from "../../components/ui/button";
import JobListFilters from "./components/job-list-filters";

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
    onSuccess: (response) => {
      void invalidateJobs();
      const count = response.deleted_jobs.length;
      toast.success(
        count === 0 ? "Nothing to prune" : "Failed jobs pruned",
        `${count} ${count === 1 ? "job" : "jobs"} removed.`,
      );
    },
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
    const ok = await confirm({
      title: "Prune failed jobs?",
      message: "Delete all failed or timed out jobs, including results outside this page?",
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
    setFilters({ mode, filter: "all", includeDeleted: undefined, offset: 0 });
  }

  function setFilter(filter: "all" | HistoryBuildStatus) {
    setFilters({ filter, offset: 0 });
  }

  function setOffset(offset: number) {
    setFilters({ offset });
  }

  function clearFilters() {
    setPackageInput("");
    setTargetInput("");
    setFilters({
      filter: "all",
      packageFilter: "",
      targetFilter: "",
      includeDeleted: undefined,
      offset: 0,
    });
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
  const resultCount = data?.page.total ?? jobs.length;
  const hasActiveFilters =
    filters.packageFilter.trim().length > 0 ||
    filters.targetFilter.trim().length > 0 ||
    (filters.mode === "history" &&
      (filters.filter !== "all" || filters.includeDeleted));
  const activeFilterCount =
    Number(filters.packageFilter.trim().length > 0) +
    Number(filters.targetFilter.trim().length > 0) +
    Number(filters.mode === "history" && filters.filter !== "all") +
    Number(filters.mode === "history" && filters.includeDeleted);

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

      <div className="border border-edge bg-black p-4">
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
      </div>

      <JobListFilters
        mode={filters.mode}
        status={filters.filter}
        packageValue={packageInput}
        targetValue={targetInput}
        includeDeleted={filters.includeDeleted}
        activeCount={activeFilterCount}
        pruning={pruneMutation.isPending}
        onPackageChange={setPackageInput}
        onTargetChange={setTargetInput}
        onStatusChange={setFilter}
        onIncludeDeletedChange={(includeDeleted) =>
          setFilters({ includeDeleted: includeDeleted || undefined, offset: 0 })
        }
        onClear={clearFilters}
        onPrune={() => void handlePruneFailed()}
      />

      {/* Result count */}
      {!loading && filters.mode === "history" && (
        <div className="font-mono text-xs font-semibold uppercase tracking-[0.16em] text-[#6b6b73]">
          {resultCount} {resultCount === 1 ? "result" : "results"}
        </div>
      )}

      {loading ? (
        <LoadingBlock label="Loading jobs…" lines={4} />
      ) : jobs.length === 0 ? (
        <EmptyState
          title={
            hasActiveFilters
              ? "No matching jobs"
              : filters.mode === "active"
                ? "No active jobs"
                : "No job history"
          }
          description={
            hasActiveFilters
              ? "Try different package, target, or status filters."
              : filters.mode === "active"
                ? "Queued and running builds will appear here."
                : "Completed builds will appear here."
          }
          action={
            hasActiveFilters ? (
              <Button variant="subtle" onClick={clearFilters}>
                Clear filters
              </Button>
            ) : undefined
          }
        />
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

export default function JobListPage() {
  return <JobList />;
}
