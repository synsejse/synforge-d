import { useState } from "react";
import { useToast } from "../../components/common/toast-provider";

interface RunBulkActionOptions<T> {
  /** The items to act on; the action runs once per item, all in flight. */
  items: T[];
  /** Per-item async action. Rejections count as failures, not throws. */
  action: (item: T) => Promise<unknown>;
  /** Toast title when every item succeeded. */
  successTitle: string;
  /** Toast title when at least one item failed. */
  partialTitle: string;
  /** Builds the success toast body, e.g. `${count} packages removed.`. */
  successMessage: (count: number) => string;
}

interface UseBulkActionResult {
  /** True while a bulk run is in flight — wire to disable trigger buttons. */
  running: boolean;
  /**
   * Run `action` across `items` with `Promise.allSettled`, then surface a
   * success or partial-failure toast and invalidate package queries. Always
   * settles (never throws) so callers can clear selection afterwards.
   */
  run: <T>(options: RunBulkActionOptions<T>) => Promise<void>;
}

/**
 * Shared bulk-action runner for the package list. Centralizes the
 * `Promise.allSettled` → count successes/failures → toast → invalidate
 * sequence that bulk refresh / rebuild / delete all share.
 */
export function useBulkAction(invalidate: () => unknown): UseBulkActionResult {
  const toast = useToast();
  const [running, setRunning] = useState(false);

  async function run<T>({
    items,
    action,
    successTitle,
    partialTitle,
    successMessage,
  }: RunBulkActionOptions<T>): Promise<void> {
    setRunning(true);
    try {
      const results = await Promise.allSettled(items.map((item) => action(item)));
      const ok = results.filter((r) => r.status === "fulfilled").length;
      const failed = results.length - ok;
      if (failed === 0) {
        toast.success(successTitle, successMessage(ok));
      } else {
        toast.error(partialTitle, `${ok} succeeded · ${failed} failed.`);
      }
      await invalidate();
    } finally {
      setRunning(false);
    }
  }

  return { running, run };
}
