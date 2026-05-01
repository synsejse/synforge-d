interface Props {
  label: string;
  value: string | number;
  mono?: boolean;
}

export default function DetailStat({ label, value, mono = false }: Props) {
  return (
    <div className="border-2 border-edge-strong bg-black px-4 py-3">
      <div className="font-mono text-xs uppercase tracking-[0.18em] text-soft">{label}</div>
      <div className={`mt-2 break-all text-sm text-strong ${mono ? "font-mono" : ""}`}>{value}</div>
    </div>
  );
}
