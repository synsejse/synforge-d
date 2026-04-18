import api from "../../lib/api";

export const jobsApi = {
  deleteJob: (...args: Parameters<typeof api.deleteJob>) => api.deleteJob(...args),
  downloadJobArtifact: (...args: Parameters<typeof api.downloadJobArtifact>) =>
    api.downloadJobArtifact(...args),
  downloadJobLog: (...args: Parameters<typeof api.downloadJobLog>) =>
    api.downloadJobLog(...args),
  getJob: (...args: Parameters<typeof api.getJob>) => api.getJob(...args),
  getJobArtifactMeta: (...args: Parameters<typeof api.getJobArtifactMeta>) =>
    api.getJobArtifactMeta(...args),
  getJobLogChunk: (...args: Parameters<typeof api.getJobLogChunk>) =>
    api.getJobLogChunk(...args),
  getJobLogManifest: (...args: Parameters<typeof api.getJobLogManifest>) =>
    api.getJobLogManifest(...args),
  getJobLogMeta: (...args: Parameters<typeof api.getJobLogMeta>) =>
    api.getJobLogMeta(...args),
  getJobUsage: (...args: Parameters<typeof api.getJobUsage>) =>
    api.getJobUsage(...args),
  getServerHardware: (...args: Parameters<typeof api.getServerHardware>) =>
    api.getServerHardware(...args),
  killJob: (...args: Parameters<typeof api.killJob>) => api.killJob(...args),
  listActiveJobs: (...args: Parameters<typeof api.listActiveJobs>) =>
    api.listActiveJobs(...args),
  listCompletedJobs: (...args: Parameters<typeof api.listCompletedJobs>) =>
    api.listCompletedJobs(...args),
  listJobArtifacts: (...args: Parameters<typeof api.listJobArtifacts>) =>
    api.listJobArtifacts(...args),
  listJobUsage: (...args: Parameters<typeof api.listJobUsage>) =>
    api.listJobUsage(...args),
  pruneFailedJobs: (...args: Parameters<typeof api.pruneFailedJobs>) =>
    api.pruneFailedJobs(...args),
  retryJob: (...args: Parameters<typeof api.retryJob>) => api.retryJob(...args),
};
