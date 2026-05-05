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

interface ScheduleItem {
  package_name: string;
  mock_chroot: string;
  seconds_until: number;
  next_eligible_at: string;
  blocked_by_backoff: boolean;
  consecutive_failures: number;
}

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
      <div className="flex items-baseline justify-between gap-4 border-b-2 border-edge-strong app-section-band px-5 py-4">
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

const CARD_HEIGHT = 64;
const CONNECTOR_HEIGHT = 14;
const TICK_SIZE = 10;
const HALF_SLOT = CARD_HEIGHT + CONNECTOR_HEIGHT;
const SLOT_HEIGHT = HALF_SLOT * 2 + TICK_SIZE;

function ScheduleTimeline({
  items,
  computedAt,
  now,
}: {
  items: ScheduleItem[];
  computedAt: string;
  now: number;
}) {
  return (
    <div className="overflow-x-auto pb-3 pt-1">
      <div className="relative min-w-max">
        <div
          aria-hidden="true"
          className="pointer-events-none absolute left-0 right-0 h-px bg-edge-strong"
          style={{ top: SLOT_HEIGHT / 2 }}
        />
        <ul
          className="relative flex items-stretch gap-3 px-1"
          style={{ height: SLOT_HEIGHT }}
        >
          {items.map((item, i) => (
            <ScheduleSlot
              key={`${item.package_name}:${item.mock_chroot}`}
              item={item}
              above={i % 2 === 0}
              remainingSec={computeRemaining(item, computedAt, now)}
            />
          ))}
        </ul>
      </div>
    </div>
  );
}

function ScheduleSlot({
  item,
  above,
  remainingSec,
}: {
  item: ScheduleItem;
  above: boolean;
  remainingSec: number;
}) {
  const overdue = remainingSec <= 0;
  const blocked = item.blocked_by_backoff;
  const card = <ScheduleCard item={item} remainingSec={remainingSec} />;
  const connector = (
    <div
      aria-hidden="true"
      className="w-px bg-edge-strong"
      style={{ height: CONNECTOR_HEIGHT }}
    />
  );
  const halfSpacer = <div style={{ height: HALF_SLOT }} />;
  const tick = <Tick blocked={blocked} overdue={overdue} />;

  return (
    <li className="flex w-32 flex-shrink-0 flex-col items-center sm:w-36">
      {above ? (
        <>
          {card}
          {connector}
          {tick}
          {halfSpacer}
        </>
      ) : (
        <>
          {halfSpacer}
          {tick}
          {connector}
          {card}
        </>
      )}
    </li>
  );
}

function Tick({ blocked, overdue }: { blocked: boolean; overdue: boolean }) {
  const fillClass = blocked
    ? "bg-accent-orange"
    : overdue
      ? "bg-accent-lime"
      : "bg-success";
  // Square tick with a 2px black halo so the connector line visibly
  // terminates AT the tick rather than embedding into it.
  return (
    <div
      aria-hidden="true"
      className={`relative z-10 ring-2 ring-black ${fillClass}`}
      style={{ width: TICK_SIZE, height: TICK_SIZE }}
    />
  );
}

function ScheduleCard({
  item,
  remainingSec,
}: {
  item: ScheduleItem;
  remainingSec: number;
}) {
  const overdue = remainingSec <= 0;
  const blocked = item.blocked_by_backoff;
  const borderClass = blocked
    ? "border-accent-orange"
    : overdue
      ? "border-accent-lime"
      : "border-edge-strong";
  const timeClass = blocked
    ? "text-accent-orange"
    : overdue
      ? "text-accent-lime"
      : "text-strong";

  return (
    <Link
      to="/packages/view"
      search={{ name: item.package_name }}
      className={`group flex w-full flex-col justify-between border-2 ${borderClass} bg-surface-alt/60 px-2 py-1.5 transition-colors hover:bg-surface-alt`}
      style={{ height: CARD_HEIGHT }}
      title={`${item.package_name} · ${item.mock_chroot}`}
    >
      <div className="truncate font-display text-xs font-bold uppercase text-white group-hover:text-accent-lime">
        {item.package_name}
      </div>
      <div className="truncate font-mono text-[9px] uppercase tracking-[0.16em] text-soft">
        {item.mock_chroot}
      </div>
      <div
        className={`flex items-center gap-1 font-mono text-xs font-bold tabular-nums ${timeClass}`}
      >
        {blocked ? (
          <FaIcon icon={faTriangleExclamation} className="text-[0.7em]" />
        ) : null}
        {overdue
          ? blocked
            ? "blocked"
            : "due now"
          : `in ${formatRemaining(remainingSec)}`}
      </div>
    </Link>
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
