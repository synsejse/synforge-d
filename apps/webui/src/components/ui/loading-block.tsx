interface Props {
  label: string;
  /** Pass 0 to render the label-only block (no skeleton bars). */
  lines?: number;
}

export default function LoadingBlock({ label, lines = 3 }: Props) {
  return (
    <div
      role="status"
      aria-live="polite"
      className="border-2 border-edge-strong bg-black p-6"
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
              className="h-4 animate-pulse border-2 border-edge-strong bg-surface-alt"
              style={{ width: `${Math.max(38, 100 - index * 12)}%` }}
            />
          ))}
        </div>
      ) : null}
    </div>
  );
}
