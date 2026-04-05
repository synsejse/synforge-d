interface Props {
  status: string;
}

export default function StatusPill({ status }: Props) {
  const classes: Record<string, string> = {
    pending: "border-amber-700/60 bg-amber-500/10 text-amber-300",
    running: "border-blue-700/60 bg-blue-500/10 text-blue-300",
    succeeded: "border-emerald-700/60 bg-emerald-500/10 text-emerald-300",
    failed: "border-red-700/60 bg-red-500/10 text-red-300",
    timed_out: "border-orange-700/60 bg-orange-500/10 text-orange-300",
    enabled: "border-emerald-700/60 bg-emerald-500/10 text-emerald-300",
    disabled: "border-zinc-800 bg-black text-zinc-400",
  };
  const dots: Record<string, string> = {
    pending: "bg-amber-400",
    running: "bg-blue-400",
    succeeded: "bg-emerald-400",
    failed: "bg-red-400",
    timed_out: "bg-orange-400",
    enabled: "bg-emerald-400",
    disabled: "bg-zinc-500",
  };

  return (
    <span className={`inline-flex items-center gap-2 border px-3 py-1 text-xs font-medium ${classes[status] || "border-zinc-800 bg-black text-zinc-200"}`}>
      <span className={`h-2 w-2 ${dots[status] || "bg-zinc-400"}`}></span>
      {status}
    </span>
  );
}
