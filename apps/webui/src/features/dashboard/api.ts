import api from "../../lib/api";

export const dashboardApi = {
  getRepoSummary: (...args: Parameters<typeof api.getRepoSummary>) =>
    api.getRepoSummary(...args),
  listActiveJobs: (...args: Parameters<typeof api.listActiveJobs>) =>
    api.listActiveJobs(...args),
  listCompletedJobs: (...args: Parameters<typeof api.listCompletedJobs>) =>
    api.listCompletedJobs(...args),
  listPackagesPage: (...args: Parameters<typeof api.listPackagesPage>) =>
    api.listPackagesPage(...args),
};
