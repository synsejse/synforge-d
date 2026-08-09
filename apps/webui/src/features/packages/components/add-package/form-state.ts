import type { CreatePackageRequest } from "../../../../lib/types";
import {
  encodeBuildEnv,
  parseBuildEnv,
  parseOptionalCpuLimit,
  parseOptionalMegabytes,
} from "./form-utils";

export interface AddPackageFormState {
  name: string;
  repoUrl: string;
  specPath: string;
  enabled: boolean;
  poll: boolean;
  publishSrpm: boolean;
  publishDebuginfo: boolean;
  networkAccess: boolean;
  mockChroots: string[];
  pollIntervalSeconds: string;
  buildTimeoutSeconds: string;
  packageHistoryCount: string;
  cpuLimitCores: string;
  cpuLimitEnabled: boolean;
  memoryLimitEnabled: boolean;
  memoryLimitMb: string;
  ccacheEnabled: boolean;
  ccacheMaxSizeMb: string;
  buildEnv: string;
}

export const INITIAL_ADD_PACKAGE_FORM: AddPackageFormState = {
  name: "",
  repoUrl: "",
  specPath: "",
  enabled: true,
  poll: true,
  publishSrpm: true,
  publishDebuginfo: true,
  networkAccess: false,
  mockChroots: ["fedora-44-x86_64"],
  pollIntervalSeconds: "900",
  buildTimeoutSeconds: "7200",
  packageHistoryCount: "3",
  cpuLimitCores: "",
  cpuLimitEnabled: false,
  memoryLimitEnabled: false,
  memoryLimitMb: "1024",
  ccacheEnabled: false,
  ccacheMaxSizeMb: "",
  buildEnv: encodeBuildEnv([]),
};

export function buildCreatePackageRequest(
  form: AddPackageFormState,
  maxCpuCores: number | null,
): CreatePackageRequest {
  return {
    name: form.name.trim(),
    source: {
      repo_url: form.repoUrl.trim(),
      spec_file: form.specPath.trim(),
      poll: form.poll,
    },
    enabled: form.enabled,
    publish_srpm: form.publishSrpm,
    publish_debuginfo: form.publishDebuginfo,
    network_access: form.networkAccess,
    mock_chroots: form.mockChroots,
    poll_interval_seconds: Number(form.pollIntervalSeconds),
    build_timeout_seconds: Number(form.buildTimeoutSeconds),
    package_history_count: Number(form.packageHistoryCount),
    cpu_limit_millicores: form.cpuLimitEnabled
      ? parseOptionalCpuLimit(form.cpuLimitCores, maxCpuCores)
      : undefined,
    memory_limit_mb: form.memoryLimitEnabled
      ? parseOptionalMegabytes(form.memoryLimitMb)
      : undefined,
    ccache_enabled: form.ccacheEnabled,
    ccache_max_size_mb: form.ccacheEnabled
      ? parseOptionalMegabytes(form.ccacheMaxSizeMb)
      : undefined,
    build_env: parseBuildEnv(form.buildEnv),
  };
}
