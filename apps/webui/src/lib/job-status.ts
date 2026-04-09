import type { BuildStatus } from "./types";

export type HistoryBuildStatus = Exclude<BuildStatus, "pending" | "running">;

export const BUILD_STATUS_LABELS: Record<BuildStatus, string> = {
  pending: "pending",
  running: "running",
  succeeded: "succeeded",
  failed: "failed",
  timed_out: "timed_out",
};

export const HISTORY_BUILD_STATUS_LABELS: Record<HistoryBuildStatus, string> = {
  succeeded: "Succeeded",
  failed: "Failed",
  timed_out: "Timed Out",
};

export function isHistoryBuildStatus(value: string): value is HistoryBuildStatus {
  return value in HISTORY_BUILD_STATUS_LABELS;
}
