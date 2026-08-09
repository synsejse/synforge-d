import { useEffect, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { getRouteApi } from "@tanstack/react-router";
import { faRotate } from "@fortawesome/free-solid-svg-icons";
import { useDebounce } from "../../lib/hooks/use-debounce";
import { syncQueries } from "../../lib/queries";
import type { SyncStatus } from "../../lib/types";
import EmptyState from "../../components/ui/empty-state";
import ErrorMessage from "../../components/common/error-message";
import LoadingBlock from "../../components/ui/loading-block";
import PageHeader from "../../components/ui/page-header";
import PaginationControls from "../../components/common/pagination-controls";
import SegmentedControl from "../../components/ui/segmented-control";
import Select from "../../components/ui/select";
import SyncBatchCard from "./components/sync-batch-card";
import SyncRunCard from "./components/sync-run-card";

const route = getRouteApi("/_authed/syncs/");
const PAGE_SIZE = 25;
const STATUS_OPTIONS = [
  { value: "all", label: "All statuses" },
  { value: "queued", label: "Queued" },
  { value: "running", label: "Running" },
  { value: "succeeded", label: "Succeeded" },
  { value: "failed", label: "Failed" },
  { value: "cancelled", label: "Cancelled" },
  { value: "interrupted", label: "Interrupted" },
];

export default function SyncListPage() {
  const navigate = route.useNavigate();
  const search = route.useSearch();
  const filters = {
    mode: search.mode ?? "runs",
    status: search.status ?? "all",
    offset: search.offset ?? 0,
    packageFilter: search.packageFilter ?? "",
  };
  const [packageInput, setPackageInput] = useState(filters.packageFilter);
  const debouncedPackage = useDebounce(packageInput, 250);
  const setSearch = (update: Partial<typeof search>) =>
    navigate({ search: (previous) => ({ ...previous, ...update }) });

  useEffect(() => {
    if (debouncedPackage !== filters.packageFilter) {
      setSearch({ packageFilter: debouncedPackage, offset: 0 });
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [debouncedPackage]);

  const runsQuery = useQuery({
    ...syncQueries.allOperations({
      limit: PAGE_SIZE,
      offset: filters.offset,
      packageName: filters.packageFilter,
      status: filters.status === "all" ? undefined : filters.status,
    }),
    enabled: filters.mode === "runs",
    refetchInterval: filters.mode === "runs" ? 2000 : false,
  });
  const batchesQuery = useQuery({
    ...syncQueries.batches(PAGE_SIZE, filters.offset),
    enabled: filters.mode === "batches",
    refetchInterval: filters.mode === "batches" ? 2000 : false,
  });
  const activeQuery = filters.mode === "runs" ? runsQuery : batchesQuery;
  const error = activeQuery.error;

  if (error) {
    return <ErrorMessage message={error instanceof Error ? error.message : "Failed to load syncs"} />;
  }

  const runs = runsQuery.data?.operations ?? [];
  const batches = batchesQuery.data?.batches ?? [];
  const page = filters.mode === "runs" ? runsQuery.data?.page : batchesQuery.data?.page;
  const count = filters.mode === "runs" ? runs.length : batches.length;

  return (
    <div className="space-y-6">
      <PageHeader
        title="Source Syncs"
        description="Live source inspection, planning stages, refresh batches, logs, and resulting builds."
        color="cyan"
        actions={[{ to: "/packages", label: "Refresh packages", icon: faRotate }]}
      />

      <div className="flex flex-col gap-3 border border-edge bg-black p-4 lg:flex-row lg:items-center">
        <SegmentedControl
          value={filters.mode}
          onChange={(mode) => setSearch({ mode, offset: 0 })}
          ariaLabel="Sync view"
          items={[
            { value: "runs", label: "Runs", tone: "white" },
            { value: "batches", label: "Refresh batches", tone: "white" },
          ]}
        />
        {filters.mode === "runs" ? (
          <>
            <input
              value={packageInput}
              onChange={(event) => setPackageInput(event.target.value)}
              placeholder="Filter package…"
              aria-label="Filter syncs by package"
              className="min-w-0 flex-1 border border-edge bg-black px-4 py-2.5 font-mono text-sm text-white outline-none placeholder:text-soft focus:border-accent-cyan"
            />
            <div className="w-full lg:w-52">
              <Select
                value={filters.status}
                options={STATUS_OPTIONS}
                onValueChange={(value) =>
                  setSearch({ status: value as "all" | SyncStatus, offset: 0 })
                }
              />
            </div>
          </>
        ) : null}
      </div>

      {activeQuery.isPending ? (
        <LoadingBlock label="Loading sync activity…" lines={4} />
      ) : count === 0 ? (
        <EmptyState
          title={filters.mode === "runs" ? "No sync runs" : "No refresh batches"}
          description="Source refresh activity will appear here as soon as it is queued."
        />
      ) : (
        <div className="space-y-3">
          {filters.mode === "runs"
            ? runs.map((operation) => (
                <SyncRunCard key={operation.id} operation={operation} />
              ))
            : batches.map((batch) => <SyncBatchCard key={batch.id} batch={batch} />)}
        </div>
      )}

      {page && count > 0 ? (
        <PaginationControls
          offset={filters.offset}
          pageSize={PAGE_SIZE}
          count={count}
          hasMore={page.has_more}
          total={page.total}
          isFetching={activeQuery.isFetching}
          onOffsetChange={(offset) => setSearch({ offset })}
        />
      ) : null}
    </div>
  );
}
