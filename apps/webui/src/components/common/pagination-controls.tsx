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
      {summary ? <div className="font-mono text-xs uppercase tracking-[0.12em] text-muted">{summary}</div> : null}
      <div className="flex items-center gap-3">
        <button
          type="button"
          onClick={onPrevious}
          disabled={previousDisabled}
          className="border-2 border-edge-strong bg-black px-4 py-2 font-mono text-xs font-bold uppercase tracking-[0.12em] text-muted transition duration-100 ease-linear hover:-translate-x-[1px] hover:-translate-y-[1px] hover:border-white hover:bg-surface-alt disabled:cursor-not-allowed disabled:opacity-50"
        >
          Previous
        </button>
        <button
          type="button"
          onClick={onNext}
          disabled={nextDisabled}
          className="border-2 border-edge-strong bg-black px-4 py-2 font-mono text-xs font-bold uppercase tracking-[0.12em] text-muted transition duration-100 ease-linear hover:-translate-x-[1px] hover:-translate-y-[1px] hover:border-white hover:bg-surface-alt disabled:cursor-not-allowed disabled:opacity-50"
        >
          Next
        </button>
      </div>
    </div>
  );
}
