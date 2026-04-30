interface Props {
  label: string;
  lines?: number;
}

export default function LoadingBlock({ label, lines = 3 }: Props) {
  return (
    <div
      role="status"
      aria-live="polite"
      className="border-2 border-zinc-700 bg-black p-6"
    >
      <div className="font-mono text-xs font-bold uppercase tracking-[0.15em] text-zinc-400">{label}</div>
      <div className="mt-4 space-y-3">
        {Array.from({ length: lines }).map((_, index) => (
          <div
            key={index}
            className="h-4 animate-pulse border-2 border-zinc-700 bg-zinc-950"
            style={{ width: `${Math.max(38, 100 - index * 12)}%` }}
          />
        ))}
      </div>
    </div>
  );
}
