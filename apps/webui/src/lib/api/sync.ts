import type {
  CacheStatsResponse,
  SyncMetricsResponse,
  SyncOperationListQuery,
  SyncOperationListResponse,
  TimeRange,
  TimeSeriesResponse,
} from "../types";
import { request } from "./client";

export function listSyncOperations(
  query: SyncOperationListQuery = {},
): Promise<SyncOperationListResponse> {
  const params = new URLSearchParams();
  if (query.limit !== undefined) params.set("limit", String(query.limit));
  if (query.offset !== undefined) params.set("offset", String(query.offset));
  if (query.package_name?.trim()) {
    params.set("package_name", query.package_name.trim());
  }
  if (query.status) params.set("status", query.status);

  return request(
    "GET",
    `/api/v1/sync/operations${params.toString() ? `?${params.toString()}` : ""}`,
  );
}

export function listPackageSyncOperations(
  packageName: string,
  query: Omit<SyncOperationListQuery, "package_name"> = {},
): Promise<SyncOperationListResponse> {
  const params = new URLSearchParams();
  if (query.limit !== undefined) params.set("limit", String(query.limit));
  if (query.offset !== undefined) params.set("offset", String(query.offset));
  if (query.status) params.set("status", query.status);

  return request(
    "GET",
    `/api/v1/packages/${encodeURIComponent(packageName)}/sync/operations${
      params.toString() ? `?${params.toString()}` : ""
    }`,
  );
}

export function getSyncMetrics(): Promise<SyncMetricsResponse> {
  return request("GET", "/api/v1/sync/metrics");
}

export function getSyncTimeseries(
  range: TimeRange = "24h",
): Promise<TimeSeriesResponse> {
  return request("GET", `/api/v1/sync/timeseries?range=${range}`);
}

export function getCacheStats(): Promise<CacheStatsResponse> {
  return request("GET", "/api/v1/cache/stats");
}
