import { faAngleLeft, faAngleRight } from "@fortawesome/free-solid-svg-icons";
import Button from "../ui/button";
import FaIcon from "../ui/fa-icon";

interface PaginationControlsProps {
  /** Zero-based offset of the current page. */
  offset: number;
  /** Page size used to step prev/next. */
  pageSize: number;
  /** Number of items on the current page. */
  count: number;
  /** Whether a next page exists. */
  hasMore: boolean;
  /** Total item count when known; enables the "of N" suffix. */
  total?: number | null;
  /** Disables both buttons while a fetch is in flight. */
  isFetching?: boolean;
  /** Called with the new offset when Previous/Next is clicked. */
  onOffsetChange: (offset: number) => void;
  /** Overrides the default footer wrapper styling. */
  className?: string;
}

const DEFAULT_WRAPPER = "border-2 border-edge-strong bg-black px-4 py-3";

export default function PaginationControls({
  offset,
  pageSize,
  count,
  hasMore,
  total,
  isFetching = false,
  onOffsetChange,
  className,
}: PaginationControlsProps) {
  const start = count === 0 ? 0 : offset + 1;
  const end = offset + count;
  const summary =
    total != null ? `Showing ${start}–${end} of ${total}` : `Showing ${start}–${end}`;

  return (
    <div className={className ?? DEFAULT_WRAPPER}>
      <div className="flex items-center justify-between gap-3">
        <div className="font-mono text-xs uppercase tracking-[0.12em] text-muted">{summary}</div>
        <div className="flex items-center gap-2">
          <Button variant="ghost" size="sm" onClick={() => onOffsetChange(Math.max(0, offset - pageSize))} disabled={isFetching || offset === 0}>
            <FaIcon icon={faAngleLeft} />
            Previous
          </Button>
          <Button variant="ghost" size="sm" onClick={() => onOffsetChange(offset + pageSize)} disabled={isFetching || !hasMore}>
            Next
            <FaIcon icon={faAngleRight} />
          </Button>
        </div>
      </div>
    </div>
  );
}
