interface Props {
  status: string;
}

export default function StatusPill({ status }: Props) {
  const classes: Record<string, string> = {
    pending: "border-accent-orange bg-black text-accent-orange",
    running: "border-accent-lime bg-black text-accent-lime",
    succeeded: "border-success bg-black text-success",
    failed: "border-error bg-black text-error",
    timed_out: "border-error bg-black text-error",
    enabled: "border-success bg-black text-success",
    disabled: "border-edge-strong bg-black text-muted",
  };
  const dots: Record<string, string> = {
    pending: "bg-accent-orange",
    running: "bg-accent-lime",
    succeeded: "bg-success",
    failed: "bg-error",
    timed_out: "bg-error",
    enabled: "bg-success",
    disabled: "bg-soft",
  };

  return (
    <span className={`inline-flex items-center gap-2 border-2 px-3 py-1 font-mono text-xs font-bold uppercase tracking-[0.1em] ${classes[status] || "border-edge-strong bg-black text-strong"}`}>
      <span className={`h-2 w-2 ${dots[status] || "bg-muted"}`}></span>
      {status}
    </span>
  );
}
