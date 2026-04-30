import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { syncQueries } from "../../../lib/queries";
import { formatDateTime } from "../../../lib/datetime";
import type { SyncOperation, SyncStatus } from "../../../lib/types";
import ErrorMessage from "../../../components/common/error-message";
import PaginationControls from "../../../components/common/pagination-controls";
import Badge from "../../../components/ui/badge";
import DataTable, { type DataTableColumn } from "../../../components/ui/data-table";
import Select from "../../../components/ui/select";

interface SyncHistoryTableProps {
  packageName: string;
}

const PAGE_SIZE = 20;

type StatusFilter = "all" | SyncStatus;

const statusOptions = [
  { value: "all", label: "All" },
  { value: "succeeded", label: "Succeeded" },
  { value: "failed", label: "Failed" },
];

function formatTrigger(trigger: SyncOperation["trigger_type"]): string {
  return trigger.replaceAll("_", " ");
}

const columns: DataTableColumn<SyncOperation>[] = [
  {
    key: "timestamp",
    header: "Timestamp",
    mobile: "title",
    cell: (op) => (
      <div>
        <div className="font-mono text-sm text-[var(--theme-text-strong)]">
          {formatDateTime(op.created_at)}
        </div>
        <div className="mt-1 font-mono text-xs uppercase tracking-[0.14em] text-[var(--theme-text-soft)] md:hidden">
          {formatTrigger(op.trigger_type)}
        </div>
      </div>
    ),
  },
  {
    key: "trigger",
    header: "Trigger",
    mobile: "hidden",
    className: "font-mono text-xs uppercase tracking-[0.14em] text-[var(--theme-text-muted)]",
    cell: (op) => formatTrigger(op.trigger_type),
  },
  {
    key: "status",
    header: "Status",
    mobile: "badge",
    cell: (op) => (
      <Badge variant={op.status === "failed" ? "error" : "success"}>
        {op.status}
      </Badge>
    ),
  },
  {
    key: "revision",
    header: "Revision",
    className: "font-mono text-sm text-[var(--theme-text-strong)]",
    cell: (op) => op.revision || "—",
  },
  {
    key: "error",
    header: "Error",
    className: "font-mono text-xs text-[var(--theme-text-muted)]",
    cell: (op) => op.error_message || "—",
  },
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
        <div className="font-mono text-xs uppercase tracking-[0.18em] text-zinc-500">
          Filter status
        </div>
        <div className="w-full md:w-56">
          <Select
            options={statusOptions}
            value={status}
            onValueChange={(value) => {
              const next = value as StatusFilter;
              setStatus(next);
              setOffset(0);
            }}
          />
        </div>
      </div>

      <DataTable
        columns={columns}
        rows={operations}
        rowKey={(op) => op.id}
        loading={operationsQuery.isPending}
        empty={{
          title: "No sync operations",
          description: "No sync operations recorded for this package yet.",
        }}
        rowClassName={(op) =>
          op.status === "failed" ? "bg-[rgba(255,51,51,0.04)]" : ""
        }
      />

      {operationsQuery.data && operations.length > 0 ? (
        <div className="border-2 border-zinc-700 bg-black px-4 py-3">
          <PaginationControls
            onPrevious={() => setOffset((current) => Math.max(0, current - PAGE_SIZE))}
            onNext={() => setOffset((current) => current + PAGE_SIZE)}
            previousDisabled={operationsQuery.isFetching || offset === 0}
            nextDisabled={
              operationsQuery.isFetching || !operationsQuery.data.page.has_more
            }
            summary={
              <>
                Showing {offset + 1}-
                {offset + operations.length}
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
