import type { ReactNode } from "react";
import { cn } from "../../lib/utils";

interface MetricCardProps {
  label: string;
  value: string | number;
  detail?: string;
  icon?: ReactNode;
  variant?: "default" | "accent" | "terminal" | "success" | "error";
  className?: string;
}

export default function MetricCard({
  label,
  value,
  detail,
  icon,
  variant = "default",
  className,
}: MetricCardProps) {
  const borderColor = {
    default: "border-[var(--theme-border-strong)]",
    accent: "border-[var(--theme-accent-lime)]",
    terminal: "border-[var(--theme-terminal-green)]",
    success: "border-[var(--theme-terminal-green)]",
    error: "border-[var(--theme-error-red)]",
  }[variant];

  const textColor = {
    default: "text-white",
    accent: "text-[var(--theme-accent-lime)]",
    terminal: "text-[var(--theme-terminal-green)]",
    success: "text-[var(--theme-terminal-green)]",
    error: "text-[var(--theme-error-red)]",
  }[variant];

  return (
    <div
      className={cn(
        "group relative border-4 bg-black p-6 transition-all duration-100 hover:translate-x-[-2px] hover:translate-y-[-2px] hover:shadow-[6px_6px_0_rgba(255,255,255,0.15)]",
        borderColor,
        className
      )}
    >
      {/* Grid pattern overlay */}
      <div className="pointer-events-none absolute inset-0 opacity-5 app-grid"></div>
      
      <div className="relative">
        {icon && (
          <div className="mb-4 inline-flex h-12 w-12 items-center justify-center border-2 border-zinc-700 bg-zinc-950 text-xl text-zinc-400">
            {icon}
          </div>
        )}
        <div className="font-mono text-xs font-bold uppercase tracking-[0.25em] text-[var(--theme-text-soft)]">
          {label}
        </div>
        <div className={cn("font-display mt-3 text-4xl font-black uppercase tracking-tighter", textColor)}>
          {value}
        </div>
        {detail && (
          <div className="mt-2 font-mono text-xs text-[var(--theme-text-muted)]">
            {detail}
          </div>
        )}
      </div>
      
      {/* Accent corner */}
      {variant !== "default" && (
        <div
          className={cn(
            "absolute bottom-0 right-0 h-3 w-3",
            variant === "accent" && "bg-[var(--theme-accent-lime)]",
            variant === "terminal" && "bg-[var(--theme-terminal-green)]",
            variant === "success" && "bg-[var(--theme-terminal-green)]",
            variant === "error" && "bg-[var(--theme-error-red)]"
          )}
        ></div>
      )}
    </div>
  );
}
