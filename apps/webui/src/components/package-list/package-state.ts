import type { PackageResponse, PackageTargetState } from "../../lib/types";

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

export function compactRevision(revision: string | null) {
  if (!revision) {
    return "No successful revision";
  }
  if (revision.length <= 44) {
    return revision;
  }
  return `${revision.slice(0, 20)}...${revision.slice(-16)}`;
}

export function formatMockChroots(chroots: string[]) {
  return chroots.join(", ");
}
