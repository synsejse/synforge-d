interface Props {
  label: string;
  value: string | number;
  detail?: string;
}

export default function MetricCard({ label, value, detail }: Props) {
  return (
    <article className="border border-zinc-800 bg-black p-5">
      <div className="text-xs uppercase tracking-[0.24em] text-zinc-400">{label}</div>
      <div className="mt-4 text-4xl font-semibold tracking-tight text-white">{value}</div>
      {detail ? <div className="mt-2 text-sm text-zinc-400">{detail}</div> : null}
    </article>
  );
}
