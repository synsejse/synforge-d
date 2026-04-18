import type { PackageResponse, PackageTargetState } from "../../../lib/types";
export { formatMockChroots } from "../../../lib/utils";

export function summarizePackageStatus(entry: PackageResponse) {
  if (!entry.package.enabled) {
    return "disabled";
  }
  if (entry.state.targets.some((target) => target.active_status === "running")) {
    return "running";
  }
  if (entry.state.targets.some((target) => target.active_status === "pending")) {
    return "pending";
  }
  return "enabled";
}

export function targetStatus(target: PackageTargetState) {
  if (target.active_status) {
    return target.active_status;
  }
  return target.last_successful_build_id ? "succeeded" : "disabled";
}

export function compactRevision(revision: string | null | undefined) {
  if (!revision) {
    return "No successful revision";
  }
  if (revision.length <= 44) {
    return revision;
  }
  return `${revision.slice(0, 20)}...${revision.slice(-16)}`;
}
