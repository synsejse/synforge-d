import { useEffect, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { faClock } from "@fortawesome/free-solid-svg-icons";
import { syncQueries } from "../../lib/queries";
import type { SyncScheduleEntry } from "../../lib/types";
import { usePageVisible } from "../../components/common/page-visibility-context";
import EmptyState from "../../components/ui/empty-state";
import FaIcon from "../../components/ui/fa-icon";
import LoadingBlock from "../../components/ui/loading-block";

const SCHEDULE_LIMIT = 10;
const POLL_INTERVAL_MS = 30_000;

/**
 * Dashboard "Up next" widget — leans on /api/v1/sync/schedule. Renders
 * the upcoming package source polls as a horizontal timeline with cards
 * alternating above and below a single track, ordered left-to-right by
 * time-until. The countdown ticks locally every second using the
 * server's `computed_at` as the anchor — avoids drift across the polling
 * interval.
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
    <section className="border border-edge bg-black">
      <div className="flex items-center justify-between gap-4 border-b border-edge px-[18px] py-[15px]">
        <div className="flex items-center gap-2.5">
          <FaIcon icon={faClock} className="text-[11px] text-soft" />
          <h2 className="font-mono text-[13px] font-bold uppercase tracking-[0.06em] text-white">
            Up next
          </h2>
        </div>
        <span className="font-mono text-[10px] font-bold uppercase tracking-[0.16em] text-soft">
          Sync schedule
        </span>
      </div>
      <div className="p-[18px]">
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
          <ScheduleTimeline
            items={data.items}
            computedAt={data.computed_at}
            now={now}
          />
        )}
      </div>
    </section>
  );
}

/**
 * Linear schedule timeline — a single left-to-right row of fixed-width
 * cards, each trailed by a marker + connector segment on a shared baseline
 * (matching the design comp). Scrolls horizontally when it overflows.
 */
function ScheduleTimeline({
  items,
  computedAt,
  now,
}: {
  items: SyncScheduleEntry[];
  computedAt: string;
  now: number;
}) {
  return (
    <div className="overflow-x-auto pb-3">
      <ul className="flex min-w-max items-stretch gap-0">
        {items.map((item) => (
          <ScheduleCard
            key={item.package_name}
            item={item}
            remainingSec={computeRemaining(item, computedAt, now)}
          />
        ))}
      </ul>
    </div>
  );
}

function ScheduleCard({
  item,
  remainingSec,
}: {
  item: SyncScheduleEntry;
  remainingSec: number;
}) {
  const overdue = remainingSec <= 0;
  const eta = overdue ? "due now" : `in ${formatRemaining(remainingSec)}`;

  return (
    <li className="flex w-[148px] shrink-0 flex-col items-start">
      <Link
        to="/packages/view"
        search={{ name: item.package_name }}
        className="group block w-[140px] border border-edge bg-surface-alt px-[11px] py-[9px] transition-colors hover:border-edge-strong"
        title={`${item.package_name} source poll`}
      >
        <div className="truncate font-mono text-[11px] font-bold leading-none text-white group-hover:text-accent-lime">
          {item.package_name}
        </div>
        <div className="mt-1.5 truncate font-mono text-[9px] leading-none text-[#6b6b73]">
          Package source
        </div>
        <div
          className="mt-2 flex items-center gap-1 font-mono text-[10px] font-semibold leading-none tabular-nums text-accent-lime"
        >
          {eta}
        </div>
      </Link>
      <div className="mt-2.5 flex w-[140px] items-center">
        <span aria-hidden="true" className="h-2 w-2 shrink-0 bg-accent-lime" />
        <span aria-hidden="true" className="h-px flex-1 bg-edge" />
      </div>
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
