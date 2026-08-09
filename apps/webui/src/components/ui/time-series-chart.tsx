import { useMemo } from "react";
import type { TimeSeriesPoint, TimeSeriesResponse } from "../../lib/types";
import LoadingBlock from "./loading-block";

interface Props {
  data: TimeSeriesResponse | undefined;
  isLoading?: boolean;
  /** Hex / var() colour for the "succeeded" stack. */
  succeededColor?: string;
  /** Hex / var() colour for the "failed" stack. */
  failedColor?: string;
  /** Message rendered when there are no events at all in the window. */
  emptyLabel?: string;
}

interface ChartRow {
  label: string;
  succeeded: number;
  failed: number;
}

const BAR_HEIGHT = 150;
// Cap how many bars we draw so a fine-grained window (e.g. 7d bucketed
// hourly → ~168 points) stays legible instead of collapsing into hairlines.
// When there are more buckets than this we merge adjacent ones, summing
// their counts. Labels are thinned separately so they never overlap.
const MAX_BARS = 48;
const MAX_LABELS = 8;

/**
 * Stacked bar chart for succeeded/failed event counts over time — one bar
 * per bucket, failed stacked above succeeded, on a bottom/left axis frame.
 * Pure CSS bars (no chart lib) to match the design comp's terminal look.
 */
export default function TimeSeriesChart({
  data,
  isLoading = false,
  succeededColor = "#1FA463",
  failedColor = "#E0383B",
  emptyLabel = "No activity in this window.",
}: Props) {
  const range = data?.range ?? "24h";

  const rows = useMemo<ChartRow[]>(() => {
    if (!data) return [];
    return data.points.map((point: TimeSeriesPoint) => ({
      label: formatBucketLabel(Date.parse(point.timestamp), range),
      succeeded: point.succeeded,
      failed: point.failed,
    }));
  }, [data, range]);

  const total = rows.reduce((sum, r) => sum + r.succeeded + r.failed, 0);

  if (isLoading) {
    return (
      <div className="flex-1" style={{ minHeight: BAR_HEIGHT }}>
        <LoadingBlock label="Loading chart…" lines={0} />
      </div>
    );
  }

  if (!data || rows.length === 0 || total === 0) {
    return (
      <div className="flex min-h-[180px] flex-1 items-center justify-center border border-dashed border-edge px-5 py-8 text-center font-mono text-xs uppercase tracking-[0.06em] text-[#52525b]">
        {emptyLabel}
      </div>
    );
  }

  // Merge adjacent buckets down to MAX_BARS so dense windows stay readable.
  const groupSize = Math.max(1, Math.ceil(rows.length / MAX_BARS));
  const bars =
    groupSize === 1
      ? rows
      : Array.from({ length: Math.ceil(rows.length / groupSize) }, (_, g) => {
          const slice = rows.slice(g * groupSize, g * groupSize + groupSize);
          return {
            label: slice[0].label,
            succeeded: slice.reduce((s, r) => s + r.succeeded, 0),
            failed: slice.reduce((s, r) => s + r.failed, 0),
          };
        });

  const max = Math.max(1, ...bars.map((r) => r.succeeded + r.failed));
  const step = Math.max(1, Math.ceil(bars.length / MAX_LABELS));
  const labels = bars.filter((_, i) => i % step === 0).map((r) => r.label);

  return (
    <div>
      <div
        className="flex items-end gap-[3px] border-b border-l border-edge px-0.5"
        style={{ height: BAR_HEIGHT }}
      >
        {bars.map((r, i) => (
          <div
            key={i}
            className="flex h-full flex-1 flex-col justify-end"
            title={`${r.label} — ${r.succeeded} succeeded, ${r.failed} failed`}
          >
            {r.failed > 0 ? (
              <div
                style={{
                  height: `${(r.failed / max) * 100}%`,
                  minHeight: 2,
                  background: failedColor,
                }}
              />
            ) : null}
            {r.succeeded > 0 ? (
              <div
                style={{
                  height: `${(r.succeeded / max) * 100}%`,
                  minHeight: 2,
                  background: succeededColor,
                }}
              />
            ) : null}
          </div>
        ))}
      </div>

      <div className="mt-2 flex justify-between font-mono text-xs leading-none text-[#52525b]">
        {labels.map((label, i) => (
          <span key={i}>{label}</span>
        ))}
      </div>

      <div className="mt-3.5 flex justify-center gap-[18px] font-mono text-xs font-semibold uppercase leading-none tracking-[0.08em] text-soft">
        <span className="flex items-center gap-1.5">
          <span className="h-[9px] w-[9px]" style={{ background: failedColor }} />
          Failed
        </span>
        <span className="flex items-center gap-1.5">
          <span className="h-[9px] w-[9px]" style={{ background: succeededColor }} />
          Succeeded
        </span>
      </div>
    </div>
  );
}

function formatBucketLabel(ts: number, range: string): string {
  const d = new Date(ts);
  if (range === "30d") {
    return d.toLocaleDateString(undefined, { month: "short", day: "numeric" });
  }
  if (range === "7d") {
    return d.toLocaleString(undefined, {
      month: "short",
      day: "numeric",
      hour: "2-digit",
      hour12: false,
    });
  }
  return d.toLocaleTimeString(undefined, {
    hour: "2-digit",
    minute: "2-digit",
    hour12: false,
  });
}
