import type { ReactNode } from "react";
import { cn } from "../../lib/utils";

interface MetaPairProps {
  label: string;
  children: ReactNode;
  className?: string;
}

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
