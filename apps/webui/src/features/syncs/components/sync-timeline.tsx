import type { SyncOperationEvent } from "../../../lib/types";
import { formatDateTime } from "../../../lib/datetime";

export default function SyncTimeline({ events }: { events: SyncOperationEvent[] }) {
  if (events.length === 0) {
    return <p className="font-mono text-sm text-soft">No timeline events recorded.</p>;
  }
  return (
    <ol className="space-y-0">
      {events.map((event, index) => (
        <li key={event.id} className="relative flex gap-4 pb-5 last:pb-0">
          {index < events.length - 1 ? (
            <span className="absolute bottom-0 left-[5px] top-3 w-px bg-edge" />
          ) : null}
          <span
            className={`relative mt-1.5 h-[11px] w-[11px] shrink-0 border ${
              event.level === "error"
                ? "border-error bg-error"
                : event.level === "warning"
                  ? "border-accent-orange bg-accent-orange"
                  : "border-accent-cyan bg-accent-cyan"
            }`}
          />
          <div className="min-w-0 flex-1">
            <div className="flex flex-wrap items-baseline justify-between gap-2">
              <span className="font-mono text-xs font-bold uppercase tracking-[0.12em] text-white">
                {event.stage.replaceAll("_", " ")}
              </span>
              <time className="font-mono text-xs text-soft">
                {formatDateTime(event.created_at)}
              </time>
            </div>
            <p className="mt-1 text-sm text-muted">{event.message}</p>
          </div>
        </li>
      ))}
    </ol>
  );
}
