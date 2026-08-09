import { createFileRoute } from "@tanstack/react-router";
import SyncDetailPage from "../../../features/syncs/sync-detail-page";

export const Route = createFileRoute("/_authed/syncs/view")({
  validateSearch: (search: Record<string, unknown>) => ({
    id: typeof search.id === "string" ? search.id : undefined,
  }),
  component: SyncView,
});

function SyncView() {
  const { id } = Route.useSearch();
  return id ? (
    <SyncDetailPage operationId={id} />
  ) : (
    <div className="flex min-h-[400px] items-center justify-center font-mono text-sm text-soft">
      No sync ID provided
    </div>
  );
}
