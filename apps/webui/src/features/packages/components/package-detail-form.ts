import type {
  BuildEnvVar,
  PackageResponse,
  SpecSource,
  UpdatePackageRequest,
} from "../../../lib/types";
import type { PackageEditFormState } from "./package-edit-form-state";

export function encodeBuildEnv(entries: BuildEnvVar[]) {
  return entries.map((entry) => `${entry.key}=${entry.value}`).join("\n");
}

function parseBuildEnv(input: string): BuildEnvVar[] {
  return input
    .split("\n")
    .map((line) => line.trim())
    .filter((line) => line.length > 0)
    .map((line) => {
      const separator = line.indexOf("=");
      if (separator === -1) {
        return { key: line, value: "" };
      }
      return {
        key: line.slice(0, separator).trim(),
        value: line.slice(separator + 1),
      };
    });
}

function parseUpdateCpuLimit(value: string, maxCpuCores: number | null): number {
  const trimmed = value.trim();
  if (!trimmed) return 0;
  const parsed = Number(trimmed);
  if (!Number.isFinite(parsed) || parsed <= 0) return 0;
  const millicores = Math.floor(parsed * 1000);
  if (!maxCpuCores || maxCpuCores <= 0) return millicores;
  return Math.min(millicores, Math.floor(maxCpuCores * 1000));
}

function parseUpdateMemoryLimit(
  value: string,
  maxMemoryMb: number | null,
): number {
  const trimmed = value.trim();
  if (!trimmed) return 0;
  const parsed = Number(trimmed);
  if (!Number.isFinite(parsed) || parsed <= 0) return 0;
  const memoryLimitMb = Math.floor(parsed);
  if (!maxMemoryMb || maxMemoryMb <= 0) return memoryLimitMb;
  return Math.min(memoryLimitMb, Math.floor(maxMemoryMb));
}

function parseOptionalMegabytes(value: string): number | undefined {
  const trimmed = value.trim();
  if (!trimmed) return undefined;
  const parsed = Number(trimmed);
  if (!Number.isFinite(parsed) || parsed <= 0) return undefined;
  return Math.floor(parsed);
}

function formatCpuLimitCores(value?: number | null): string {
  if (!value || value <= 0) return "";
  const cores = value / 1000;
  return Number.isInteger(cores)
    ? String(cores)
    : cores.toFixed(2).replace(/\.?0+$/, "");
}

export function buildFormFromPackage(
  packageRes: PackageResponse,
): PackageEditFormState {
  const definition = packageRes.package;
  return {
    repoUrl: definition.source.repo_url,
    specPath: definition.source.spec_file,
    poll: definition.source.poll ?? true,
    mockChroots: definition.mock_chroots ?? [],
    pollIntervalSeconds: String(definition.poll_interval_seconds ?? 900),
    buildTimeoutSeconds: String(definition.build_timeout_seconds ?? 7200),
    packageHistoryCount: String(definition.package_history_count ?? 3),
    cpuLimitCores: formatCpuLimitCores(definition.cpu_limit_millicores),
    cpuLimitEnabled: Number(definition.cpu_limit_millicores ?? 0) > 0,
    memoryLimitEnabled: Number(definition.memory_limit_mb ?? 0) > 0,
    memoryLimitMb: definition.memory_limit_mb
      ? String(definition.memory_limit_mb)
      : "",
    ccache_enabled: definition.ccache_enabled ?? false,
    ccacheMaxSizeMb: definition.ccache_max_size_mb
      ? String(definition.ccache_max_size_mb)
      : "",
    buildEnv: encodeBuildEnv(definition.build_env ?? []),
    enabled: definition.enabled ?? true,
    publish_srpm: definition.publish_srpm ?? true,
    publish_debuginfo: definition.publish_debuginfo ?? true,
    network_access: definition.network_access ?? false,
  };
}

export function buildUpdateRequest(
  form: PackageEditFormState,
  maxCpuCores: number | null,
  maxMemoryMb: number | null,
): UpdatePackageRequest {
  const source: SpecSource = {
    repo_url: form.repoUrl,
    spec_file: form.specPath,
    poll: form.poll,
  };
  return {
    source,
    enabled: form.enabled,
    publish_srpm: form.publish_srpm,
    publish_debuginfo: form.publish_debuginfo,
    network_access: form.network_access,
    mock_chroots: form.mockChroots,
    poll_interval_seconds: Number(form.pollIntervalSeconds),
    build_timeout_seconds: Number(form.buildTimeoutSeconds),
    package_history_count: Number(form.packageHistoryCount),
    cpu_limit_millicores: form.cpuLimitEnabled
      ? parseUpdateCpuLimit(form.cpuLimitCores, maxCpuCores)
      : 0,
    memory_limit_mb: form.memoryLimitEnabled
      ? parseUpdateMemoryLimit(form.memoryLimitMb, maxMemoryMb)
      : 0,
    ccache_enabled: form.ccache_enabled,
    ccache_max_size_mb: parseOptionalMegabytes(form.ccacheMaxSizeMb) ?? 0,
    build_env: parseBuildEnv(form.buildEnv),
  };
}

export const EMPTY_FORM: PackageEditFormState = {
  repoUrl: "",
  specPath: "",
  poll: true,
  mockChroots: [],
  pollIntervalSeconds: "900",
  buildTimeoutSeconds: "7200",
  packageHistoryCount: "3",
  cpuLimitCores: "",
  cpuLimitEnabled: false,
  memoryLimitEnabled: false,
  memoryLimitMb: "",
  ccache_enabled: false,
  ccacheMaxSizeMb: "",
  buildEnv: "",
  enabled: true,
  publish_srpm: true,
  publish_debuginfo: true,
  network_access: false,
};
