import * as TabsPrimitive from "@radix-ui/react-tabs";
import type { ReactNode } from "react";
import { cn } from "../../lib/utils";

interface Tab {
  value: string;
  label: string;
  icon?: ReactNode;
}

interface TabsProps {
  tabs: Tab[];
  defaultValue?: string;
  value?: string;
  onValueChange?: (value: string) => void;
  children: ReactNode;
  className?: string;
}

export default function Tabs({
  tabs,
  defaultValue,
  value,
  onValueChange,
  children,
  className,
}: TabsProps) {
  return (
    <TabsPrimitive.Root
      defaultValue={defaultValue}
      value={value}
      onValueChange={onValueChange}
      className={cn("flex flex-col", className)}
    >
      <TabsPrimitive.List className="flex border-b-2 border-[var(--theme-border-strong)] bg-[var(--theme-surface)]">
        {tabs.map((tab) => (
          <TabsPrimitive.Trigger
            key={tab.value}
            value={tab.value}
            className={cn(
              "relative flex items-center gap-2 border-r-2 border-[var(--theme-border-strong)] px-6 py-3 font-mono text-sm font-semibold uppercase tracking-wider text-[var(--theme-text-muted)] transition-colors",
              "hover:bg-[var(--theme-surface-hover)] hover:text-white",
              "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--theme-accent-lime)] focus-visible:ring-offset-2 focus-visible:ring-offset-black",
              "data-[state=active]:bg-black data-[state=active]:text-[var(--theme-accent-lime)] data-[state=active]:after:absolute data-[state=active]:after:bottom-0 data-[state=active]:after:left-0 data-[state=active]:after:right-0 data-[state=active]:after:h-[3px] data-[state=active]:after:bg-[var(--theme-accent-lime)]",
              "last:border-r-0"
            )}
          >
            {tab.icon && <span className="text-base">{tab.icon}</span>}
            {tab.label}
          </TabsPrimitive.Trigger>
        ))}
      </TabsPrimitive.List>
      {children}
    </TabsPrimitive.Root>
  );
}

export function TabsContent({
  value,
  children,
  className,
}: {
  value: string;
  children: ReactNode;
  className?: string;
}) {
  return (
    <TabsPrimitive.Content
      value={value}
      className={cn(
        "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--theme-accent-lime)]",
        className
      )}
    >
      {children}
    </TabsPrimitive.Content>
  );
}

export { TabsPrimitive };
