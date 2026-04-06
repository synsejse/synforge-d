import { cva, type VariantProps } from "class-variance-authority";
import type { HTMLAttributes } from "react";
import { cn } from "../../lib/utils";

const badgeVariants = cva(
  "inline-flex items-center gap-1.5 border-2 px-2.5 py-1 font-mono text-xs font-bold uppercase tracking-wider",
  {
    variants: {
      variant: {
        default:
          "border-[var(--theme-border-strong)] bg-[var(--theme-surface)] text-[var(--theme-text-muted)]",
        success:
          "border-[var(--theme-terminal-green)] bg-black text-[var(--theme-terminal-green)]",
        warning:
          "border-[var(--theme-accent-orange)] bg-black text-[var(--theme-accent-orange)]",
        error:
          "border-[var(--theme-error-red)] bg-black text-[var(--theme-error-red)]",
        lime: "border-[var(--theme-accent-lime)] bg-black text-[var(--theme-accent-lime)]",
        ghost:
          "border-transparent bg-[var(--theme-surface-hover)] text-[var(--theme-text)]",
      },
    },
    defaultVariants: {
      variant: "default",
    },
  }
);

export interface BadgeProps
  extends HTMLAttributes<HTMLSpanElement>,
    VariantProps<typeof badgeVariants> {
  pulse?: boolean;
}

export default function Badge({
  className,
  variant,
  pulse = false,
  children,
  ...props
}: BadgeProps) {
  return (
    <span
      className={cn(badgeVariants({ variant, className }))}
      {...props}
    >
      {pulse && (
        <span className="relative flex h-2 w-2">
          <span className="absolute inline-flex h-full w-full animate-ping bg-current opacity-75"></span>
          <span className="relative inline-flex h-2 w-2 bg-current"></span>
        </span>
      )}
      {children}
    </span>
  );
}
