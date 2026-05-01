import { cva, type VariantProps } from "class-variance-authority";
import type { HTMLAttributes } from "react";
import { cn } from "../../lib/utils";

const badgeVariants = cva(
  "inline-flex items-center gap-1.5 border-2 px-2.5 py-1 font-mono text-xs font-bold uppercase tracking-wider",
  {
    variants: {
      variant: {
        default:
          "border-edge-strong bg-surface text-muted",
        success:
          "border-success bg-black text-success",
        warning:
          "border-accent-orange bg-black text-accent-orange",
        error:
          "border-error bg-black text-error",
        lime: "border-accent-lime bg-black text-accent-lime",
        ghost:
          "border-transparent bg-surface-hover text-strong",
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
