import { useEffect, useId, useState, type ReactNode } from "react";
import { faFilter, faXmark } from "@fortawesome/free-solid-svg-icons";
import Button from "../ui/button";
import FaIcon from "../ui/fa-icon";

interface FilterBarProps {
  /** Renders the actual filter inputs. Caller controls layout (grid, etc.) */
  children: ReactNode;
  /**
   * Number of currently-active filters. Drives the "Filters (3)" badge on the
   * mobile toggle and the visibility of the Clear button.
   */
  activeCount?: number;
  /** Optional clear-all handler. When provided and activeCount > 0, shows a Clear button. */
  onClear?: () => void;
  /** Optional extra elements (e.g. a primary "Apply" button) to render in the trailing slot. */
  trailing?: ReactNode;
  className?: string;
}

/**
 * Wraps a list-page filter form. Inline on md:+; collapsible behind a
 * "Filters (N)" toggle on mobile.
 */
export default function FilterBar({
  children,
  activeCount = 0,
  onClear,
  trailing,
  className = "",
}: FilterBarProps) {
  const [mobileOpen, setMobileOpen] = useState(false);
  const panelId = useId();

  useEffect(() => {
    if (activeCount > 0) setMobileOpen(true);
  }, [activeCount]);

  const showClear = onClear && activeCount > 0;
  const clearFilters = () => {
    onClear?.();
    setMobileOpen(false);
  };

  return (
    <div
      className={`border border-edge-strong bg-black ${className}`.trim()}
    >
      <div className="flex items-center justify-between gap-3 border-b border-edge bg-surface-alt px-4 py-3 md:hidden">
        <button
          type="button"
          onClick={() => setMobileOpen((open) => !open)}
          aria-expanded={mobileOpen}
          aria-controls={panelId}
          className="flex min-h-10 items-center gap-2 font-mono text-xs font-bold uppercase tracking-[0.18em] text-white"
        >
          <FaIcon icon={faFilter} />
          Filters
          {activeCount > 0 ? (
            <span className="border border-accent-lime bg-black px-2 py-0.5 text-xs text-accent-lime">
              {activeCount}
            </span>
          ) : null}
        </button>
        {showClear ? (
          <Button
            variant="subtle"
            size="xs"
            onClick={clearFilters}
            className="min-h-10"
          >
            <FaIcon icon={faXmark} />
            Clear
          </Button>
        ) : null}
      </div>

      {/* Filter content: hidden on mobile until toggled, always shown on md:+ */}
      <div
        id={panelId}
        className={`p-5 ${mobileOpen ? "" : "hidden"} md:block`}
      >
        {children}

        {(showClear || trailing) ? (
          <div className="mt-4 flex flex-wrap items-center justify-end gap-3 border-t border-edge pt-4">
            {showClear ? (
              <Button
                variant="ghost"
                size="sm"
                onClick={clearFilters}
                className="hidden md:inline-flex"
              >
                <FaIcon icon={faXmark} />
                Clear filters
              </Button>
            ) : null}
            {trailing}
          </div>
        ) : null}
      </div>
    </div>
  );
}
