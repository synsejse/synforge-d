import { Link, type LinkProps } from "@tanstack/react-router";
import type { AnchorHTMLAttributes, ReactNode } from "react";
import { buttonVariants, type ButtonVariantProps } from "./button";
import { cn } from "../../lib/utils";

interface ButtonLinkOwnProps extends ButtonVariantProps {
  className?: string;
  children?: ReactNode;
}

type RouterLinkProps = ButtonLinkOwnProps & Omit<LinkProps, "className" | "children">;

type ExternalLinkProps = ButtonLinkOwnProps &
  Omit<AnchorHTMLAttributes<HTMLAnchorElement>, "className" | "children"> & {
    href: string;
    external: true;
  };

export type ButtonLinkProps = RouterLinkProps | ExternalLinkProps;

/** Use for navigation actions; use `<Button>` for in-place actions. */
export default function ButtonLink(props: ButtonLinkProps) {
  if ("external" in props && props.external) {
    const {
      external: _external,
      variant,
      size,
      fullWidth,
      className,
      children,
      ...rest
    } = props;
    return (
      <a
        {...rest}
        className={cn(buttonVariants({ variant, size, fullWidth }), className)}
      >
        {children}
      </a>
    );
  }

  const {
    variant,
    size,
    fullWidth,
    className,
    children,
    ...linkProps
  } = props as RouterLinkProps;

  return (
    <Link
      {...linkProps}
      className={cn(buttonVariants({ variant, size, fullWidth }), className)}
    >
      {children}
    </Link>
  );
}
