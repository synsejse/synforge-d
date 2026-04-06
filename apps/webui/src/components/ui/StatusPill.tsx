interface Props {
  status: string;
}

export default function StatusPill({ status }: Props) {
  const classes: Record<string, string> = {
    pending: "border-[var(--theme-accent-orange)] bg-black text-[var(--theme-accent-orange)]",
    running: "border-[var(--theme-accent-lime)] bg-black text-[var(--theme-accent-lime)]",
    completed: "border-[var(--theme-terminal-green)] bg-black text-[var(--theme-terminal-green)]",
    succeeded: "border-[var(--theme-terminal-green)] bg-black text-[var(--theme-terminal-green)]",
    failed: "border-[var(--theme-error-red)] bg-black text-[var(--theme-error-red)]",
    timed_out: "border-[var(--theme-error-red)] bg-black text-[var(--theme-error-red)]",
    enabled: "border-[var(--theme-terminal-green)] bg-black text-[var(--theme-terminal-green)]",
    disabled: "border-[var(--theme-border-strong)] bg-black text-zinc-400",
  };
  const dots: Record<string, string> = {
    pending: "bg-[var(--theme-accent-orange)]",
    running: "bg-[var(--theme-accent-lime)]",
    completed: "bg-[var(--theme-terminal-green)]",
    succeeded: "bg-[var(--theme-terminal-green)]",
    failed: "bg-[var(--theme-error-red)]",
    timed_out: "bg-[var(--theme-error-red)]",
    enabled: "bg-[var(--theme-terminal-green)]",
    disabled: "bg-zinc-500",
  };

  return (
    <span className={`inline-flex items-center gap-2 border-2 px-3 py-1 font-mono text-xs font-bold uppercase tracking-[0.1em] ${classes[status] || "border-[var(--theme-border-strong)] bg-black text-zinc-200"}`}>
      <span className={`h-2 w-2 ${dots[status] || "bg-zinc-400"}`}></span>
      {status}
    </span>
  );
}
