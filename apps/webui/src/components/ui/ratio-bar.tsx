import type { ReactNode } from "react";
import { cn } from "../../lib/utils";

export interface RatioSegment {
  label: string;
  value: number;
  /** CSS color (hex / var()). Defaults rotate through a brutalist palette. */
  color?: string;
}

interface RatioBarProps {
  segments: RatioSegment[];
  className?: string;
  /** Render below the bar as label/value pairs. Default true. */
  showLegend?: boolean;
  /** Optional summary line above the bar. */
  caption?: ReactNode;
}

const DEFAULT_PALETTE = [
  "var(--theme-accent-lime)",
  "var(--theme-terminal-green)",
  "var(--theme-accent-orange)",
  "var(--theme-accent-cyan)",
  "var(--theme-accent-violet)",
  "var(--theme-error-red)",
];

/**
 * Horizontal stacked bar showing the proportional split of a fixed total.
 * Use for hit/miss/stale distributions, success/failure ratios, etc.
 */
export default function RatioBar({
  segments,
  className,
  showLegend = true,
  caption,
}: RatioBarProps) {
  const total = segments.reduce((acc, seg) => acc + Math.max(0, seg.value), 0);

  if (total === 0) {
    return (
      <div
        className={cn(
          "border-2 border-[var(--theme-border-strong)] bg-[var(--theme-surface-alt)] p-4 font-mono text-xs uppercase tracking-[0.15em] text-[var(--theme-text-soft)]",
          className,
        )}
      >
        No data
      </div>
    );
  }

  return (
    <div className={cn("space-y-3", className)}>
      {caption ? (
        <div className="font-mono text-xs uppercase tracking-[0.15em] text-[var(--theme-text-soft)]">
          {caption}
        </div>
      ) : null}
      <div
        className="flex h-6 w-full overflow-hidden border-2 border-[var(--theme-border-strong)]"
        role="img"
        aria-label={segments
          .map((s) => `${s.label} ${s.value} of ${total}`)
          .join(", ")}
      >
        {segments.map((seg, idx) => {
          const percent = (Math.max(0, seg.value) / total) * 100;
          if (percent === 0) return null;
          return (
            <div
              key={`${seg.label}-${idx}`}
              style={{
                width: `${percent}%`,
                backgroundColor: seg.color ?? DEFAULT_PALETTE[idx % DEFAULT_PALETTE.length],
              }}
              title={`${seg.label}: ${seg.value} (${percent.toFixed(1)}%)`}
            />
          );
        })}
      </div>
      {showLegend ? (
        <div className="flex flex-wrap gap-x-4 gap-y-2 font-mono text-xs">
          {segments.map((seg, idx) => {
            const percent = total > 0 ? (Math.max(0, seg.value) / total) * 100 : 0;
            return (
              <div
                key={`legend-${seg.label}-${idx}`}
                className="flex items-center gap-2"
              >
                <span
                  className="inline-block h-3 w-3 border border-black"
                  style={{
                    backgroundColor:
                      seg.color ?? DEFAULT_PALETTE[idx % DEFAULT_PALETTE.length],
                  }}
                  aria-hidden="true"
                />
                <span className="uppercase tracking-[0.12em] text-[var(--theme-text-muted)]">
                  {seg.label}
                </span>
                <span className="font-bold text-white tabular-nums">
                  {seg.value}
                </span>
                <span className="text-[var(--theme-text-soft)] tabular-nums">
                  ({percent.toFixed(0)}%)
                </span>
              </div>
            );
          })}
        </div>
      ) : null}
    </div>
  );
}
