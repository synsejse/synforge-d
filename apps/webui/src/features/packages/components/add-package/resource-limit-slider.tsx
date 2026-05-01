interface ResourceLimitSliderProps {
  enabled: boolean;
  label: string;
  max: number;
  min: number;
  onEnabledChange: (enabled: boolean) => void;
  onValueChange: (value: string) => void;
  step: number;
  toggleLabel: string;
  unit: string;
  value: number;
}

export default function ResourceLimitSlider({
  enabled,
  label,
  max,
  min,
  onEnabledChange,
  onValueChange,
  step,
  toggleLabel,
  unit,
  value,
}: ResourceLimitSliderProps) {
  return (
    <div className="border-2 border-edge-strong bg-surface-alt p-4 lg:col-span-2">
      <div className="flex flex-col gap-3 md:flex-row md:items-center md:justify-between">
        <span className="font-mono text-xs font-bold uppercase tracking-[0.15em] text-muted">
          {label}
        </span>
        <label className="flex items-center gap-2 font-mono text-xs font-bold uppercase tracking-[0.12em] text-muted">
          <input
            type="checkbox"
            checked={enabled}
            onChange={(event) => onEnabledChange(event.target.checked)}
          />
          {toggleLabel}
        </label>
      </div>
      <div
        className={`mt-4 border-2 border-edge px-3 py-4 transition ${
          enabled ? "bg-black" : "bg-surface-hover/60 opacity-70"
        }`}
      >
        <input
          type="range"
          min={min}
          max={max}
          step={step}
          value={value}
          onChange={(event) => onValueChange(event.target.value)}
          disabled={!enabled}
          aria-label={label}
          className="h-3 w-full cursor-pointer appearance-none bg-surface-hover accent-accent-lime disabled:cursor-not-allowed [&::-moz-range-thumb]:h-5 [&::-moz-range-thumb]:w-3 [&::-moz-range-thumb]:border-2 [&::-moz-range-thumb]:border-black [&::-moz-range-thumb]:bg-accent-lime [&::-webkit-slider-thumb]:h-5 [&::-webkit-slider-thumb]:w-3 [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:border-2 [&::-webkit-slider-thumb]:border-black [&::-webkit-slider-thumb]:bg-accent-lime"
        />
      </div>
      <div className="mt-3 flex items-center justify-between gap-3 font-mono text-xs font-bold uppercase tracking-[0.12em]">
        <span className="text-accent-lime">
          {enabled ? `${value} ${unit}` : "Unlimited"}
        </span>
        <span className="text-soft">
          {min} - {max} {unit}
        </span>
      </div>
    </div>
  );
}
