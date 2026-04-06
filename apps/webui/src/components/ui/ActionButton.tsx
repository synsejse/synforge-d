import type { IconDefinition } from "@fortawesome/fontawesome-svg-core";
import FaIcon from "./FaIcon";

interface CommonProps {
  icon?: IconDefinition;
  children: React.ReactNode;
  variant?: "default" | "primary";
  className?: string;
}

type AnchorProps = CommonProps &
  Omit<
    React.AnchorHTMLAttributes<HTMLAnchorElement>,
    "className" | "children"
  > & {
    href: string;
  };

type ButtonProps = CommonProps &
  Omit<
    React.ButtonHTMLAttributes<HTMLButtonElement>,
    "className" | "children"
  > & {
    href?: never;
  };

export default function ActionButton(props: AnchorProps | ButtonProps) {
  const variant = props.variant ?? "default";
  const classes = [
    "inline-flex min-w-[108px] items-center justify-center gap-2 border-2 px-3 py-2 font-mono text-xs font-semibold uppercase tracking-[0.12em] transition-all duration-100 ease-linear focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--theme-accent-lime)] focus-visible:ring-offset-2 focus-visible:ring-offset-black",
    variant === "primary"
      ? "border-[var(--theme-accent-lime)] bg-[var(--theme-accent-lime)] text-black shadow-[2px_2px_0_rgba(0,0,0,0.8)] hover:translate-x-[-1px] hover:translate-y-[-1px]"
      : "border-[var(--theme-border-strong)] bg-black text-zinc-200 hover:border-white hover:bg-zinc-950",
    props.className,
  ]
    .filter(Boolean)
    .join(" ");

  const content = (
    <>
      {props.icon ? (
        <FaIcon icon={props.icon} className="mr-2 text-[0.95em]" />
      ) : null}
      {props.children}
    </>
  );

  if ("href" in props && typeof props.href === "string") {
    const {
      icon: _icon,
      variant: _variant,
      className: _className,
      children,
      ...anchorProps
    } = props;
    return (
      <a {...anchorProps} className={classes}>
        {content}
      </a>
    );
  }

  const {
    icon: _icon,
    variant: _variant,
    className: _className,
    children,
    ...buttonProps
  } = props;
  return (
    <button {...buttonProps} className={classes}>
      {content}
    </button>
  );
}
