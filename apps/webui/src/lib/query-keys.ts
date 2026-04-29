/**
 * Centralized query key factories for TanStack Query.
 * Use these everywhere instead of inline string arrays so invalidations stay in sync.
 */
export const queryKeys = {
  dashboard: () => ["dashboard"] as const,

  packages: {
    list: (params: {
      limit: number;
      offset: number;
      search?: string;
      enabled?: boolean | "all";
    }) => ["packages", "list", params] as const,
    detail: (name: string) => ["packages", "detail", name] as const,
    builds: (name: string, params: { limit: number; offset: number }) =>
      ["packages", "builds", name, params] as const,
    repoFiles: (name: string, params: { limit: number; offset: number }) =>
      ["packages", "repo-files", name, params] as const,
    refreshAllProgress: () => ["packages", "refresh-all-progress"] as const,
    mockChroots: () => ["packages", "mock-chroots"] as const,
  },

  sync: {
    operations: (
      packageName: string,
      params: { limit: number; offset: number; status?: string },
    ) => ["sync", "operations", packageName, params] as const,
  },

  jobs: {
    active: (params: {
      limit: number;
      offset: number;
      packageName?: string;
      mockChroot?: string;
    }) => ["jobs", "active", params] as const,
    completed: (params: {
      limit: number;
      offset: number;
      status?: string;
      packageName?: string;
      mockChroot?: string;
    }) => ["jobs", "completed", params] as const,
    detail: (jobId: string) => ["jobs", "detail", jobId] as const,
    artifacts: (jobId: string) => ["jobs", "artifacts", jobId] as const,
    usageList: () => ["jobs", "usage-list"] as const,
    usage: (jobId: string) => ["jobs", "usage", jobId] as const,
    logManifest: (jobId: string) => ["jobs", "log-manifest", jobId] as const,
  },

  repository: {
    summary: () => ["repository", "summary"] as const,
    inventory: (params: {
      limit: number;
      offset: number;
      packageName?: string;
      mockChroot?: string;
      kind?: string;
    }) => ["repository", "inventory", params] as const,
    setup: () => ["repository", "setup"] as const,
  },

  signing: {
    status: () => ["signing", "status"] as const,
    reconcileProgress: () => ["signing", "reconcile-progress"] as const,
  },

  config: {
    effective: () => ["config", "effective"] as const,
    schema: () => ["config", "schema"] as const,
  },

  users: {
    list: () => ["users", "list"] as const,
  },

  statistics: {
    overview: () => ["statistics", "overview"] as const,
    cache: () => ["statistics", "cache"] as const,
    sync: () => ["statistics", "sync"] as const,
  },
};
