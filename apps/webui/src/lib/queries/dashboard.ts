import { queryOptions } from "@tanstack/react-query";
import api from "../api";

export const dashboardQueries = {
  overview: () =>
    queryOptions({
      queryKey: ["dashboard", "overview"] as const,
      queryFn: async () => {
        const [packages, enabledPackages, recentJobs, activeJobs, repository] =
          await Promise.all([
            api.listPackagesPage(1, 0),
            api.listPackagesPage(1, 0, { enabled: true }),
            api.listCompletedJobs({ limit: 6, offset: 0 }),
            api.listActiveJobs({ limit: 6, offset: 0 }),
            api.getRepoSummary(),
          ]);
        return {
          packageCount: packages.page.total ?? packages.packages.length,
          enabledPackageCount:
            enabledPackages.page.total ?? enabledPackages.packages.length,
          activeJobCount: activeJobs.page.total ?? activeJobs.jobs.length,
          jobs: recentJobs.jobs,
          liveJobs: activeJobs.jobs,
          repoSummary: repository,
        };
      },
    }),
};
