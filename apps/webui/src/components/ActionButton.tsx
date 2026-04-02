import type { IconDefinition } from "@fortawesome/fontawesome-svg-core";
import FaIcon from "./FaIcon";

interface CommonProps {
  icon?: IconDefinition;
  children: React.ReactNode;
  variant?: "default" | "primary";
  className?: string;
}

type AnchorProps = CommonProps &
  Omit<React.AnchorHTMLAttributes<HTMLAnchorElement>, "className" | "children"> & {
    href: string;
  };

type ButtonProps = CommonProps &
  Omit<React.ButtonHTMLAttributes<HTMLButtonElement>, "className" | "children"> & {
    href?: never;
  };

export default function ActionButton(props: AnchorProps | ButtonProps) {
  const variant = props.variant ?? "default";
  const classes = [
    "inline-flex items-center border px-3 py-1.5 text-xs transition",
    variant === "primary"
      ? "border-zinc-200 bg-zinc-100 font-semibold text-black hover:bg-white"
      : "border-zinc-800 bg-black text-zinc-200 hover:border-zinc-600 hover:bg-zinc-950",
    props.className,
  ]
    .filter(Boolean)
    .join(" ");

  const content = (
    <>
      {props.icon ? <FaIcon icon={props.icon} className="mr-2 text-[0.95em]" /> : null}
      {props.children}
    </>
  );

  if ("href" in props && typeof props.href === "string") {
    const { icon: _icon, variant: _variant, className: _className, children, ...anchorProps } = props;
    return (
      <a {...anchorProps} className={classes}>
        {content}
      </a>
    );
  }

  const { icon: _icon, variant: _variant, className: _className, children, ...buttonProps } = props;
  return (
    <button {...buttonProps} className={classes}>
      {content}
    </button>
  );
}
