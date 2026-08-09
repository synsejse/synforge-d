import { createFileRoute } from "@tanstack/react-router";
import SyncBatchDetailPage from "../../../features/syncs/sync-batch-detail-page";

export const Route = createFileRoute("/_authed/syncs/batch")({
  validateSearch: (search: Record<string, unknown>) => ({
    id: typeof search.id === "string" ? search.id : undefined,
  }),
  component: SyncBatchView,
});

function SyncBatchView() {
  const { id } = Route.useSearch();
  return id ? (
    <SyncBatchDetailPage batchId={id} />
  ) : (
    <div className="flex min-h-[400px] items-center justify-center font-mono text-sm text-soft">
      No batch ID provided
    </div>
  );
}
