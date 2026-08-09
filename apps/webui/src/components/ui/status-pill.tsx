interface Props {
  status: string;
}

export default function StatusPill({ status }: Props) {
  const classes: Record<string, string> = {
    pending: "border-accent-orange bg-black text-accent-orange",
    queued: "border-accent-orange bg-black text-accent-orange",
    running: "border-accent-lime bg-black text-accent-lime",
    succeeded: "border-success bg-black text-success",
    failed: "border-error bg-black text-error",
    timed_out: "border-error bg-black text-error",
    cancelled: "border-edge-strong bg-black text-soft",
    interrupted: "border-error bg-black text-error",
    enabled: "border-success bg-black text-success",
    disabled: "border-edge-strong bg-black text-muted",
  };
  const dots: Record<string, string> = {
    pending: "bg-accent-orange",
    queued: "bg-accent-orange",
    running: "bg-accent-lime",
    succeeded: "bg-success",
    failed: "bg-error",
    timed_out: "bg-error",
    cancelled: "bg-soft",
    interrupted: "bg-error",
    enabled: "bg-success",
    disabled: "bg-soft",
  };

  return (
    <span className={`inline-flex items-center gap-1.5 border px-2 py-[5px] font-mono text-[9px] font-semibold uppercase leading-none tracking-[0.1em] ${classes[status] || "border-edge-strong bg-black text-strong"}`}>
      <span className={`h-[5px] w-[5px] ${dots[status] || "bg-muted"}`}></span>
      {status}
    </span>
  );
}
