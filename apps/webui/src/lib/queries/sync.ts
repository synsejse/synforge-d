import { queryOptions } from "@tanstack/react-query";
import api from "../api";
import type { SyncStatus } from "../types";

export interface SyncOperationsParams {
  limit: number;
  offset: number;
  status?: SyncStatus;
}

export const syncQueries = {
  operations: (packageName: string, params: SyncOperationsParams) =>
    queryOptions({
      queryKey: ["sync", "operations", packageName, params] as const,
      queryFn: () =>
        api.listPackageSyncOperations(packageName, {
          limit: params.limit,
          offset: params.offset,
          status: params.status,
        }),
      placeholderData: (previous) => previous,
    }),
  schedule: (limit: number = 10) =>
    queryOptions({
      queryKey: ["sync", "schedule", limit] as const,
      queryFn: () => api.getSyncSchedule(limit),
    }),
};
