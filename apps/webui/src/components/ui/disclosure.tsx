import * as AccordionPrimitive from "@radix-ui/react-accordion";
import { faChevronDown } from "@fortawesome/free-solid-svg-icons";
import type { ReactNode } from "react";
import { cn } from "../../lib/utils";
import FaIcon from "./fa-icon";

interface DisclosureGroupProps {
  /**
   * If "single", only one item open at a time. If "multiple", any subset.
   * Default "multiple" — long detail pages should let users open several
   * sections without forcing them to dismiss others.
   */
  type?: "single" | "multiple";
  /** Item value(s) open by default. */
  defaultValue?: string | string[];
  children: ReactNode;
  className?: string;
}

export function DisclosureGroup({
  type = "multiple",
  defaultValue,
  children,
  className,
}: DisclosureGroupProps) {
  if (type === "single") {
    return (
      <AccordionPrimitive.Root
        type="single"
        collapsible
        defaultValue={defaultValue as string | undefined}
        className={cn("space-y-3", className)}
      >
        {children}
      </AccordionPrimitive.Root>
    );
  }
  return (
    <AccordionPrimitive.Root
      type="multiple"
      defaultValue={defaultValue as string[] | undefined}
      className={cn("space-y-3", className)}
    >
      {children}
    </AccordionPrimitive.Root>
  );
}

interface DisclosureProps {
  value: string;
  title: string;
  description?: string;
  trailing?: ReactNode;
  children: ReactNode;
  className?: string;
}

export function Disclosure({
  value,
  title,
  description,
  trailing,
  children,
  className,
}: DisclosureProps) {
  return (
    <AccordionPrimitive.Item
      value={value}
      className={cn(
        "border-2 border-[var(--theme-border-strong)] bg-black",
        className,
      )}
    >
      <AccordionPrimitive.Header className="flex">
        <AccordionPrimitive.Trigger
          className={cn(
            "group flex w-full items-center justify-between gap-3 px-5 py-4 text-left transition-colors",
            "hover:bg-[var(--theme-surface-alt)]",
            "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--theme-accent-lime)] focus-visible:ring-offset-2 focus-visible:ring-offset-black",
            "data-[state=open]:border-b-2 data-[state=open]:border-[var(--theme-border-strong)]",
          )}
        >
          <div className="min-w-0 flex-1">
            <div className="font-mono text-sm font-bold uppercase tracking-[0.18em] text-white">
              {title}
            </div>
            {description ? (
              <p className="mt-1 font-mono text-xs text-[var(--theme-text-muted)]">
                {description}
              </p>
            ) : null}
          </div>
          {trailing ? (
            <div className="flex shrink-0 items-center gap-2" onClick={(e) => e.stopPropagation()}>
              {trailing}
            </div>
          ) : null}
          <FaIcon
            icon={faChevronDown}
            className="shrink-0 text-[var(--theme-text-muted)] transition-transform duration-200 group-data-[state=open]:rotate-180"
          />
        </AccordionPrimitive.Trigger>
      </AccordionPrimitive.Header>
      <AccordionPrimitive.Content
        className={cn(
          "overflow-hidden",
          "data-[state=open]:animate-in data-[state=closed]:animate-out",
          "data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0",
        )}
      >
        <div className="px-5 py-4">{children}</div>
      </AccordionPrimitive.Content>
    </AccordionPrimitive.Item>
  );
}
