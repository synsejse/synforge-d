import type {
  CacheStatsResponse,
  SyncMetricsResponse,
  SyncOperationListQuery,
  SyncOperationListResponse,
} from "../types";
import { UserApiClient } from "./users";

export class SyncApiClient extends UserApiClient {
  async listSyncOperations(
    query: SyncOperationListQuery = {},
  ): Promise<SyncOperationListResponse> {
    const params = new URLSearchParams();
    if (query.limit !== undefined) {
      params.set("limit", String(query.limit));
    }
    if (query.offset !== undefined) {
      params.set("offset", String(query.offset));
    }
    if (query.package_name?.trim()) {
      params.set("package_name", query.package_name.trim());
    }
    if (query.status) {
      params.set("status", query.status);
    }

    return this.request(
      "GET",
      `/api/v1/sync/operations${params.toString() ? `?${params.toString()}` : ""}`,
    );
  }

  async listPackageSyncOperations(
    packageName: string,
    query: Omit<SyncOperationListQuery, "package_name"> = {},
  ): Promise<SyncOperationListResponse> {
    const params = new URLSearchParams();
    if (query.limit !== undefined) {
      params.set("limit", String(query.limit));
    }
    if (query.offset !== undefined) {
      params.set("offset", String(query.offset));
    }
    if (query.status) {
      params.set("status", query.status);
    }

    return this.request(
      "GET",
      `/api/v1/packages/${encodeURIComponent(packageName)}/sync/operations${params.toString() ? `?${params.toString()}` : ""}`,
    );
  }

  async getSyncMetrics(): Promise<SyncMetricsResponse> {
    return this.request("GET", "/api/v1/sync/metrics");
  }

  async getCacheStats(): Promise<CacheStatsResponse> {
    return this.request("GET", "/api/v1/cache/stats");
  }
}
