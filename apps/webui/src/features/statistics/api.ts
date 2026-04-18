import api from "../../lib/api";

export const statisticsApi = {
  getCacheStats: (...args: Parameters<typeof api.getCacheStats>) =>
    api.getCacheStats(...args),
  getRepoSummary: (...args: Parameters<typeof api.getRepoSummary>) =>
    api.getRepoSummary(...args),
  getSyncMetrics: (...args: Parameters<typeof api.getSyncMetrics>) =>
    api.getSyncMetrics(...args),
  listActiveJobs: (...args: Parameters<typeof api.listActiveJobs>) =>
    api.listActiveJobs(...args),
  listPackagesPage: (...args: Parameters<typeof api.listPackagesPage>) =>
    api.listPackagesPage(...args),
};
