import api from "../../lib/api";

export const packagesApi = {
  browseRepository: (...args: Parameters<typeof api.browseRepository>) =>
    api.browseRepository(...args),
  createPackage: (...args: Parameters<typeof api.createPackage>) =>
    api.createPackage(...args),
  deleteJob: (...args: Parameters<typeof api.deleteJob>) => api.deleteJob(...args),
  deletePackage: (...args: Parameters<typeof api.deletePackage>) =>
    api.deletePackage(...args),
  getPackage: (...args: Parameters<typeof api.getPackage>) => api.getPackage(...args),
  getPackageBuilds: (...args: Parameters<typeof api.getPackageBuilds>) =>
    api.getPackageBuilds(...args),
  getRefreshAllPackagesProgress: (
    ...args: Parameters<typeof api.getRefreshAllPackagesProgress>
  ) => api.getRefreshAllPackagesProgress(...args),
  getRepoInventory: (...args: Parameters<typeof api.getRepoInventory>) =>
    api.getRepoInventory(...args),
  getServerHardware: (...args: Parameters<typeof api.getServerHardware>) =>
    api.getServerHardware(...args),
  listMockChroots: (...args: Parameters<typeof api.listMockChroots>) =>
    api.listMockChroots(...args),
  listPackagesPage: (...args: Parameters<typeof api.listPackagesPage>) =>
    api.listPackagesPage(...args),
  listPackageSyncOperations: (
    ...args: Parameters<typeof api.listPackageSyncOperations>
  ) => api.listPackageSyncOperations(...args),
  rebuildPackage: (...args: Parameters<typeof api.rebuildPackage>) =>
    api.rebuildPackage(...args),
  rebuildPackageTarget: (...args: Parameters<typeof api.rebuildPackageTarget>) =>
    api.rebuildPackageTarget(...args),
  refreshAllPackages: (...args: Parameters<typeof api.refreshAllPackages>) =>
    api.refreshAllPackages(...args),
  refreshPackage: (...args: Parameters<typeof api.refreshPackage>) =>
    api.refreshPackage(...args),
  refreshPackageTarget: (...args: Parameters<typeof api.refreshPackageTarget>) =>
    api.refreshPackageTarget(...args),
  updatePackage: (...args: Parameters<typeof api.updatePackage>) =>
    api.updatePackage(...args),
};
