import { queryOptions } from "@tanstack/react-query";
import api from "../api";

function normalizeBaseUrl(baseUrl: string): string {
  return baseUrl.replace(/\/+$/, "");
}

export interface RepositoryInventoryParams {
  limit: number;
  offset: number;
  packageName?: string;
  mockChroot?: string;
  kind?: "rpm" | "srpm" | "log" | "other" | "all";
}

export const repositoryQueries = {
  summary: () =>
    queryOptions({
      queryKey: ["repository", "summary"] as const,
      queryFn: () => api.getRepoSummary(),
    }),
  inventory: (params: RepositoryInventoryParams) =>
    queryOptions({
      queryKey: ["repository", "inventory", params] as const,
      queryFn: () =>
        api.getRepoInventory(params.limit, params.offset, {
          packageName: params.packageName,
          mockChroot: params.mockChroot,
          kind: params.kind,
        }),
      placeholderData: (previous) => previous,
    }),
  setup: () =>
    queryOptions({
      queryKey: ["repository", "setup"] as const,
      queryFn: async () => {
        const info = await api.getRepoSetupInfo();
        return {
          publicBaseUrl: normalizeBaseUrl(info.public_base_url),
          signingEnabled: info.signing_enabled,
          publicKeyName: info.public_key_name,
        };
      },
    }),
};
