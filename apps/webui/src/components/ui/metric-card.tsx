import type { ReactNode } from "react";
import { cn } from "../../lib/utils";

interface MetricCardProps {
  label: string;
  value: string | number;
  detail?: string;
  icon?: ReactNode;
  /**
   * Highlight the value in the brand accent and surface a small tag in the
   * top-right (defaults to "LIVE"). Use for streaming/active metrics — e.g.
   * the active-jobs counter — so a single card stands out without breaking
   * the uniform neutral grid.
   */
  live?: boolean;
  /** Override the top-right tag text shown while `live`. */
  tag?: string;
  className?: string;
}

/**
 * Terminal-control metric tile. Uniform 1px neutral frame across the grid;
 * the value is white by default and switches to the brand accent only when
 * `live`, paired with a faint accent wash and a tag. Mirrors the dashboard
 * stat cards in the design comp.
 */
export default function MetricCard({
  label,
  value,
  detail,
  icon,
  live = false,
  tag = "LIVE",
  className,
}: MetricCardProps) {
  return (
    <div
      className={cn(
        "group relative flex min-h-[9rem] flex-col border border-edge bg-black p-[18px] transition-colors",
        live && "bg-gradient-to-b from-accent-lime/[0.07] to-black",
        className,
      )}
    >
      <div className="flex items-center justify-between">
        {icon ? (
          <div
            className={cn(
              "inline-flex h-9 w-9 items-center justify-center border border-edge text-base",
              live ? "text-accent-lime" : "text-muted",
            )}
          >
            {icon}
          </div>
        ) : (
          <span />
        )}
        {live ? (
          <span className="inline-flex items-center gap-1.5 border border-accent-lime px-1.5 py-0.5 font-mono text-[9px] font-bold uppercase tracking-[0.18em] text-accent-lime">
            <span
              aria-hidden="true"
              className="h-1.5 w-1.5 animate-pulse bg-accent-lime"
            />
            {tag}
          </span>
        ) : null}
      </div>

      <div className="mt-5 font-mono text-[10px] font-bold uppercase tracking-[0.22em] text-soft">
        {label}
      </div>
      <div
        className={cn(
          "mt-2.5 whitespace-nowrap font-mono text-[2rem] font-extrabold leading-none tracking-tight tabular-nums",
          live ? "text-accent-lime" : "text-white",
        )}
      >
        {value}
      </div>
      {detail ? (
        <div className="font-body mt-2 text-xs text-muted">{detail}</div>
      ) : null}
    </div>
  );
}
