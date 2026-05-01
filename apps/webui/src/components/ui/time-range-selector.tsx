import type { TimeRange } from "../../lib/types";

interface Props {
  value: TimeRange;
  onChange: (next: TimeRange) => void;
  className?: string;
}

const OPTIONS: ReadonlyArray<{ value: TimeRange; label: string }> = [
  { value: "24h", label: "24h" },
  { value: "7d", label: "7d" },
  { value: "30d", label: "30d" },
];

/**
 * Compact segmented selector for the statistics page time window.
 * Brutalist visual: bordered group, lime fill on the selected segment.
 */
export default function TimeRangeSelector({ value, onChange, className = "" }: Props) {
  return (
    <div
      role="group"
      aria-label="Time range"
      className={`inline-flex border-2 border-edge-strong ${className}`}
    >
      {OPTIONS.map((opt, idx) => {
        const active = opt.value === value;
        return (
          <button
            key={opt.value}
            type="button"
            onClick={() => onChange(opt.value)}
            aria-pressed={active}
            className={`px-4 py-2 font-mono text-xs font-bold uppercase tracking-[0.18em] transition-colors ${
              idx > 0 ? "border-l-2 border-edge-strong" : ""
            } ${
              active
                ? "bg-accent-lime text-black"
                : "bg-black text-muted hover:text-white"
            }`}
          >
            {opt.label}
          </button>
        );
      })}
    </div>
  );
}
