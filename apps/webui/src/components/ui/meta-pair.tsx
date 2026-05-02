import type { ReactNode } from "react";
import { cn } from "../../lib/utils";

interface MetaPairProps {
  label: string;
  children: ReactNode;
  className?: string;
}

/**
 * Vertical label/value pair — small uppercase mono label on top, value
 * below. Designed to live inside a `flex flex-wrap` row so multiple
 * pairs read like a compact key/value table; long values (revisions,
 * job ids, repo paths) wrap inside their own cell instead of pushing
 * the next pair sideways.
 */
export default function MetaPair({ label, children, className }: MetaPairProps) {
  return (
    <div className={cn("flex min-w-0 flex-col gap-0.5", className)}>
      <span className="font-mono text-[10px] font-bold uppercase tracking-[0.22em] text-soft">
        {label}
      </span>
      <span className="min-w-0 break-all">{children}</span>
    </div>
  );
}
