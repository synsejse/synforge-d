import type { ReactNode } from "react";
import { cn } from "../../lib/utils";
import { Skeleton } from "./skeleton";

interface MetricCardProps {
  label: string;
  value: string | number;
  detail?: string;
  icon?: ReactNode;
  variant?: "default" | "accent" | "terminal" | "success" | "error";
  /**
   * Optional sparkline series (most-recent-last). Rendered as a row of
   * 2px-wide vertical bars at the bottom of the card. Pass null to skip.
   */
  sparkline?: ReadonlyArray<number> | null;
  /** When true, the label/value/detail/sparkline render as skeleton
   *  placeholders inside the real card frame. Useful so the page layout
   *  is fixed before the data lands. */
  loading?: boolean;
  className?: string;
}

export default function MetricCard({
  label,
  value,
  detail,
  icon,
  variant = "default",
  sparkline,
  loading = false,
  className,
}: MetricCardProps) {
  const borderColor = {
    default: "border-edge-strong",
    accent: "border-accent-lime",
    terminal: "border-success",
    success: "border-success",
    error: "border-error",
  }[variant];

  const textColor = {
    default: "text-white",
    accent: "text-accent-lime",
    terminal: "text-success",
    success: "text-success",
    error: "text-error",
  }[variant];

  const barColor = {
    default: "bg-accent-lime",
    accent: "bg-accent-lime",
    terminal: "bg-success",
    success: "bg-success",
    error: "bg-error",
  }[variant];

  return (
    <div
      className={cn(
        "group relative flex min-h-[10rem] flex-col border-4 bg-black p-6 transition-all duration-100 hover:translate-x-[-2px] hover:translate-y-[-2px] hover:shadow-[6px_6px_0_rgba(255,255,255,0.15)]",
        borderColor,
        className
      )}
    >
      <div className="pointer-events-none absolute inset-0 opacity-5 app-grid"></div>

      <div className="relative">
        {icon && (
          <div className="mb-4 inline-flex h-12 w-12 items-center justify-center border-2 border-edge-strong bg-surface-alt text-xl text-muted">
            {icon}
          </div>
        )}
        {loading ? (
          <>
            <Skeleton className="h-3 w-1/2" />
            <Skeleton className="mt-3 h-9 w-2/3" />
            <Skeleton className="mt-2 h-3 w-3/4" />
          </>
        ) : (
          <>
            <div className="font-mono text-xs font-bold uppercase tracking-[0.25em] text-soft">
              {label}
            </div>
            <div
              className={cn(
                "font-display mt-3 text-4xl font-black uppercase tracking-tighter",
                textColor,
              )}
            >
              {value}
            </div>
            {detail && (
              <div className="mt-2 font-mono text-xs text-muted">{detail}</div>
            )}
          </>
        )}
      </div>

      {/* Sparkline gutter — always reserve the slot so cards in a grid
          stay the same height regardless of which ones have a series. */}
      {loading ? (
        <Skeleton className="mt-auto mt-3 h-4 w-1/2" />
      ) : sparkline && sparkline.length > 0 ? (
        <Sparkline values={sparkline} barClass={barColor} />
      ) : (
        <div aria-hidden="true" className="mt-auto h-7 pt-3" />
      )}

      {variant !== "default" && (
        <div
          className={cn(
            "absolute bottom-0 right-0 h-3 w-3",
            variant === "accent" && "bg-accent-lime",
            variant === "terminal" && "bg-success",
            variant === "success" && "bg-success",
            variant === "error" && "bg-error"
          )}
        ></div>
      )}
    </div>
  );
}

/**
 * Brutalist sparkline — fixed-width 2px bars, no smoothing, sat at the
 * bottom of the card. Each bar's height encodes the value relative to
 * the series max. Empty/zero series renders a flat baseline.
 */
function Sparkline({
  values,
  barClass,
}: {
  values: ReadonlyArray<number>;
  barClass: string;
}) {
  const max = Math.max(1, ...values.map((v) => Math.max(0, v)));
  return (
    <div
      aria-hidden="true"
      className="relative mt-auto flex h-7 items-end gap-px pt-3"
    >
      {values.map((raw, idx) => {
        const v = Math.max(0, raw);
        const pct = Math.max(2, Math.round((v / max) * 100));
        return (
          <div
            key={idx}
            className={cn("w-[2px] shrink-0", barClass, v === 0 && "opacity-30")}
            style={{ height: `${pct}%` }}
          />
        );
      })}
    </div>
  );
}
