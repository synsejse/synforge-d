import type { IconDefinition } from "@fortawesome/fontawesome-svg-core";
import type { LinkProps } from "@tanstack/react-router";
import type { ButtonHTMLAttributes, ReactNode } from "react";
import Button from "./button";
import ButtonLink from "./button-link";

interface CommonProps {
  icon?: IconDefinition;
  children: ReactNode;
  variant?: "default" | "primary" | "danger";
  className?: string;
}

type LinkActionProps = CommonProps &
  Omit<LinkProps, "children" | "className"> & {
    to: LinkProps["to"];
    onClick?: never;
  };

type ButtonActionProps = CommonProps &
  Omit<ButtonHTMLAttributes<HTMLButtonElement>, "children" | "className"> & {
    to?: never;
  };

const variantMap = {
  default: "ghost",
  primary: "primary",
  danger: "danger",
} as const;

/**
 * Thin compatibility shim: existing callers keep using `<ActionButton>`,
 * but rendering now delegates to the unified `<Button>` / `<ButtonLink>`
 * primitives so visual updates flow through one place.
 *
 * Prefer `<Button>` or `<ButtonLink>` directly in new code.
 */
export default function ActionButton(
  props: LinkActionProps | ButtonActionProps,
) {
  const variant = variantMap[props.variant ?? "default"];

  if ("to" in props && props.to !== undefined) {
    const {
      icon,
      variant: _variant,
      className,
      children,
      ...linkProps
    } = props;
    return (
      <ButtonLink
        {...linkProps}
        variant={variant}
        size="sm"
        iconLeft={icon}
        className={className}
      >
        {children}
      </ButtonLink>
    );
  }

  const {
    icon,
    variant: _variant,
    className,
    children,
    to: _to,
    ...buttonProps
  } = props;
  return (
    <Button
      {...buttonProps}
      variant={variant}
      size="sm"
      iconLeft={icon}
      className={className}
    >
      {children}
    </Button>
  );
}
