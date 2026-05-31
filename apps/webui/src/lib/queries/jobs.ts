import { queryOptions } from "@tanstack/react-query";
import api from "../api";

export interface JobsListParams {
  scope: "active" | "completed";
  limit: number;
  offset: number;
  status?: string;
  packageName?: string;
  mockChroot?: string;
  /** When true, soft-deleted jobs are included (history scope only). */
  includeDeleted?: boolean;
}

export const jobsQueries = {
  list: (params: JobsListParams) =>
    queryOptions({
      queryKey: ["jobs", "list", params] as const,
      queryFn: () =>
        params.scope === "active"
          ? api.listActiveJobs({
              limit: params.limit,
              offset: params.offset,
              packageName: params.packageName,
              mockChroot: params.mockChroot,
            })
          : api.listCompletedJobs({
              limit: params.limit,
              offset: params.offset,
              status: params.status,
              packageName: params.packageName,
              mockChroot: params.mockChroot,
              includeDeleted: params.includeDeleted,
            }),
      placeholderData: (previous) => previous,
    }),
  detail: (id: string) =>
    queryOptions({
      queryKey: ["jobs", "detail", id] as const,
      queryFn: () => api.getJob(id),
    }),
  artifacts: (id: string, params: { limit: number; offset: number }) =>
    queryOptions({
      queryKey: ["jobs", "artifacts", id, params] as const,
      queryFn: () => api.listJobArtifacts(id, params.limit, params.offset),
      placeholderData: (previous) => previous,
    }),
  usageList: () =>
    queryOptions({
      queryKey: ["jobs", "usage-list"] as const,
      queryFn: () => api.listJobUsage(),
    }),
  usage: (id: string) =>
    queryOptions({
      queryKey: ["jobs", "usage", id] as const,
      queryFn: () => api.getJobUsage(id),
    }),
  logManifest: (id: string) =>
    queryOptions({
      queryKey: ["jobs", "log-manifest", id] as const,
      queryFn: () => api.getJobLogManifest(id),
    }),
};
