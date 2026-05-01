import { useState, type ReactNode } from "react";
import { faFilter, faXmark } from "@fortawesome/free-solid-svg-icons";
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

  const showClear = onClear && activeCount > 0;

  return (
    <div
      className={`border-2 border-white bg-black ${className}`.trim()}
    >
      {/* Mobile toggle row */}
      <div className="flex items-center justify-between gap-3 border-b-2 border-edge-strong bg-surface-alt px-4 py-3 md:hidden">
        <button
          type="button"
          onClick={() => setMobileOpen((open) => !open)}
          aria-expanded={mobileOpen}
          aria-controls="filter-bar-panel"
          className="flex items-center gap-2 font-mono text-xs font-bold uppercase tracking-[0.18em] text-white"
        >
          <FaIcon icon={faFilter} />
          Filters
          {activeCount > 0 ? (
            <span className="border-2 border-accent-lime bg-black px-2 py-0.5 text-[10px] text-accent-lime">
              {activeCount}
            </span>
          ) : null}
        </button>
        {showClear ? (
          <button
            type="button"
            onClick={onClear}
            className="flex items-center gap-1 font-mono text-xs uppercase tracking-[0.15em] text-muted transition hover:text-white focus:outline-none focus:ring-2 focus:ring-accent-lime"
          >
            <FaIcon icon={faXmark} />
            Clear
          </button>
        ) : null}
      </div>

      {/* Filter content: hidden on mobile until toggled, always shown on md:+ */}
      <div
        id="filter-bar-panel"
        className={`p-5 ${mobileOpen ? "" : "hidden"} md:block`}
      >
        {children}

        {(showClear || trailing) ? (
          <div className="mt-4 flex flex-wrap items-center justify-end gap-3 border-t-2 border-edge pt-4">
            {showClear ? (
              <button
                type="button"
                onClick={onClear}
                className="hidden md:flex items-center gap-2 border-2 border-edge-strong bg-black px-3 py-1.5 font-mono text-xs font-bold uppercase tracking-[0.15em] text-muted transition hover:border-white hover:text-white focus:outline-none focus:ring-2 focus:ring-accent-lime"
              >
                <FaIcon icon={faXmark} />
                Clear filters
              </button>
            ) : null}
            {trailing}
          </div>
        ) : null}
      </div>
    </div>
  );
}
