import type { IconDefinition } from "@fortawesome/fontawesome-svg-core";
import type { ReactNode } from "react";
import FaIcon from "./fa-icon";

interface Props {
  icon?: IconDefinition;
  title?: string;
  description?: string;
  action?: ReactNode;
  /** Free-form fallback. Used when title/description are not provided. */
  children?: ReactNode;
  className?: string;
}

export default function EmptyState({
  icon,
  title,
  description,
  action,
  children,
  className,
}: Props) {
  const wrapperClass = `border-2 border-dashed border-[var(--theme-border-strong)] bg-black p-8 text-center font-mono text-sm text-[var(--theme-text-muted)] ${className ?? ""}`;

  const hasStructuredContent = Boolean(title || description || icon || action);

  if (!hasStructuredContent) {
    return <div className={wrapperClass}>{children}</div>;
  }

  return (
    <div className={wrapperClass}>
      {icon ? (
        <FaIcon
          icon={icon}
          className="mb-4 text-3xl text-[var(--theme-text-soft)]"
        />
      ) : null}
      {title ? (
        <div className="font-display text-base font-bold uppercase tracking-[0.15em] text-white">
          {title}
        </div>
      ) : null}
      {description ? (
        <p className="mx-auto mt-2 max-w-md text-sm text-[var(--theme-text-muted)]">
          {description}
        </p>
      ) : null}
      {children ? <div className="mt-3">{children}</div> : null}
      {action ? <div className="mt-5 flex justify-center">{action}</div> : null}
    </div>
  );
}
