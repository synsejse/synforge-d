import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { syncQueries } from "../../../lib/queries";
import type { SyncStatus } from "../../../lib/types";
import EmptyState from "../../../components/ui/empty-state";
import ErrorMessage from "../../../components/common/error-message";
import { SkeletonCardList } from "../../../components/ui/skeleton";
import PaginationControls from "../../../components/common/pagination-controls";
import SegmentedControl from "../../../components/ui/segmented-control";
import SyncOpCard from "./sync-op-card";

interface SyncHistoryTableProps {
  packageName: string;
}

const PAGE_SIZE = 20;

type StatusFilter = "all" | SyncStatus;

const STATUS_FILTERS = [
  { value: "all" as const, label: "All" },
  { value: "succeeded" as const, label: "Succeeded" },
  { value: "failed" as const, label: "Failed" },
];

export default function SyncHistoryTable({ packageName }: SyncHistoryTableProps) {
  const [offset, setOffset] = useState(0);
  const [status, setStatus] = useState<StatusFilter>("all");

  const operationsQuery = useQuery(
    syncQueries.operations(packageName, {
      limit: PAGE_SIZE,
      offset,
      status: status === "all" ? undefined : status,
    }),
  );

  if (operationsQuery.error || (!operationsQuery.isPending && !operationsQuery.data)) {
    return (
      <ErrorMessage
        message={
          operationsQuery.error instanceof Error
            ? operationsQuery.error.message
            : "Failed to load sync history"
        }
      />
    );
  }

  const operations = operationsQuery.data?.operations ?? [];

  return (
    <div className="space-y-4">
      <div className="flex flex-col gap-3 md:flex-row md:items-center md:justify-between">
        <span className="font-mono text-[10px] font-bold uppercase tracking-[0.22em] text-soft">
          Filter status
        </span>
        <SegmentedControl<StatusFilter>
          value={status}
          onChange={(next) => {
            setStatus(next);
            setOffset(0);
          }}
          ariaLabel="Filter sync history by status"
          size="sm"
          items={STATUS_FILTERS}
        />
      </div>

      {operationsQuery.isPending ? (
        <SkeletonCardList count={4} lines={2} />
      ) : operations.length === 0 ? (
        <EmptyState
          title="No sync operations"
          description="No sync operations recorded for this package yet."
        />
      ) : (
        <div className="space-y-3">
          {operations.map((op) => (
            <SyncOpCard key={op.id} op={op} />
          ))}
        </div>
      )}

      {operationsQuery.data && operations.length > 0 ? (
        <div className="border-2 border-edge-strong bg-black px-4 py-3">
          <PaginationControls
            onPrevious={() => setOffset((current) => Math.max(0, current - PAGE_SIZE))}
            onNext={() => setOffset((current) => current + PAGE_SIZE)}
            previousDisabled={operationsQuery.isFetching || offset === 0}
            nextDisabled={
              operationsQuery.isFetching || !operationsQuery.data.page.has_more
            }
            summary={
              <>
                Showing {offset + 1}-{offset + operations.length}
                {operationsQuery.data.page.total !== null
                  ? ` of ${operationsQuery.data.page.total}`
                  : ""}
              </>
            }
          />
        </div>
      ) : null}
    </div>
  );
}
