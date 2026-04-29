import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import api from "../../../lib/api";
import { queryKeys } from "../../../lib/query-keys";
import { formatDateTime } from "../../../lib/datetime";
import type { SyncOperation, SyncStatus } from "../../../lib/types";
import ErrorMessage from "../../../components/common/ErrorMessage";
import PaginationControls from "../../../components/common/PaginationControls";
import Badge from "../../../components/ui/Badge";
import EmptyState from "../../../components/ui/EmptyState";
import LoadingBlock from "../../../components/ui/LoadingBlock";
import Select from "../../../components/ui/Select";

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

export default function SyncHistoryTable({ packageName }: SyncHistoryTableProps) {
  const [offset, setOffset] = useState(0);
  const [status, setStatus] = useState<StatusFilter>("all");

  const operationsQuery = useQuery({
    queryKey: queryKeys.sync.operations(packageName, {
      limit: PAGE_SIZE,
      offset,
      status: status === "all" ? undefined : status,
    }),
    queryFn: () =>
      api.listPackageSyncOperations(packageName, {
        limit: PAGE_SIZE,
        offset,
        status: status === "all" ? undefined : status,
      }),
    placeholderData: (previous) => previous,
  });

  if (operationsQuery.isPending) {
    return <LoadingBlock label="Loading sync history…" lines={4} />;
  }

  if (operationsQuery.error || !operationsQuery.data) {
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

      {operationsQuery.data.operations.length === 0 ? (
        <EmptyState>No sync operations recorded for this package.</EmptyState>
      ) : (
        <div className="space-y-4">
          <div className="space-y-3 md:hidden">
            {operationsQuery.data.operations.map((operation) => (
              <article
                key={`mobile:${operation.id}`}
                className={`border-2 p-4 ${
                  operation.status === "failed"
                    ? "border-[rgba(255,51,51,0.45)] bg-[rgba(255,51,51,0.06)]"
                    : "border-zinc-700 bg-black"
                }`}
              >
                <div className="flex items-start justify-between gap-3">
                  <div className="min-w-0">
                    <div className="font-mono text-sm text-zinc-200">
                      {formatDateTime(operation.created_at)}
                    </div>
                    <div className="mt-1 font-mono text-xs uppercase tracking-[0.14em] text-zinc-500">
                      {formatTrigger(operation.trigger_type)}
                    </div>
                  </div>
                  <Badge variant={operation.status === "failed" ? "error" : "success"}>
                    {operation.status}
                  </Badge>
                </div>
                <div className="mt-3 space-y-2 font-mono text-xs text-zinc-400">
                  <div className="break-all">
                    <span className="text-zinc-500">Revision:</span>{" "}
                    {operation.revision || "—"}
                  </div>
                  <div className="break-all">
                    <span className="text-zinc-500">Error:</span>{" "}
                    {operation.error_message || "—"}
                  </div>
                </div>
              </article>
            ))}
          </div>
          <div className="hidden overflow-x-auto border-2 border-zinc-700 md:block">
            <table className="w-full min-w-[640px] lg:min-w-[980px]">
              <thead className="border-b-2 border-zinc-700 bg-zinc-950 text-left font-mono text-xs uppercase tracking-[0.2em] text-zinc-500">
                <tr>
                  <th className="px-4 py-3">Timestamp</th>
                  <th className="px-4 py-3">Trigger</th>
                  <th className="px-4 py-3">Status</th>
                  <th className="px-4 py-3">Revision</th>
                  <th className="px-4 py-3">Error</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-zinc-800 bg-black">
                {operationsQuery.data.operations.map((operation) => (
                  <tr
                    key={operation.id}
                    className={
                      operation.status === "failed"
                        ? "bg-[rgba(255,51,51,0.04)] hover:bg-[rgba(255,51,51,0.1)]"
                        : "hover:bg-zinc-950"
                    }
                  >
                    <td className="px-4 py-3 font-mono text-sm text-zinc-300">
                      {formatDateTime(operation.created_at)}
                    </td>
                    <td className="px-4 py-3 font-mono text-xs uppercase tracking-[0.14em] text-zinc-400">
                      {formatTrigger(operation.trigger_type)}
                    </td>
                    <td className="px-4 py-3">
                      <Badge variant={operation.status === "failed" ? "error" : "success"}>
                        {operation.status}
                      </Badge>
                    </td>
                    <td className="px-4 py-3 font-mono text-sm text-zinc-300">
                      {operation.revision || "—"}
                    </td>
                    <td className="px-4 py-3 font-mono text-xs text-zinc-400">
                      {operation.error_message || "—"}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>

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
                  {offset + operationsQuery.data.operations.length}
                  {operationsQuery.data.page.total !== null
                    ? ` of ${operationsQuery.data.page.total}`
                    : ""}
                </>
              }
            />
          </div>
        </div>
      )}
    </div>
  );
}
