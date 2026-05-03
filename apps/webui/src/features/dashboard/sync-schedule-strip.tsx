import { useEffect, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { faClock, faTriangleExclamation } from "@fortawesome/free-solid-svg-icons";
import { syncQueries } from "../../lib/queries";
import { usePageVisible } from "../../components/common/page-visibility-provider";
import EmptyState from "../../components/ui/empty-state";
import FaIcon from "../../components/ui/fa-icon";
import LoadingBlock from "../../components/ui/loading-block";

const SCHEDULE_LIMIT = 10;
const POLL_INTERVAL_MS = 30_000;

/**
 * Dashboard "Up next" widget — leans on /api/v1/sync/schedule. Each
 * row shows a package + chroot, a countdown that ticks against the
 * server-supplied `seconds_until`, and a backoff badge when applicable.
 *
 * The clock ticks locally every second using the server's `computed_at`
 * as the anchor — avoids drift across the polling interval.
 */
export default function SyncScheduleStrip() {
  const visible = usePageVisible();
  const { data, isPending, error } = useQuery({
    ...syncQueries.schedule(SCHEDULE_LIMIT),
    refetchInterval: visible ? POLL_INTERVAL_MS : false,
  });
  const [now, setNow] = useState(() => Date.now());
  useEffect(() => {
    if (!visible) return;
    const id = window.setInterval(() => setNow(Date.now()), 1000);
    return () => window.clearInterval(id);
  }, [visible]);

  return (
    <section className="border-2 border-edge-strong bg-black shadow-card-sm">
      <div className="flex items-end justify-between gap-4 border-b-2 border-edge-strong app-section-band px-5 py-4">
        <div className="flex items-center gap-2">
          <FaIcon icon={faClock} className="text-soft" />
          <h2 className="text-xl font-bold text-white">Up next</h2>
        </div>
        <span className="font-mono text-xs uppercase tracking-[0.18em] text-soft">
          Sync schedule
        </span>
      </div>
      <div className="p-5">
        {isPending ? (
          <LoadingBlock label="Loading schedule…" lines={0} />
        ) : error || !data ? (
          <EmptyState>
            Couldn&apos;t load schedule.{" "}
            {error instanceof Error ? error.message : ""}
          </EmptyState>
        ) : data.items.length === 0 ? (
          <EmptyState
            title="No packages polling"
            description="Enable polling on at least one package to see the upcoming schedule."
            hint="enable a package to start polling"
          />
        ) : (
          <ul className="space-y-2">
            {data.items.map((item) => {
              const remainingSec = computeRemaining(item, data.computed_at, now);
              return (
                <ScheduleRow
                  key={`${item.package_name}:${item.mock_chroot}`}
                  packageName={item.package_name}
                  mockChroot={item.mock_chroot}
                  remainingSec={remainingSec}
                  blockedByBackoff={item.blocked_by_backoff}
                  consecutiveFailures={item.consecutive_failures}
                />
              );
            })}
          </ul>
        )}
      </div>
    </section>
  );
}

function ScheduleRow({
  packageName,
  mockChroot,
  remainingSec,
  blockedByBackoff,
  consecutiveFailures,
}: {
  packageName: string;
  mockChroot: string;
  remainingSec: number;
  blockedByBackoff: boolean;
  consecutiveFailures: number;
}) {
  const overdue = remainingSec <= 0;
  // Visual progress: bar fills as countdown elapses. Cap window at 1h
  // so even very-overdue packages don't fill 100% indefinitely.
  const windowSec = blockedByBackoff
    ? Math.max(remainingSec, 1)
    : Math.min(Math.max(remainingSec, 60), 3_600);
  const percentRemaining = overdue
    ? 0
    : Math.min(100, Math.max(0, (remainingSec / windowSec) * 100));
  const accent = blockedByBackoff
    ? "bg-accent-orange"
    : overdue
      ? "bg-accent-lime"
      : "bg-success";

  return (
    <li className="relative border-2 border-edge bg-surface-alt/40">
      <Link
        to="/packages/view"
        search={{ name: packageName }}
        className="group block px-4 py-2.5 transition-colors hover:bg-surface-alt"
      >
        <div className="flex flex-wrap items-center gap-x-3 gap-y-1">
          <span className="font-display font-bold text-white transition-colors group-hover:text-accent-lime">
            {packageName}
          </span>
          <span className="border-2 border-edge-strong bg-black px-2 py-0.5 font-mono text-[10px] font-bold uppercase tracking-[0.14em] text-soft">
            {mockChroot}
          </span>
          {blockedByBackoff ? (
            <span className="inline-flex items-center gap-1 border-2 border-accent-orange bg-black px-2 py-0.5 font-mono text-[10px] font-bold uppercase tracking-[0.14em] text-accent-orange">
              <FaIcon icon={faTriangleExclamation} className="text-[0.7em]" />
              Backoff · {consecutiveFailures} fail
              {consecutiveFailures === 1 ? "" : "s"}
            </span>
          ) : null}
          <span
            className={`ml-auto font-mono text-xs tabular-nums ${
              overdue && !blockedByBackoff
                ? "text-accent-lime"
                : blockedByBackoff
                  ? "text-accent-orange"
                  : "text-strong"
            }`}
          >
            {overdue
              ? blockedByBackoff
                ? "blocked"
                : "due now"
              : `in ${formatRemaining(remainingSec)}`}
          </span>
        </div>
        {/* Countdown bar — track sits at the bottom of the row */}
        <div className="relative mt-2 h-1 bg-edge">
          <div
            className={`absolute inset-y-0 left-0 transition-[width] duration-1000 ease-linear ${accent}`}
            style={{ width: `${percentRemaining}%` }}
          />
        </div>
      </Link>
    </li>
  );
}

/**
 * Recompute remaining seconds locally so the countdown ticks without
 * waiting for the next refetch. We anchor to the server's `computed_at`
 * to avoid clock drift between client and daemon.
 */
function computeRemaining(
  item: { seconds_until: number; next_eligible_at: string },
  computedAt: string,
  nowMs: number,
): number {
  const computedAtMs = Date.parse(computedAt);
  if (!Number.isFinite(computedAtMs)) {
    return item.seconds_until;
  }
  const elapsedSec = Math.max(0, Math.floor((nowMs - computedAtMs) / 1000));
  return item.seconds_until - elapsedSec;
}

function formatRemaining(seconds: number): string {
  if (seconds < 60) return `${seconds}s`;
  if (seconds < 3_600) {
    const m = Math.floor(seconds / 60);
    const s = seconds % 60;
    return s > 0 ? `${m}m ${s}s` : `${m}m`;
  }
  const h = Math.floor(seconds / 3_600);
  const m = Math.floor((seconds % 3_600) / 60);
  return m > 0 ? `${h}h ${m}m` : `${h}h`;
}
