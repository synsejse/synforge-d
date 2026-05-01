import { cva, type VariantProps } from "class-variance-authority";
import type { ButtonHTMLAttributes } from "react";
import { cn } from "../../lib/utils";

const buttonVariants = cva(
  "inline-flex items-center justify-center gap-2 border-2 font-medium transition-all duration-100 ease-linear focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent-lime focus-visible:ring-offset-2 focus-visible:ring-offset-black disabled:pointer-events-none disabled:opacity-40",
  {
    variants: {
      variant: {
        primary:
          "bg-accent-lime text-black border-accent-lime hover:translate-x-[-2px] hover:translate-y-[-2px] shadow-brutal-sm hover:shadow-brutal-md",
        secondary:
          "border-white bg-white text-black shadow-[2px_2px_0_rgba(255,255,255,0.2)] hover:translate-x-[-1px] hover:translate-y-[-1px] hover:bg-strong",
        ghost:
          "bg-transparent text-strong border-edge-strong hover:bg-surface-hover hover:border-muted",
        danger:
          "border-error bg-error text-white shadow-brutal-sm hover:translate-x-[-2px] hover:translate-y-[-2px]",
        warning:
          "border-accent-orange bg-accent-orange text-black shadow-brutal-sm hover:translate-x-[-2px] hover:translate-y-[-2px]",
        terminal:
          "bg-black text-success border-success font-mono hover:bg-success hover:text-black",
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
