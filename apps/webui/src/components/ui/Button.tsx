import { cva, type VariantProps } from "class-variance-authority";
import type { ButtonHTMLAttributes } from "react";
import { cn } from "../../lib/utils";

const buttonVariants = cva(
  "inline-flex items-center justify-center gap-2 border-2 font-medium transition-all duration-100 ease-linear focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--theme-accent-lime)] focus-visible:ring-offset-2 focus-visible:ring-offset-black disabled:pointer-events-none disabled:opacity-40",
  {
    variants: {
      variant: {
        primary:
          "bg-[var(--theme-accent-lime)] text-black border-[var(--theme-accent-lime)] hover:translate-x-[-2px] hover:translate-y-[-2px] shadow-[2px_2px_0_rgba(0,0,0,0.8)] hover:shadow-[4px_4px_0_rgba(0,0,0,0.9)]",
        secondary:
          "border-white bg-white text-black shadow-[2px_2px_0_rgba(255,255,255,0.2)] hover:translate-x-[-1px] hover:translate-y-[-1px] hover:bg-[var(--theme-text-strong)]",
        ghost:
          "bg-transparent text-[var(--theme-text-strong)] border-[var(--theme-border-strong)] hover:bg-[var(--theme-surface-hover)] hover:border-[var(--theme-text-muted)]",
        danger:
          "border-[var(--theme-error-red)] bg-[var(--theme-error-red)] text-white shadow-[2px_2px_0_rgba(0,0,0,0.8)] hover:translate-x-[-2px] hover:translate-y-[-2px]",
        warning:
          "border-[var(--theme-accent-orange)] bg-[var(--theme-accent-orange)] text-black shadow-[2px_2px_0_rgba(0,0,0,0.8)] hover:translate-x-[-2px] hover:translate-y-[-2px]",
        terminal:
          "bg-black text-[var(--theme-terminal-green)] border-[var(--theme-terminal-green)] font-mono hover:bg-[var(--theme-terminal-green)] hover:text-black",
      },
      size: {
        sm: "px-3 py-1.5 text-xs font-semibold uppercase tracking-wider",
        md: "px-4 py-2.5 text-sm font-semibold",
        lg: "px-6 py-3.5 text-base font-bold",
        icon: "h-9 w-9 p-0",
      },
    },
    defaultVariants: {
      variant: "ghost",
      size: "md",
    },
  }
);

export interface ButtonProps
  extends ButtonHTMLAttributes<HTMLButtonElement>,
    VariantProps<typeof buttonVariants> {}

export default function Button({
  className,
  variant,
  size,
  ...props
}: ButtonProps) {
  return (
    <button
      className={cn(buttonVariants({ variant, size, className }))}
      {...props}
    />
  );
}
