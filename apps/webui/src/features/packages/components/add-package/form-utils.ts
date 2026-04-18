import type { BuildEnvVar } from "../../../../lib/types";

export function encodeBuildEnv(entries: BuildEnvVar[]): string {
  return entries.map((entry) => `${entry.key}=${entry.value}`).join("\n");
}

export function parseBuildEnv(input: string): BuildEnvVar[] {
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

export function parseOptionalCpuLimit(
  value: string,
  maxCpuCores: number | null,
): number | undefined {
  const trimmed = value.trim();
  if (!trimmed) {
    return undefined;
  }
  const parsed = Number(trimmed);
  if (!Number.isFinite(parsed) || parsed <= 0) {
    return undefined;
  }
  const millicores = Math.floor(parsed * 1000);
  if (!maxCpuCores || maxCpuCores <= 0) {
    return millicores;
  }
  return Math.min(millicores, Math.floor(maxCpuCores * 1000));
}

export function parseOptionalMegabytes(value: string): number | undefined {
  const trimmed = value.trim();
  if (!trimmed) {
    return undefined;
  }
  const parsed = Number(trimmed);
  if (!Number.isFinite(parsed) || parsed <= 0) {
    return undefined;
  }
  return Math.floor(parsed);
}
