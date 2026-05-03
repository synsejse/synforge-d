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
        // Hover lift + shadow are driven by the WRAPPER span's hover
        // (group-hover) so the cursor never lands in a "stuck off" zone
        // when the button translates away from the pointer. Cosmetic
        // hover (colour/border) stays on the button itself.
        primary: [
          "bg-accent-lime text-black border-accent-lime shadow-brutal-sm",
          "group-hover:shadow-brutal-lg group-hover:-translate-x-0.5 group-hover:-translate-y-0.5",
          "active:translate-x-0 active:translate-y-0 active:shadow-brutal-sm",
        ].join(" "),
        secondary: [
          "bg-white text-black border-white shadow-brutal-sm",
          "hover:bg-strong",
          "group-hover:shadow-brutal-lg group-hover:-translate-x-0.5 group-hover:-translate-y-0.5",
          "active:translate-x-0 active:translate-y-0 active:shadow-brutal-sm",
        ].join(" "),
        ghost: [
          "bg-transparent text-strong border-edge-strong shadow-brutal-sm",
          "hover:border-muted hover:bg-surface-hover hover:text-white",
          "group-hover:shadow-brutal-lg group-hover:-translate-x-0.5 group-hover:-translate-y-0.5",
          "active:translate-x-0 active:translate-y-0 active:shadow-brutal-sm",
        ].join(" "),
        subtle: [
          "bg-transparent text-soft border-transparent shadow-brutal-sm",
          "hover:bg-surface-hover hover:text-white",
          "group-hover:shadow-brutal-lg group-hover:-translate-x-0.5 group-hover:-translate-y-0.5",
          "active:translate-x-0 active:translate-y-0 active:shadow-brutal-sm",
        ].join(" "),
        danger: [
          "bg-error text-white border-error shadow-brutal-sm",
          "group-hover:shadow-brutal-lg group-hover:-translate-x-0.5 group-hover:-translate-y-0.5",
          "active:translate-x-0 active:translate-y-0 active:shadow-brutal-sm",
        ].join(" "),
        warning: [
          "bg-accent-orange text-black border-accent-orange shadow-brutal-sm",
          "group-hover:shadow-brutal-lg group-hover:-translate-x-0.5 group-hover:-translate-y-0.5",
          "active:translate-x-0 active:translate-y-0 active:shadow-brutal-sm",
        ].join(" "),
        terminal: [
          "bg-black text-success border-success font-mono shadow-brutal-sm",
          "hover:bg-success hover:text-black",
          "group-hover:shadow-brutal-lg group-hover:-translate-x-0.5 group-hover:-translate-y-0.5",
          "active:translate-x-0 active:translate-y-0 active:shadow-brutal-sm",
        ].join(" "),
      },
      size: {
        xs: "px-2 py-1 text-[10px] font-bold uppercase tracking-[0.18em] gap-1",
        sm: "px-3 py-1.5 text-xs font-bold uppercase tracking-[0.16em]",
        // md matches form-input height (px-4 py-2.5 text-sm = ~44px) so a
        // submit button in a filter row aligns with its sibling textboxes.
        md: "px-4 py-2.5 text-sm font-semibold",
        lg: "px-5 py-3 text-base font-bold",
        "icon-sm": "h-8 w-8 p-0",
        icon: "h-9 w-9 p-0",
        "icon-lg": "h-11 w-11 p-0",
      },
      fullWidth: {
        true: "w-full",
        // Full on mobile, auto from sm+ — for tight clusters of 1–2
        // buttons (modal footers, the sticky save bar) where the row
        // is short enough to fit at sm.
        responsive: "w-full sm:w-auto",
        // For detail-header toolbars (3–4 buttons) where wrapping at
        // sm/md would create unbalanced rows. Stays full-width until
        // lg so each stacked row is uniform; goes inline at lg+.
        "responsive-lg": "w-full lg:w-auto",
        false: "",
      },
    },
    compoundVariants: [
      // Bigger gap for larger sizes when icon + text are combined.
      { size: "lg", className: "gap-3" },
      // Icon-only buttons sit in tight per-row toolbars where the
      // offset shadow + lift becomes noise. Drop the brutal shadow
      // and the hover-translate; keep colour/border hover only.
      {
        size: ["icon-sm", "icon", "icon-lg"],
        className: [
          "shadow-none",
          "group-hover:shadow-none group-hover:translate-x-0 group-hover:translate-y-0",
          "active:shadow-none",
        ].join(" "),
      },
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

/**
 * Width handling for the wrapper span — mirrors the button's fullWidth
 * variant so the wrapper takes the same outer footprint as the button
 * used to. Without this, full-width buttons would shrink-wrap.
 */
function wrapperWidthClass(fullWidth: ButtonProps["fullWidth"]): string {
  switch (fullWidth) {
    case true:
      return "flex w-full";
    case "responsive":
      return "flex w-full sm:inline-flex sm:w-auto";
    case "responsive-lg":
      return "flex w-full lg:inline-flex lg:w-auto";
    default:
      return "inline-flex";
  }
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
  // Wrapper provides a stable hover hit-area: the button inside
  // translates on hover, but the wrapper's box doesn't move, so the
  // pointer never falls outside its hover region during the lift —
  // no more flicker at the seam. The ::before extends 6px past every
  // edge to absorb the shadow zone.
  return (
    <span
      className={cn(
        "group relative",
        wrapperWidthClass(fullWidth),
        "before:absolute before:-inset-1.5 before:content-['']",
      )}
    >
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
    </span>
  );
}
