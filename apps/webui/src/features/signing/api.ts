import api from "../../lib/api";

export const signingApi = {
  exportRepoSigningKey: (...args: Parameters<typeof api.exportRepoSigningKey>) =>
    api.exportRepoSigningKey(...args),
  exportRepoSigningPublicKey: (
    ...args: Parameters<typeof api.exportRepoSigningPublicKey>
  ) => api.exportRepoSigningPublicKey(...args),
  generateRepoSigningKey: (
    ...args: Parameters<typeof api.generateRepoSigningKey>
  ) => api.generateRepoSigningKey(...args),
  getRepoSigningReconcileProgress: (
    ...args: Parameters<typeof api.getRepoSigningReconcileProgress>
  ) => api.getRepoSigningReconcileProgress(...args),
  getRepoSigningStatus: (...args: Parameters<typeof api.getRepoSigningStatus>) =>
    api.getRepoSigningStatus(...args),
  importRepoSigningKey: (...args: Parameters<typeof api.importRepoSigningKey>) =>
    api.importRepoSigningKey(...args),
  removeRepoSigningKey: (...args: Parameters<typeof api.removeRepoSigningKey>) =>
    api.removeRepoSigningKey(...args),
  testRepoSigning: (...args: Parameters<typeof api.testRepoSigning>) =>
    api.testRepoSigning(...args),
  updateRepoSigningConfig: (
    ...args: Parameters<typeof api.updateRepoSigningConfig>
  ) => api.updateRepoSigningConfig(...args),
};
