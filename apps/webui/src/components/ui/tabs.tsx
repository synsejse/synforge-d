import { useId, type KeyboardEvent, type ReactNode } from "react";
import { cn } from "../../lib/utils";

export interface TabItem<TValue extends string> {
  value: TValue;
  label: string;
  /** Optional count badge ("Builds 42"). */
  count?: number | null;
  disabled?: boolean;
}

interface TabsProps<TValue extends string> {
  items: ReadonlyArray<TabItem<TValue>>;
  value: TValue;
  onChange: (next: TValue) => void;
  ariaLabel: string;
  /** The active tab's panel content. */
  children: ReactNode;
  className?: string;
}

/**
 * Brutalist tab strip with proper ARIA roles + roving keyboard nav
 * (Left/Right + Home/End cycle through tabs). Only the active panel renders.
 */
export default function Tabs<TValue extends string>({
  items,
  value,
  onChange,
  ariaLabel,
  children,
  className,
}: TabsProps<TValue>) {
  const baseId = useId();

  function focusTabAt(idx: number) {
    const wrap = (idx + items.length) % items.length;
    const target = items[wrap];
    if (target && !target.disabled) {
      onChange(target.value);
      const button = document.getElementById(tabButtonId(baseId, target.value));
      button?.focus();
    }
  }

  function handleKeyDown(event: KeyboardEvent<HTMLDivElement>) {
    const currentIdx = items.findIndex((item) => item.value === value);
    if (currentIdx < 0) return;
    if (event.key === "ArrowRight") {
      event.preventDefault();
      focusTabAt(currentIdx + 1);
    } else if (event.key === "ArrowLeft") {
      event.preventDefault();
      focusTabAt(currentIdx - 1);
    } else if (event.key === "Home") {
      event.preventDefault();
      focusTabAt(0);
    } else if (event.key === "End") {
      event.preventDefault();
      focusTabAt(items.length - 1);
    }
  }

  return (
    <div className={cn("min-w-0", className)}>
      <div
        role="tablist"
        aria-label={ariaLabel}
        onKeyDown={handleKeyDown}
        className="flex flex-wrap items-stretch border border-edge border-b-0 bg-surface-alt"
      >
        {items.map((item, idx) => {
          const isActive = item.value === value;
          return (
            <button
              key={item.value}
              id={tabButtonId(baseId, item.value)}
              type="button"
              role="tab"
              aria-selected={isActive}
              aria-controls={tabPanelId(baseId, item.value)}
              tabIndex={isActive ? 0 : -1}
              disabled={item.disabled}
              onClick={() => onChange(item.value)}
              className={cn(
                "shrink-0 whitespace-nowrap px-4 py-2.5 font-mono text-xs font-bold uppercase tracking-[0.16em] transition-colors",
                "focus-visible:outline-none focus-visible:relative focus-visible:z-10 focus-visible:ring-2 focus-visible:ring-accent-lime focus-visible:ring-offset-2 focus-visible:ring-offset-black",
                "disabled:pointer-events-none disabled:opacity-40",
                idx > 0 ? "border-l border-edge-strong" : "",
                isActive
                  ? "bg-black text-accent-lime border-b border-accent-lime -mb-[2px]"
                  : "bg-surface-alt text-soft hover:bg-black hover:text-white",
              )}
            >
              <span>{item.label}</span>
              {typeof item.count === "number" ? (
                <span
                  className={cn(
                    "ml-2 inline-block min-w-[1.5rem] px-1 py-0.5 text-xs tracking-[0.12em]",
                    isActive
                      ? "border border-accent-lime text-accent-lime"
                      : "border border-edge text-soft",
                  )}
                >
                  {item.count}
                </span>
              ) : null}
            </button>
          );
        })}
      </div>
      <div
        role="tabpanel"
        id={tabPanelId(baseId, value)}
        aria-labelledby={tabButtonId(baseId, value)}
        className="border border-edge bg-black p-4 sm:p-5"
      >
        {children}
      </div>
    </div>
  );
}

function tabButtonId(base: string, value: string) {
  return `${base}-tab-${value}`;
}

function tabPanelId(base: string, value: string) {
  return `${base}-panel-${value}`;
}
