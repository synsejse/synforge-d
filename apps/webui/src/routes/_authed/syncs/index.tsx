import { createFileRoute } from "@tanstack/react-router";
import SyncListPage from "../../../features/syncs/sync-list-page";
import type { SyncStatus } from "../../../lib/types";

export type SyncListMode = "runs" | "batches";
export type SyncStatusFilter = "all" | SyncStatus;

export interface SyncListSearch {
  mode?: SyncListMode;
  status?: SyncStatusFilter;
  offset?: number;
  packageFilter?: string;
}

const SYNC_STATUSES = new Set<SyncStatus>([
  "queued",
  "running",
  "succeeded",
  "failed",
  "cancelled",
  "interrupted",
]);

export const Route = createFileRoute("/_authed/syncs/")({
  validateSearch: (search: Record<string, unknown>): SyncListSearch => ({
    mode: search.mode === "batches" ? "batches" : "runs",
    status:
      typeof search.status === "string" && SYNC_STATUSES.has(search.status as SyncStatus)
        ? (search.status as SyncStatus)
        : "all",
    offset: Number(search.offset ?? 0) || 0,
    packageFilter:
      typeof search.packageFilter === "string" ? search.packageFilter : "",
  }),
  component: SyncListPage,
});
