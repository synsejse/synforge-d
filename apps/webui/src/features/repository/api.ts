import api from "../../lib/api";

export const repositoryApi = {
  getConfig: (...args: Parameters<typeof api.getConfig>) => api.getConfig(...args),
  getRepoInventory: (...args: Parameters<typeof api.getRepoInventory>) =>
    api.getRepoInventory(...args),
  getRepoSigningStatus: (...args: Parameters<typeof api.getRepoSigningStatus>) =>
    api.getRepoSigningStatus(...args),
  getRepoSummary: (...args: Parameters<typeof api.getRepoSummary>) =>
    api.getRepoSummary(...args),
  getSession: (...args: Parameters<typeof api.getSession>) =>
    api.getSession(...args),
};
