interface Props {
  label: string;
  value: string;
  mono?: boolean;
}

export default function DetailStat({ label, value, mono = false }: Props) {
  return (
    <div className="border border-zinc-800 bg-black px-4 py-3">
      <div className="text-xs uppercase tracking-[0.18em] text-zinc-500">{label}</div>
      <div className={`mt-2 break-all text-sm text-zinc-100 ${mono ? "font-mono" : ""}`}>{value}</div>
    </div>
  );
}
