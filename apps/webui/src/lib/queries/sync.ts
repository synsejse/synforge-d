import { queryOptions } from "@tanstack/react-query";
import api from "../api";
import type { SyncStatus } from "../types";

export interface SyncOperationsParams {
  limit: number;
  offset: number;
  status?: SyncStatus;
}

export const syncQueries = {
  allOperations: (params: SyncOperationsParams & { packageName?: string }) =>
    queryOptions({
      queryKey: ["sync", "operations", "all", params] as const,
      queryFn: () =>
        api.listSyncOperations({
          limit: params.limit,
          offset: params.offset,
          status: params.status,
          package_name: params.packageName,
        }),
      placeholderData: (previous) => previous,
    }),
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
  detail: (id: string) =>
    queryOptions({
      queryKey: ["sync", "detail", id] as const,
      queryFn: () => api.getSyncOperation(id),
    }),
  batches: (limit = 25, offset = 0) =>
    queryOptions({
      queryKey: ["sync", "batches", limit, offset] as const,
      queryFn: () => api.listSyncBatches(limit, offset),
      placeholderData: (previous) => previous,
    }),
  batch: (id: string) =>
    queryOptions({
      queryKey: ["sync", "batch", id] as const,
      queryFn: () => api.getSyncBatch(id),
    }),
  schedule: (limit: number = 10) =>
    queryOptions({
      queryKey: ["sync", "schedule", limit] as const,
      queryFn: () => api.getSyncSchedule(limit),
    }),
};
