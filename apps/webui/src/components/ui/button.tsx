import type { ButtonHTMLAttributes, ReactNode } from "react";
import { faSpinner } from "@fortawesome/free-solid-svg-icons";
import { cn } from "../../lib/utils";
import FaIcon from "./fa-icon";
import {
  buttonVariants,
  type ButtonVariantProps,
} from "./button-variants";

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
