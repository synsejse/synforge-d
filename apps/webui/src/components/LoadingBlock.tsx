interface Props {
  label: string;
  lines?: number;
}

export default function LoadingBlock({ label, lines = 3 }: Props) {
  return (
    <div
      role="status"
      aria-live="polite"
      className="border border-zinc-800 bg-black p-6"
    >
      <div className="text-sm text-zinc-400">{label}</div>
      <div className="mt-4 space-y-3">
        {Array.from({ length: lines }).map((_, index) => (
          <div
            key={index}
            className="h-4 animate-pulse border border-zinc-800 bg-zinc-950"
            style={{ width: `${Math.max(38, 100 - index * 12)}%` }}
          />
        ))}
      </div>
    </div>
  );
}
