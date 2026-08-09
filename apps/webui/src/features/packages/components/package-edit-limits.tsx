import {
  CCACHE_SUPPORTED_ARCHES,
  incompatibleCcacheChroots,
} from "../../../lib/utils";

interface ResourceLimitCardProps {
  label: string;
  unit: string;
  checkboxLabel: string;
  enabled: boolean;
  value: number;
  min: number;
  max: number;
  step: number;
  onToggle: (next: boolean) => void;
  onSliderChange: (value: number) => void;
}

export function ResourceLimitCard({
  label,
  unit,
  checkboxLabel,
  enabled,
  value,
  min,
  max,
  step,
  onToggle,
  onSliderChange,
}: ResourceLimitCardProps) {
  return (
    <div className="border border-edge bg-surface-alt/40 p-4">
      <div className="flex flex-col gap-3 md:flex-row md:items-center md:justify-between">
        <span className="font-mono text-xs font-bold uppercase tracking-[0.15em] text-muted">
          {label}
        </span>
        <label className="flex items-center gap-2 font-mono text-xs font-bold uppercase tracking-[0.12em] text-muted">
          <input
            type="checkbox"
            checked={enabled}
            onChange={(event) => onToggle(event.target.checked)}
          />
          {checkboxLabel}
        </label>
      </div>
      <div
        className={`mt-3 transition ${
          enabled ? "" : "opacity-40 pointer-events-none"
        }`}
      >
        <input
          type="range"
          min={min}
          max={max}
          step={step}
          value={value}
          onChange={(event) => onSliderChange(Number(event.target.value))}
          disabled={!enabled}
          aria-label={label}
          className="h-2 w-full cursor-pointer appearance-none bg-edge-strong accent-accent-lime disabled:cursor-not-allowed [&::-moz-range-thumb]:h-4 [&::-moz-range-thumb]:w-3 [&::-moz-range-thumb]:border [&::-moz-range-thumb]:border-black [&::-moz-range-thumb]:bg-accent-lime [&::-webkit-slider-thumb]:h-4 [&::-webkit-slider-thumb]:w-3 [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:border [&::-webkit-slider-thumb]:border-black [&::-webkit-slider-thumb]:bg-accent-lime"
        />
        <div className="mt-2 flex items-center justify-between gap-3 font-mono text-xs font-bold uppercase tracking-[0.12em]">
          <span className="text-accent-lime">
            {enabled ? `${value} ${unit}` : "Unlimited"}
          </span>
          <span className="text-soft">
            {min} – {max} {unit}
          </span>
        </div>
      </div>
    </div>
  );
}

export function CcacheCompatibilityNotice({ chroots }: { chroots: string[] }) {
  const incompatible = incompatibleCcacheChroots(chroots);
  const supportedList = CCACHE_SUPPORTED_ARCHES.join(", ");

  if (incompatible.length === 0) {
    return (
      <p className="border-l border-edge-strong px-3 py-2 font-mono text-xs text-soft">
        ccache only works with{" "}
        <span className="text-strong">{supportedList}</span>. Builds for any
        other arch will fail or silently bypass the cache.
      </p>
    );
  }

  return (
    <div className="border border-accent-orange bg-black px-3 py-2">
      <p className="font-mono text-xs font-bold uppercase tracking-[0.22em] text-accent-orange">
        ccache incompatible with selected targets
      </p>
      <p className="mt-1 font-mono text-xs text-strong">
        These chroots will fail or skip ccache:{" "}
        <span className="text-accent-orange">{incompatible.join(", ")}</span>.
        Mock&apos;s ccache plugin only supports{" "}
        <span className="text-strong">{supportedList}</span>.
      </p>
    </div>
  );
}
