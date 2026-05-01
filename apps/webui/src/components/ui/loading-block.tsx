interface Props {
  label: string;
  lines?: number;
}

export default function LoadingBlock({ label, lines = 3 }: Props) {
  return (
    <div
      role="status"
      aria-live="polite"
      className="border-2 border-edge-strong bg-black p-6"
    >
      <div className="font-mono text-xs font-bold uppercase tracking-[0.15em] text-muted">{label}</div>
      <div className="mt-4 space-y-3">
        {Array.from({ length: lines }).map((_, index) => (
          <div
            key={index}
            className="h-4 animate-pulse border-2 border-edge-strong bg-surface-alt"
            style={{ width: `${Math.max(38, 100 - index * 12)}%` }}
          />
        ))}
      </div>
    </div>
  );
}
