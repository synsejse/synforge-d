import { faXmark } from "@fortawesome/free-solid-svg-icons";
import type { ReactNode } from "react";
import FaIcon from "../ui/fa-icon";

interface Props {
  count: number;
  /** What's been selected — used in the summary copy ("N packages selected"). */
  noun?: { singular: string; plural: string };
  onClear: () => void;
  children: ReactNode;
}

/**
 * Sticky bottom-of-viewport bar that surfaces bulk actions when the user has
 * checked at least one row. Hidden when `count === 0`. Renders nothing on
 * its own — pass action buttons via `children`.
 */
export default function SelectionActionBar({
  count,
  noun = { singular: "item", plural: "items" },
  onClear,
  children,
}: Props) {
  if (count === 0) {
    return null;
  }

  const label = count === 1 ? noun.singular : noun.plural;

  return (
    <div
      role="region"
      aria-label="Bulk actions"
      className="sticky bottom-0 z-30 -mx-3 mt-6 border-t-4 border-accent-lime bg-black px-4 py-3 shadow-card-md sm:-mx-5 lg:-mx-8"
    >
      <div className="mx-auto flex max-w-[96rem] flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
        <div className="flex items-center gap-3">
          <span className="border-2 border-accent-lime bg-black px-3 py-1 font-mono text-xs font-bold uppercase tracking-[0.18em] text-accent-lime">
            {count}
          </span>
          <span className="font-mono text-sm uppercase tracking-[0.12em] text-strong">
            {label} selected
          </span>
          <button
            type="button"
            onClick={onClear}
            className="ml-1 inline-flex items-center gap-1 font-mono text-xs uppercase tracking-[0.15em] text-muted transition hover:text-white focus:outline-none focus:ring-2 focus:ring-accent-lime"
            aria-label="Clear selection"
          >
            <FaIcon icon={faXmark} />
            Clear
          </button>
        </div>
        <div className="flex flex-wrap items-center gap-2 sm:flex-nowrap sm:justify-end">
          {children}
        </div>
      </div>
    </div>
  );
}
