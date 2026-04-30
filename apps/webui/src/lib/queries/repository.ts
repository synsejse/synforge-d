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
        const [config, signing] = await Promise.all([
          api.getConfig(),
          api.getRepoSigningStatus(),
        ]);
        return {
          publicBaseUrl: normalizeBaseUrl(config.config.public_base_url),
          signingEnabled: signing.status.enabled,
        };
      },
    }),
};
