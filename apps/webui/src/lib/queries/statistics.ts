import { queryOptions } from "@tanstack/react-query";
import api from "../api";
import type { TimeRange } from "../types";

export const statisticsQueries = {
  overview: () =>
    queryOptions({
      queryKey: ["statistics", "overview"] as const,
      queryFn: async () => {
        const [cache, sync, repo, packages, enabled, active] = await Promise.all([
          api.getCacheStats(),
          api.getSyncMetrics(),
          api.getRepoSummary(),
          api.listPackagesPage(1, 0),
          api.listPackagesPage(1, 0, { enabled: true }),
          api.listActiveJobs({ limit: 1, offset: 0 }),
        ]);
        return {
          cacheStats: cache,
          syncMetrics: sync,
          repoSummary: repo,
          packageCount: packages.page.total ?? packages.packages.length,
          enabledPackageCount: enabled.page.total ?? enabled.packages.length,
          activeJobCount: active.page.total ?? active.jobs.length,
        };
      },
    }),

  timeseries: (range: TimeRange) =>
    queryOptions({
      queryKey: ["statistics", "timeseries", range] as const,
      queryFn: async () => {
        const [sync, jobs] = await Promise.all([
          api.getSyncTimeseries(range),
          api.getJobsTimeseries(range),
        ]);
        return { sync, jobs };
      },
    }),
};
