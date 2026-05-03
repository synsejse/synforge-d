import { queryOptions } from "@tanstack/react-query";
import api from "../api";

export interface PackageListParams {
  limit: number;
  offset: number;
  search?: string;
  enabled?: boolean | "all";
}

export const packagesQueries = {
  list: (params: PackageListParams) =>
    queryOptions({
      queryKey: ["packages", "list", params] as const,
      queryFn: () =>
        api.listPackagesPage(params.limit, params.offset, {
          search: params.search,
          enabled: params.enabled,
        }),
      placeholderData: (previous) => previous,
    }),
  detail: (name: string) =>
    queryOptions({
      queryKey: ["packages", "detail", name] as const,
      queryFn: () => api.getPackage(name),
    }),
  builds: (
    name: string,
    params: { limit: number; offset: number; includeDeleted?: boolean },
  ) =>
    queryOptions({
      queryKey: ["packages", "builds", name, params] as const,
      queryFn: () =>
        api.getPackageBuilds(name, params.limit, params.offset, {
          includeDeleted: params.includeDeleted,
        }),
      placeholderData: (previous) => previous,
    }),
  repoFiles: (name: string, params: { limit: number; offset: number }) =>
    queryOptions({
      queryKey: ["packages", "repo-files", name, params] as const,
      queryFn: () =>
        api.getRepoInventory(params.limit, params.offset, {
          packageName: name,
        }),
      placeholderData: (previous) => previous,
    }),
  refreshAllProgress: () =>
    queryOptions({
      queryKey: ["packages", "refresh-all-progress"] as const,
      queryFn: () => api.getRefreshAllPackagesProgress(),
    }),
  mockChroots: () =>
    queryOptions({
      queryKey: ["packages", "mock-chroots"] as const,
      queryFn: () => api.listMockChroots(),
    }),
};
