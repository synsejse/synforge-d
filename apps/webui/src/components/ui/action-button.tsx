import type { IconDefinition } from "@fortawesome/fontawesome-svg-core";
import { Link, type LinkProps } from "@tanstack/react-router";
import type { ButtonHTMLAttributes, ReactNode } from "react";
import FaIcon from "./fa-icon";

interface CommonProps {
  icon?: IconDefinition;
  children: ReactNode;
  variant?: "default" | "primary";
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

const baseClasses =
  "inline-flex min-w-[96px] items-center justify-center gap-2 border-2 px-3 py-2 font-mono text-xs font-semibold uppercase tracking-[0.12em] transition-all duration-100 ease-linear sm:min-w-[108px] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent-lime focus-visible:ring-offset-2 focus-visible:ring-offset-black";

function variantClasses(variant: CommonProps["variant"] = "default") {
  return variant === "primary"
    ? "border-accent-lime bg-accent-lime text-black shadow-brutal-sm hover:translate-x-[-1px] hover:translate-y-[-1px]"
    : "border-edge-strong bg-black text-strong hover:border-white hover:bg-surface-alt";
}

export default function ActionButton(props: LinkActionProps | ButtonActionProps) {
  const classes = [baseClasses, variantClasses(props.variant), props.className]
    .filter(Boolean)
    .join(" ");

  const content = (
    <>
      {props.icon ? <FaIcon icon={props.icon} className="mr-2 text-[0.95em]" /> : null}
      {props.children}
    </>
  );

  if ("to" in props && props.to !== undefined) {
    const { icon: _icon, variant: _variant, className: _className, children: _children, ...linkProps } = props;
    return (
      <Link {...linkProps} className={classes}>
        {content}
      </Link>
    );
  }

  const { icon: _icon, variant: _variant, className: _className, children: _children, ...buttonProps } = props;
  return (
    <button {...buttonProps} className={classes}>
      {content}
    </button>
  );
}
