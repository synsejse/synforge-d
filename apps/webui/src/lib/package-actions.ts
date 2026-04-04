import type {
  PackageActionResponse,
  PackageActionTargetResult,
} from "./types";

export function summarizePackageAction(response: PackageActionResponse): string {
  const queued = response.results.filter(
    (result) => result.disposition === "queued",
  );
  const skipped = response.results.filter(
    (result) => result.disposition === "skipped",
  );
  const blocked = response.results.filter(
    (result) => result.disposition === "blocked",
  );

  const parts: string[] = [];
  if (queued.length > 0) {
    parts.push(`queued ${queued.length} target${queued.length === 1 ? "" : "s"}`);
  }
  if (skipped.length > 0) {
    parts.push(
      `skipped ${skipped.length} unchanged target${skipped.length === 1 ? "" : "s"}`,
    );
  }
  if (blocked.length > 0) {
    parts.push(
      `blocked ${blocked.length} busy target${blocked.length === 1 ? "" : "s"}`,
    );
  }

  if (parts.length === 0) {
    return "No targets were affected.";
  }

  const summary = `${capitalizeTrigger(response.trigger)}: ${parts.join(", ")}.`;
  const detailLines: string[] = [];
  if (queued.length > 0) {
    detailLines.push(`Queued: ${queued.map((result) => result.mock_chroot).join(", ")}`);
  }
  if (skipped.length > 0) {
    detailLines.push(`Skipped: ${skipped.map((result) => result.mock_chroot).join(", ")}`);
  }
  if (blocked.length > 0) {
    detailLines.push(`Blocked: ${blocked.map((result) => result.mock_chroot).join(", ")}`);
  }
  if (detailLines.length === 0) {
    return summary;
  }
  return `${summary}\n${detailLines.join("\n")}`;
}

function capitalizeTrigger(trigger: string): string {
  return trigger
    .split("_")
    .map((segment) => segment.charAt(0).toUpperCase() + segment.slice(1))
    .join(" ");
}

export function summarizePackageTargetAction(
  result: PackageActionTargetResult,
  actionLabel: string,
): string {
  switch (result.disposition) {
    case "queued":
      return `${actionLabel}: queued ${result.mock_chroot}.`;
    case "skipped":
      return `${actionLabel}: skipped ${result.mock_chroot} (${humanizeReason(result.reason)}).`;
    case "blocked":
      return `${actionLabel}: blocked ${result.mock_chroot} (${humanizeReason(result.reason)}).`;
    default:
      return `${actionLabel}: ${result.mock_chroot}.`;
  }
}

function humanizeReason(reason: string | null): string {
  switch (reason) {
    case "no_source_change":
      return "no source change";
    case "pending_or_running":
      return "pending or running";
    default:
      return reason ?? "no reason provided";
  }
}
