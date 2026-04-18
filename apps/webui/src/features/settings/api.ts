import api from "../../lib/api";

export const settingsApi = {
  getConfig: (...args: Parameters<typeof api.getConfig>) => api.getConfig(...args),
  getConfigSchema: (...args: Parameters<typeof api.getConfigSchema>) =>
    api.getConfigSchema(...args),
  updateRuntimeSettings: (
    ...args: Parameters<typeof api.updateRuntimeSettings>
  ) => api.updateRuntimeSettings(...args),
};
