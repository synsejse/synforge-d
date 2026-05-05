import type { IconDefinition } from "@fortawesome/fontawesome-svg-core";
import type { ReactNode } from "react";
import FaIcon from "./fa-icon";

interface Props {
  icon?: IconDefinition;
  title?: string;
  description?: string;
  action?: ReactNode;
  children?: ReactNode;
  className?: string;
  hint?: string;
}

export default function EmptyState({
  icon,
  title,
  description,
  action,
  children,
  className,
  hint,
}: Props) {
  const wrapperClass = `relative border-2 border-dashed border-edge-strong bg-black p-8 text-center font-mono text-sm text-muted ${className ?? ""}`;

  const hasStructuredContent = Boolean(
    title || description || icon || action || hint,
  );

  if (!hasStructuredContent) {
    return (
      <div className={wrapperClass}>
        <CornerBrackets />
        {children}
      </div>
    );
  }

  return (
    <div className={wrapperClass}>
      <CornerBrackets />
      {icon ? (
        <FaIcon icon={icon} className="mb-4 text-3xl text-soft" />
      ) : null}
      {title ? (
        <div className="font-display text-base font-bold uppercase tracking-[0.15em] text-white">
          {title}
        </div>
      ) : null}
      {description ? (
        <p className="mx-auto mt-2 max-w-md text-sm text-muted">
          {description}
        </p>
      ) : null}
      {children ? <div className="mt-3">{children}</div> : null}
      {hint ? (
        <p className="mx-auto mt-4 inline-flex items-center gap-1 max-w-md font-mono text-xs uppercase tracking-[0.16em] text-soft">
          <span className="text-accent-lime">{">"}</span>
          <span>{hint}</span>
          <span
            aria-hidden="true"
            className="inline-block h-3 w-[2px] animate-pulse bg-accent-lime"
          />
        </p>
      ) : null}
      {action ? <div className="mt-5 flex justify-center">{action}</div> : null}
    </div>
  );
}

function CornerBrackets() {
  return (
    <>
      <span
        aria-hidden="true"
        className="pointer-events-none absolute left-2 top-2 h-3 w-3 border-l-2 border-t-2 border-accent-lime/60"
      />
      <span
        aria-hidden="true"
        className="pointer-events-none absolute right-2 top-2 h-3 w-3 border-r-2 border-t-2 border-accent-lime/60"
      />
      <span
        aria-hidden="true"
        className="pointer-events-none absolute bottom-2 left-2 h-3 w-3 border-b-2 border-l-2 border-accent-lime/60"
      />
      <span
        aria-hidden="true"
        className="pointer-events-none absolute bottom-2 right-2 h-3 w-3 border-b-2 border-r-2 border-accent-lime/60"
      />
    </>
  );
}
