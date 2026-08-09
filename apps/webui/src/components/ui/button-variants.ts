import { cva, type VariantProps } from "class-variance-authority";

/** Shared brutalist button variants and size scale. */
export const buttonVariants = cva(
  [
    "inline-flex items-center justify-center gap-2 border font-mono font-bold uppercase leading-none",
    "transition-[color,background-color,border-color,filter] duration-100 ease-linear",
    "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent-lime focus-visible:ring-offset-2 focus-visible:ring-offset-black",
    "disabled:pointer-events-none disabled:opacity-40",
  ].join(" "),
  {
    variants: {
      variant: {
        primary: "bg-accent-lime text-black border-accent-lime hover:brightness-110",
        secondary: "bg-white text-black border-white hover:bg-strong",
        ghost:
          "bg-transparent text-strong border-edge-strong hover:border-muted hover:bg-surface-hover hover:text-white",
        subtle:
          "bg-transparent text-soft border-transparent hover:bg-surface-hover hover:text-white",
        danger: "bg-error text-white border-error hover:brightness-110",
        warning:
          "bg-accent-orange text-black border-accent-orange hover:brightness-110",
        terminal:
          "bg-black text-success border-success hover:bg-success hover:text-black",
      },
      size: {
        xs: "min-h-8 px-2 py-1 text-[11px] tracking-[0.1em] gap-1.5",
        sm: "min-h-10 px-3 py-2 text-xs tracking-[0.08em]",
        md: "min-h-11 px-3.5 py-2.5 text-xs tracking-[0.08em]",
        lg: "min-h-12 px-5 py-3 text-sm font-extrabold tracking-[0.06em]",
        "icon-sm": "h-9 w-9 p-0",
        icon: "h-10 w-10 p-0",
        "icon-lg": "h-11 w-11 p-0",
      },
      fullWidth: {
        true: "w-full",
        responsive: "w-full sm:w-auto",
        "responsive-lg": "w-full lg:w-auto",
        false: "",
      },
    },
    compoundVariants: [{ size: "lg", className: "gap-3" }],
    defaultVariants: {
      variant: "ghost",
      size: "md",
      fullWidth: false,
    },
  },
);

export type ButtonVariantProps = VariantProps<typeof buttonVariants>;
