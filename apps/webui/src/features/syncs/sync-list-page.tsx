import { useEffect, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { getRouteApi } from "@tanstack/react-router";
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
import FilterBar from "../../components/common/filter-bar";
import Button from "../../components/ui/button";

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

  const clearFilters = () => {
    setPackageInput("");
    setSearch({ packageFilter: "", status: "all", offset: 0 });
  };

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
  const hasActiveFilters =
    filters.mode === "runs" &&
    (filters.packageFilter.trim().length > 0 || filters.status !== "all");
  const activeFilterCount =
    Number(filters.packageFilter.trim().length > 0) +
    Number(filters.status !== "all");

  return (
    <div className="space-y-6">
      <PageHeader
        title="Syncs"
        description="Live source inspection, planning stages, refresh batches, logs, and resulting builds."
        color="cyan"
      />

      <div className="border border-edge bg-black p-4">
        <SegmentedControl
          value={filters.mode}
          onChange={(mode) => setSearch({ mode, offset: 0 })}
          ariaLabel="Sync view"
          items={[
            { value: "runs", label: "Runs", tone: "white" },
            { value: "batches", label: "Refresh batches", tone: "white" },
          ]}
        />
      </div>

      {filters.mode === "runs" ? (
        <FilterBar activeCount={activeFilterCount} onClear={clearFilters}>
          <div className="grid items-end gap-4 sm:grid-cols-[minmax(0,1fr)_220px]">
            <label className="block">
              <span className="block font-mono text-xs font-semibold uppercase tracking-[0.22em] text-soft">
                Package
              </span>
              <input
                value={packageInput}
                onChange={(event) => setPackageInput(event.target.value)}
                placeholder="Filter by package"
                className="mt-2.5 w-full border border-edge bg-black px-4 py-2.5 font-mono text-sm text-white outline-none placeholder:text-soft focus:border-accent-cyan"
              />
            </label>
            <label className="block">
              <span className="block font-mono text-xs font-semibold uppercase tracking-[0.22em] text-soft">
                Status
              </span>
              <div className="mt-2.5">
                <Select
                  value={filters.status}
                  options={STATUS_OPTIONS}
                  onValueChange={(value) =>
                    setSearch({ status: value as "all" | SyncStatus, offset: 0 })
                  }
                />
              </div>
            </label>
          </div>
        </FilterBar>
      ) : null}

      {activeQuery.isPending ? (
        <LoadingBlock label="Loading sync activity…" lines={4} />
      ) : count === 0 ? (
        <EmptyState
          title={
            hasActiveFilters
              ? "No matching syncs"
              : filters.mode === "runs"
                ? "No sync runs"
                : "No refresh batches"
          }
          description={
            hasActiveFilters
              ? "Try a different package or status filter."
              : "Source refresh activity will appear here as soon as it is queued."
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
