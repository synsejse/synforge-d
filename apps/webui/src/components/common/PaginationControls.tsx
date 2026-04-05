import type { ReactNode } from "react";

interface PaginationControlsProps {
  onPrevious: () => void;
  onNext: () => void;
  previousDisabled?: boolean;
  nextDisabled?: boolean;
  summary?: ReactNode;
}

export default function PaginationControls({
  onPrevious,
  onNext,
  previousDisabled = false,
  nextDisabled = false,
  summary,
}: PaginationControlsProps) {
  return (
    <div
      className={`flex items-center gap-3 ${summary ? "justify-between" : "justify-end"}`}
    >
      {summary ? <div className="text-sm text-zinc-400">{summary}</div> : null}
      <div className="flex items-center gap-3">
        <button
          type="button"
          onClick={onPrevious}
          disabled={previousDisabled}
          className="border border-zinc-800 bg-black px-4 py-2 text-sm text-zinc-300 transition hover:border-zinc-600 hover:bg-zinc-950 disabled:cursor-not-allowed disabled:opacity-50"
        >
          Previous
        </button>
        <button
          type="button"
          onClick={onNext}
          disabled={nextDisabled}
          className="border border-zinc-800 bg-black px-4 py-2 text-sm text-zinc-300 transition hover:border-zinc-600 hover:bg-zinc-950 disabled:cursor-not-allowed disabled:opacity-50"
        >
          Next
        </button>
      </div>
    </div>
  );
}
