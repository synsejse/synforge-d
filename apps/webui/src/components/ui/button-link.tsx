import { Link, type LinkProps } from "@tanstack/react-router";
import type { IconDefinition } from "@fortawesome/fontawesome-svg-core";
import type { AnchorHTMLAttributes, ReactNode } from "react";
import FaIcon from "./fa-icon";
import { buttonVariants, type ButtonVariantProps } from "./button";
import { cn } from "../../lib/utils";

interface ButtonLinkOwnProps extends ButtonVariantProps {
  iconLeft?: IconDefinition;
  iconRight?: IconDefinition;
  className?: string;
  children?: ReactNode;
}

type RouterLinkProps = ButtonLinkOwnProps & Omit<LinkProps, "className" | "children">;

type ExternalLinkProps = ButtonLinkOwnProps &
  Omit<AnchorHTMLAttributes<HTMLAnchorElement>, "className" | "children"> & {
    href: string;
    /** Force a plain `<a>` (e.g. for non-SPA targets like /docs Swagger UI). */
    external: true;
  };

export type ButtonLinkProps = RouterLinkProps | ExternalLinkProps;

/**
 * Renders a styled link that visually matches `<Button>`.
 *
 * Use this for navigation actions ("Open detail", "Back to list"); use
 * `<Button>` for in-place actions (submit, delete, toggle).
 */
export default function ButtonLink(props: ButtonLinkProps) {
  if ("external" in props && props.external) {
    const {
      external: _external,
      variant,
      size,
      fullWidth,
      iconLeft,
      iconRight,
      className,
      children,
      ...rest
    } = props;
    return (
      <a
        {...rest}
        className={cn(buttonVariants({ variant, size, fullWidth }), className)}
      >
        <ButtonLinkBody
          iconLeft={iconLeft}
          iconRight={iconRight}
          isIconOnly={isIconSize(size)}
        >
          {children}
        </ButtonLinkBody>
      </a>
    );
  }

  const {
    variant,
    size,
    fullWidth,
    iconLeft,
    iconRight,
    className,
    children,
    ...linkProps
  } = props as RouterLinkProps;

  return (
    <Link
      {...linkProps}
      className={cn(buttonVariants({ variant, size, fullWidth }), className)}
    >
      <ButtonLinkBody
        iconLeft={iconLeft}
        iconRight={iconRight}
        isIconOnly={isIconSize(size)}
      >
        {children}
      </ButtonLinkBody>
    </Link>
  );
}

function isIconSize(size: ButtonVariantProps["size"]): boolean {
  return size === "icon" || size === "icon-sm" || size === "icon-lg";
}

function ButtonLinkBody({
  iconLeft,
  iconRight,
  isIconOnly,
  children,
}: {
  iconLeft?: IconDefinition;
  iconRight?: IconDefinition;
  isIconOnly: boolean;
  children?: ReactNode;
}) {
  return (
    <>
      {iconLeft ? <FaIcon icon={iconLeft} aria-hidden="true" /> : null}
      {!isIconOnly ? children : iconLeft ? null : children}
      {iconRight ? <FaIcon icon={iconRight} aria-hidden="true" /> : null}
    </>
  );
}
