interface Props {
  label: string;
  /**
   * Number of skeleton bars to render. Pass 0 for a label-only "typewriter"
   * style block (used in compact contexts like tab panels).
   */
  lines?: number;
}

/**
 * Brutalist loading state — label with a blinking cursor + a 4-cell
 * `▰▰▱▱` progress strip, optionally followed by skeleton bars.
 */
export default function LoadingBlock({ label, lines = 3 }: Props) {
  return (
    <div
      role="status"
      aria-live="polite"
      className="border border-edge bg-black p-6"
    >
      <div className="flex flex-wrap items-center gap-3">
        <span className="font-mono text-xs font-bold uppercase tracking-[0.18em] text-white">
          {label}
        </span>
        <span
          aria-hidden="true"
          className="inline-block h-4 w-[2px] animate-pulse bg-accent-lime"
        />
        <span aria-hidden="true" className="synforge-progress-glyph text-sm">
          <span>▰</span>
          <span>▰</span>
          <span>▰</span>
          <span>▰</span>
        </span>
      </div>
      {lines > 0 ? (
        <div className="mt-4 space-y-3">
          {Array.from({ length: lines }).map((_, index) => (
            <div
              key={index}
              className="h-4 animate-pulse border border-edge bg-surface-alt"
              style={{ width: `${Math.max(38, 100 - index * 12)}%` }}
            />
          ))}
        </div>
      ) : null}
    </div>
  );
}
