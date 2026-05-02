import type { ReactNode } from "react";
import { faAngleLeft, faAngleRight } from "@fortawesome/free-solid-svg-icons";
import Button from "../ui/button";
import FaIcon from "../ui/fa-icon";

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
      {summary ? (
        <div className="font-mono text-xs uppercase tracking-[0.12em] text-muted">
          {summary}
        </div>
      ) : null}
      <div className="flex items-center gap-2">
        <Button
          variant="ghost"
          size="sm"
          onClick={onPrevious}
          disabled={previousDisabled}
        >
          <FaIcon icon={faAngleLeft} />
          Previous
        </Button>
        <Button
          variant="ghost"
          size="sm"
          onClick={onNext}
          disabled={nextDisabled}
        >
          Next
          <FaIcon icon={faAngleRight} />
        </Button>
      </div>
    </div>
  );
}
