import type { SyncEnqueueResponse } from "./types";

export function summarizeSyncEnqueue(response: SyncEnqueueResponse): string {
  const operation = response.operation;
  const shortId = operation.id.slice(0, 8);
  if (!response.created) {
    return `Already ${operation.status} as sync ${shortId}.`;
  }
  const scope = operation.target_mock_chroot
    ? `${operation.package_name} / ${operation.target_mock_chroot}`
    : operation.package_name;
  return `${scope} queued as sync ${shortId}.`;
}
