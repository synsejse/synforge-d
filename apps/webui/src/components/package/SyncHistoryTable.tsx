import { useEffect, useState } from "react";
import api from "../../lib/api";
import { formatDateTime } from "../../lib/datetime";
import type { SyncOperation, SyncStatus } from "../../lib/types";
import ErrorMessage from "../common/ErrorMessage";
import PaginationControls from "../common/PaginationControls";
import Badge from "../ui/Badge";
import EmptyState from "../ui/EmptyState";
import LoadingBlock from "../ui/LoadingBlock";
import Select from "../ui/Select";

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
  const [operations, setOperations] = useState<SyncOperation[]>([]);
  const [offset, setOffset] = useState(0);
  const [hasMore, setHasMore] = useState(false);
  const [total, setTotal] = useState<number | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [status, setStatus] = useState<StatusFilter>("all");

  useEffect(() => {
    let cancelled = false;

    async function load() {
      try {
        setLoading(true);
        const response = await api.listPackageSyncOperations(packageName, {
          limit: PAGE_SIZE,
          offset,
          status: status === "all" ? undefined : status,
        });
        if (cancelled) {
          return;
        }
        setOperations(response.operations);
        setHasMore(response.page.has_more);
        setTotal(response.page.total ?? null);
        setError(null);
      } catch (e) {
        if (cancelled) {
          return;
        }
        setError(e instanceof Error ? e.message : "Failed to load sync history");
      } finally {
        if (!cancelled) {
          setLoading(false);
        }
      }
    }

    load();

    return () => {
      cancelled = true;
    };
  }, [packageName, offset, status]);

  if (loading && operations.length === 0) {
    return <LoadingBlock label="Loading sync history…" lines={4} />;
  }

  if (error) {
    return <ErrorMessage message={error} />;
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

      {operations.length === 0 ? (
        <EmptyState>No sync operations recorded for this package.</EmptyState>
      ) : (
        <div className="space-y-4">
          <div className="overflow-x-auto border-2 border-zinc-700">
            <table className="min-w-[980px] w-full">
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
                {operations.map((operation) => (
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
              previousDisabled={loading || offset === 0}
              nextDisabled={loading || !hasMore}
              summary={
                <>
                  Showing {offset + 1}-{offset + operations.length}
                  {total !== null ? ` of ${total}` : ""}
                </>
              }
            />
          </div>
        </div>
      )}
    </div>
  );
}
