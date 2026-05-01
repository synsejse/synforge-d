import { Link, type LinkProps } from "@tanstack/react-router";
import type { ReactNode } from "react";

export interface BreadcrumbItem {
  label: string;
  to?: LinkProps["to"];
  search?: LinkProps["search"];
}

interface Props {
  items: BreadcrumbItem[];
  className?: string;
  trailing?: ReactNode;
}

export default function Breadcrumbs({ items, className = "", trailing }: Props) {
  return (
    <nav
      aria-label="Breadcrumb"
      className={`flex flex-wrap items-center gap-2 font-mono text-xs uppercase tracking-[0.18em] text-soft ${className}`}
    >
      {items.map((item, index) => {
        const isLast = index === items.length - 1;
        const separator = index > 0 ? (
          <span aria-hidden="true" className="text-edge-strong">
            /
          </span>
        ) : null;

        if (isLast || !item.to) {
          return (
            <span key={`${item.label}-${index}`} className="contents">
              {separator}
              <span
                className={
                  isLast
                    ? "text-strong"
                    : "text-muted"
                }
                aria-current={isLast ? "page" : undefined}
              >
                {item.label}
              </span>
            </span>
          );
        }

        return (
          <span key={`${item.label}-${index}`} className="contents">
            {separator}
            <Link
              to={item.to}
              search={item.search}
              className="text-muted underline-offset-2 transition-colors hover:text-accent-lime hover:underline"
            >
              {item.label}
            </Link>
          </span>
        );
      })}
      {trailing ? (
        <span className="ml-2 text-muted">{trailing}</span>
      ) : null}
    </nav>
  );
}
