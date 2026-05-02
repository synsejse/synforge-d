import { cva, type VariantProps } from "class-variance-authority";
import type { ButtonHTMLAttributes, ReactNode } from "react";
import { faSpinner } from "@fortawesome/free-solid-svg-icons";
import { cn } from "../../lib/utils";
import FaIcon from "./fa-icon";

/**
 * Brutalist button system.
 *
 * Size scale (matches form fields where it makes sense):
 *   xs       — inline mini (e.g. reset-to-default), ~24px tall
 *   sm       — toolbar / dense action, 32px tall
 *   md       — default, 40px tall, pairs with <Select> (44px) and most form rows
 *   lg       — primary submit, 48px tall, matches <input>/<textarea> height
 *   icon-sm  — square 32px (paired with size sm)
 *   icon     — square 36px (default icon-only — sidebar, dialog close)
 *   icon-lg  — square 44px (paired with size md/lg)
 *
 * Pass icons as children — wrap a `<FaIcon>` next to the label text. The
 * built-in `gap-2` (or per-size override) takes care of spacing.
 */
export const buttonVariants = cva(
  [
    "inline-flex items-center justify-center gap-2 border-2 font-medium",
    "transition-[color,background-color,border-color,box-shadow,transform] duration-100 ease-linear",
    "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent-lime focus-visible:ring-offset-2 focus-visible:ring-offset-black",
    "disabled:pointer-events-none disabled:opacity-40",
  ].join(" "),
  {
    variants: {
      variant: {
        primary: [
          "bg-accent-lime text-black border-accent-lime shadow-brutal-sm",
          "hover:shadow-brutal-md hover:-translate-x-px hover:-translate-y-px",
          "active:translate-x-0 active:translate-y-0 active:shadow-brutal-sm",
        ].join(" "),
        secondary: [
          "bg-white text-black border-white shadow-[2px_2px_0_rgba(255,255,255,0.2)]",
          "hover:bg-strong hover:-translate-x-px hover:-translate-y-px",
          "active:translate-x-0 active:translate-y-0",
        ].join(" "),
        ghost:
          "bg-transparent text-strong border-edge-strong hover:border-muted hover:bg-surface-hover hover:text-white",
        subtle:
          "bg-transparent text-soft border-transparent hover:bg-surface-hover hover:text-white",
        danger: [
          "bg-error text-white border-error shadow-brutal-sm",
          "hover:shadow-brutal-md hover:-translate-x-px hover:-translate-y-px",
          "active:translate-x-0 active:translate-y-0 active:shadow-brutal-sm",
        ].join(" "),
        warning: [
          "bg-accent-orange text-black border-accent-orange shadow-brutal-sm",
          "hover:shadow-brutal-md hover:-translate-x-px hover:-translate-y-px",
          "active:translate-x-0 active:translate-y-0 active:shadow-brutal-sm",
        ].join(" "),
        terminal:
          "bg-black text-success border-success font-mono hover:bg-success hover:text-black",
      },
      size: {
        xs: "px-2 py-1 text-[10px] font-bold uppercase tracking-[0.18em] gap-1",
        sm: "px-3 py-1.5 text-xs font-bold uppercase tracking-[0.16em]",
        md: "px-4 py-2 text-sm font-semibold",
        lg: "px-5 py-3 text-base font-bold",
        "icon-sm": "h-8 w-8 p-0",
        icon: "h-9 w-9 p-0",
        "icon-lg": "h-11 w-11 p-0",
      },
      fullWidth: {
        true: "w-full",
        responsive: "w-full sm:w-auto",
        false: "",
      },
    },
    compoundVariants: [
      // Bigger gap for larger sizes when icon + text are combined.
      { size: "lg", className: "gap-3" },
    ],
    defaultVariants: {
      variant: "ghost",
      size: "md",
      fullWidth: false,
    },
  },
);

export type ButtonVariantProps = VariantProps<typeof buttonVariants>;

export interface ButtonProps
  extends Omit<ButtonHTMLAttributes<HTMLButtonElement>, "children">,
    ButtonVariantProps {
  /** Renders a leading spinner and disables the button while truthy. */
  loading?: boolean;
  children?: ReactNode;
}

export default function Button({
  className,
  variant,
  size,
  fullWidth,
  loading = false,
  disabled,
  children,
  type = "button",
  ...props
}: ButtonProps) {
  return (
    <button
      type={type}
      disabled={disabled || loading}
      aria-busy={loading || undefined}
      className={cn(buttonVariants({ variant, size, fullWidth }), className)}
      {...props}
    >
      {loading ? (
        <FaIcon icon={faSpinner} className="animate-spin" aria-hidden="true" />
      ) : null}
      {children}
    </button>
  );
}
