import { useState, type ReactNode } from "react";
import { faFilter, faXmark } from "@fortawesome/free-solid-svg-icons";
import Button from "../ui/button";
import FaIcon from "../ui/fa-icon";

interface FilterBarProps {
  children: ReactNode;
  activeCount?: number;
  onClear?: () => void;
  trailing?: ReactNode;
  className?: string;
}

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
          <Button variant="subtle" size="xs" onClick={onClear}>
            <FaIcon icon={faXmark} />
            Clear
          </Button>
        ) : null}
      </div>

      <div
        id="filter-bar-panel"
        className={`p-5 ${mobileOpen ? "" : "hidden"} md:block`}
      >
        {children}

        {(showClear || trailing) ? (
          <div className="mt-4 flex flex-wrap items-center justify-end gap-3 border-t-2 border-edge pt-4">
            {showClear ? (
              <Button
                variant="ghost"
                size="sm"
                onClick={onClear}
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
