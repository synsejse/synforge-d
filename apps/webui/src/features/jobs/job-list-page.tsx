import { useEffect, useState } from "react";
import { useDebounce } from "../../lib/hooks/use-debounce";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { getRouteApi } from "@tanstack/react-router";
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

const route = getRouteApi("/_authed/jobs/");
import JobListTable from "./components/job-list-table";
import ErrorMessage from "../../components/common/error-message";
import { useDialogs } from "../../components/common/dialogs-provider";
import { useToast } from "../../components/common/toast-provider";
import { useServerHardware } from "../../components/common/server-hardware-provider";
import LoadingBlock from "../../components/ui/loading-block";
import FaIcon from "../../components/ui/fa-icon";
import Button from "../../components/ui/button";
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
  const serverHardware = useServerHardware();
  const navigate = route.useNavigate();
  const search = route.useSearch();
  const filters = {
    mode: search.mode ?? "history",
    filter: search.filter ?? "all",
    offset: search.offset ?? 0,
    packageFilter: search.packageFilter ?? "",
    targetFilter: search.targetFilter ?? "",
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
    setFilters({ mode, filter: "all", offset: 0 });
  }

  function setFilter(filter: "all" | HistoryBuildStatus) {
    setFilters({ filter, offset: 0 });
  }

  function applyTextFilters() {
    setFilters({
      packageFilter: packageInput,
      targetFilter: targetInput,
      offset: 0,
    });
  }

  function clearAllFilters() {
    setPackageInput("");
    setTargetInput("");
    setFilters({
      filter: "all",
      packageFilter: "",
      targetFilter: "",
      offset: 0,
    });
  }

  const activeFilterCount =
    (filters.filter !== "all" ? 1 : 0) +
    (filters.packageFilter ? 1 : 0) +
    (filters.targetFilter ? 1 : 0);

  function setOffset(offset: number) {
    setFilters({ offset });
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
        title={filters.mode === "active" ? "Active Builds" : "Build Timeline"}
        description={
          filters.mode === "active"
            ? "Pending and running jobs currently in flight."
            : "Finished job history across all packages and targets."
        }
        color="orange"
        actions={[{ to: "/", label: "Overview", icon: faChartLine }]}
      />

      {/* Mode Toggle + Filters */}
      <div className="border-4 border-edge-strong bg-black shadow-card-sm">
        <div className="border-b-4 border-edge-strong app-section-band px-6 py-4">
          <div className="flex flex-wrap items-center gap-4">
            {/* Mode Toggle */}
            <SegmentedControl<JobViewMode>
              value={filters.mode}
              onChange={setMode}
              ariaLabel="Job view mode"
              size="lg"
              items={[
                { value: "history", label: "History", tone: "lime" },
                { value: "active", label: "Active", tone: "success" },
              ]}
            />


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
                  onValueChange={(val) =>
                    setFilter(isHistoryBuildStatus(val) ? val : "all")
                  }
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
        <div className="border-b-4 border-edge-strong bg-black px-6 py-4">
          <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
            <div>
              <label className="mb-2 block font-mono text-xs font-bold uppercase tracking-wider text-soft">
                Package
              </label>
              <input
                type="text"
                value={packageInput}
                onChange={(e) => setPackageInput(e.target.value)}
                onKeyDown={(e) => e.key === "Enter" && applyTextFilters()}
                placeholder="Filter by package..."
                className="w-full border-2 border-edge-strong bg-black px-4 py-2.5 font-mono text-sm text-white transition focus:border-accent-lime focus:outline-none focus:ring-2 focus:ring-accent-lime focus:ring-offset-2 focus:ring-offset-black"
              />
            </div>
            <div>
              <label className="mb-2 block font-mono text-xs font-bold uppercase tracking-wider text-soft">
                Target
              </label>
              <input
                type="text"
                value={targetInput}
                onChange={(e) => setTargetInput(e.target.value)}
                onKeyDown={(e) => e.key === "Enter" && applyTextFilters()}
                placeholder="Filter by target..."
                className="w-full border-2 border-edge-strong bg-black px-4 py-2.5 font-mono text-sm text-white transition focus:border-accent-lime focus:outline-none focus:ring-2 focus:ring-accent-lime focus:ring-offset-2 focus:ring-offset-black"
              />
            </div>
            <div className="flex flex-col items-stretch gap-2 sm:flex-row sm:items-end">
              <Button variant="primary" className="w-full sm:flex-1" onClick={applyTextFilters}>
                <FaIcon icon={faFilter} />
                Apply Filters
              </Button>
              {activeFilterCount > 0 ? (
                <Button
                  variant="ghost"
                  className="w-full sm:w-auto"
                  onClick={clearAllFilters}
                  aria-label="Clear all filters"
                  title="Clear all filters"
                >
                  Clear ({activeFilterCount})
                </Button>
              ) : null}
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
          <div className="border-t-4 border-edge-strong bg-surface-alt px-6 py-4">
            <PaginationControls
              onPrevious={() => setOffset(Math.max(0, filters.offset - PAGE_SIZE))}
              onNext={() => setOffset(filters.offset + PAGE_SIZE)}
              previousDisabled={filters.offset === 0}
              nextDisabled={!jobsQuery.data.page.has_more}
              summary={`Showing ${filters.offset + 1}-${filters.offset + jobsQuery.data.jobs.length}`}
            />
          </div>
        )}
      </div>
    </div>
  );
}

export default function JobListPage() {
  return <JobList />;
}
