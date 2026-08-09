import { useId, useMemo } from "react";
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
  /** Accessible name used by the screen-reader data table. */
  ariaLabel?: string;
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
  succeededColor = "var(--theme-terminal-green)",
  failedColor = "var(--theme-error-red)",
  emptyLabel = "No activity in this window.",
  ariaLabel = "Activity outcomes over time",
}: Props) {
  const captionId = useId();
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
  const succeededTotal = rows.reduce((sum, row) => sum + row.succeeded, 0);
  const failedTotal = rows.reduce((sum, row) => sum + row.failed, 0);

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
    <figure aria-labelledby={captionId}>
      <figcaption id={captionId} className="sr-only">
        {ariaLabel}. {total} total events: {succeededTotal} succeeded and{" "}
        {failedTotal} failed.
      </figcaption>

      <div className="mb-3 flex flex-wrap gap-x-5 gap-y-1 font-mono text-xs text-soft">
        <span>
          Total <strong className="text-strong">{total}</strong>
        </span>
        <span>
          Succeeded <strong className="text-success">{succeededTotal}</strong>
        </span>
        <span>
          Failed <strong className="text-error">{failedTotal}</strong>
        </span>
        <span>
          Peak / bucket <strong className="text-strong">{max}</strong>
        </span>
      </div>

      <div aria-hidden="true" className="grid grid-cols-[auto_minmax(0,1fr)] gap-2">
        <div
          className="relative w-8 font-mono text-xs tabular-nums text-soft"
          style={{ height: BAR_HEIGHT }}
        >
          <span className="absolute right-0 top-0">{max}</span>
          <span className="absolute bottom-0 right-0">0</span>
        </div>
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
      </div>

      <div aria-hidden="true" className="ml-10 mt-2 flex justify-between font-mono text-xs leading-none text-soft">
        {labels.map((label, i) => (
          <span key={i}>{label}</span>
        ))}
      </div>

      <div aria-hidden="true" className="mt-3.5 flex justify-center gap-[18px] font-mono text-xs font-semibold uppercase leading-none tracking-[0.08em] text-soft">
        <span className="flex items-center gap-1.5">
          <span className="h-[9px] w-[9px]" style={{ background: failedColor }} />
          Failed
        </span>
        <span className="flex items-center gap-1.5">
          <span className="h-[9px] w-[9px]" style={{ background: succeededColor }} />
          Succeeded
        </span>
      </div>

      <table className="sr-only">
        <caption>{ariaLabel} data</caption>
        <thead>
          <tr>
            <th scope="col">Time bucket</th>
            <th scope="col">Succeeded</th>
            <th scope="col">Failed</th>
          </tr>
        </thead>
        <tbody>
          {rows.map((row, index) => (
            <tr key={`${row.label}-${index}`}>
              <th scope="row">{row.label}</th>
              <td>{row.succeeded}</td>
              <td>{row.failed}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </figure>
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
