export interface PackageEditFormState {
  repoUrl: string;
  specPath: string;
  poll: boolean;
  mockChroots: string[];
  pollIntervalSeconds: string;
  buildTimeoutSeconds: string;
  packageHistoryCount: string;
  cpuLimitCores: string;
  cpuLimitEnabled: boolean;
  memoryLimitEnabled: boolean;
  memoryLimitMb: string;
  ccache_enabled: boolean;
  ccacheMaxSizeMb: string;
  buildEnv: string;
  enabled: boolean;
  publish_srpm: boolean;
  publish_debuginfo: boolean;
  network_access: boolean;
}
