import { cn } from "../../lib/utils";

interface SkeletonProps {
  className?: string;
  width?: string | number;
  height?: string | number;
}

const baseClass =
  "animate-pulse border-2 border-edge-strong bg-surface-alt";

export function Skeleton({ className, width, height }: SkeletonProps) {
  const style: React.CSSProperties = {};
  if (width !== undefined) style.width = typeof width === "number" ? `${width}px` : width;
  if (height !== undefined) style.height = typeof height === "number" ? `${height}px` : height;
  return <div className={cn(baseClass, className)} style={style} />;
}

interface SkeletonRowProps {
  columns: number;
  className?: string;
}

export function SkeletonRow({ columns, className }: SkeletonRowProps) {
  return (
    <div className={cn("flex items-center gap-4", className)}>
      {Array.from({ length: columns }).map((_, index) => (
        <Skeleton key={index} className="h-4 flex-1" />
      ))}
    </div>
  );
}

interface SkeletonTableProps {
  columns: number;
  rows?: number;
  className?: string;
}

export function SkeletonTable({ columns, rows = 5, className }: SkeletonTableProps) {
  return (
    <div
      role="status"
      aria-live="polite"
      className={cn(
        "border-2 border-edge-strong bg-black p-4 space-y-3",
        className,
      )}
    >
      <span className="sr-only">Loading…</span>
      {Array.from({ length: rows }).map((_, index) => (
        <SkeletonRow key={index} columns={columns} />
      ))}
    </div>
  );
}

interface SkeletonCardProps {
  lines?: number;
  className?: string;
}

export function SkeletonCard({ lines = 3, className }: SkeletonCardProps) {
  return (
    <div
      role="status"
      aria-live="polite"
      className={cn(
        "border-2 border-edge-strong bg-black p-5 space-y-3",
        className,
      )}
    >
      <span className="sr-only">Loading…</span>
      <Skeleton className="h-5 w-1/3" />
      {Array.from({ length: lines }).map((_, index) => (
        <Skeleton
          key={index}
          className="h-3"
          width={`${Math.max(40, 100 - index * 12)}%`}
        />
      ))}
    </div>
  );
}

interface SkeletonCardListProps {
  /** Number of cards in the list. */
  count?: number;
  /** Lines inside each card body. */
  lines?: number;
  className?: string;
}

/**
 * Stack of card-shaped skeletons mimicking pages that render a vertical
 * list of cards (jobs list, package list, repo files, users).
 */
export function SkeletonCardList({
  count = 4,
  lines = 2,
  className,
}: SkeletonCardListProps) {
  return (
    <div
      role="status"
      aria-live="polite"
      className={cn("space-y-3", className)}
    >
      <span className="sr-only">Loading…</span>
      {Array.from({ length: count }).map((_, index) => (
        <SkeletonListRow key={index} />
      ))}
    </div>
  );
}

/**
 * Skeleton variant matching the brutalist card chrome used across
 * jobs / packages / users / repo file lists: bordered box with the
 * 1px status rail on the left, a header line + chips, and a meta row.
 * Renders in place of a real card so the page layout is fixed before
 * data lands.
 */
export function SkeletonListRow({ className }: { className?: string }) {
  return (
    <div
      className={cn(
        "relative border-2 border-edge-strong bg-black",
        className,
      )}
    >
      <span
        aria-hidden="true"
        className="absolute inset-y-0 left-0 w-1 bg-edge-strong"
      />
      <div className="flex flex-col gap-3 pl-4 pr-4 py-3 sm:pl-6 sm:pr-5 sm:py-4 lg:flex-row lg:items-start lg:gap-4">
        <div className="min-w-0 flex-1 space-y-3">
          <div className="flex flex-wrap items-center gap-2">
            <Skeleton className="h-5 w-40" />
            <Skeleton className="h-5 w-16" />
            <Skeleton className="h-5 w-20" />
          </div>
          <div className="flex flex-wrap items-center gap-x-6 gap-y-2">
            <Skeleton className="h-3 w-24" />
            <Skeleton className="h-3 w-32" />
            <Skeleton className="h-3 w-28" />
            <Skeleton className="h-3 w-36" />
          </div>
        </div>
        <div className="flex shrink-0 gap-1">
          <Skeleton className="h-8 w-8" />
          <Skeleton className="h-8 w-8" />
        </div>
      </div>
    </div>
  );
}

interface SkeletonMetricGridProps {
  /** Number of metric cells. Defaults to 4 to match the dashboard / repo
   *  summary grids. */
  count?: number;
  className?: string;
}

/**
 * Grid of MetricCard-shaped skeletons. Sized to match the real
 * MetricCard footprint so the layout doesn't reflow when data lands.
 */
export function SkeletonMetricGrid({
  count = 4,
  className,
}: SkeletonMetricGridProps) {
  return (
    <div
      role="status"
      aria-live="polite"
      className={cn(
        "grid gap-4 md:grid-cols-2 xl:grid-cols-4",
        className,
      )}
    >
      <span className="sr-only">Loading…</span>
      {Array.from({ length: count }).map((_, index) => (
        <div
          key={index}
          className="flex min-h-[10rem] flex-col justify-between border-4 border-edge-strong bg-black p-6"
        >
          <div className="space-y-3">
            <Skeleton className="h-3 w-1/2" />
            <Skeleton className="h-9 w-2/3" />
          </div>
          <Skeleton className="h-3 w-1/3" />
        </div>
      ))}
    </div>
  );
}

interface SkeletonFormProps {
  /** Number of disclosure-style sections to mock. */
  sections?: number;
  /** Fields per section. */
  fieldsPerSection?: number;
  className?: string;
}

/**
 * Stack of section blocks each with a header + 2-column field grid.
 * Mimics the settings / package-edit / repo-setup loading shape.
 */
export function SkeletonForm({
  sections = 3,
  fieldsPerSection = 4,
  className,
}: SkeletonFormProps) {
  return (
    <div
      role="status"
      aria-live="polite"
      className={cn("space-y-4", className)}
    >
      <span className="sr-only">Loading…</span>
      {Array.from({ length: sections }).map((_, sIndex) => (
        <div
          key={sIndex}
          className="border-2 border-edge-strong bg-black p-5 space-y-4"
        >
          <Skeleton className="h-4 w-1/4" />
          <div className="grid gap-4 md:grid-cols-2">
            {Array.from({ length: fieldsPerSection }).map((_, fIndex) => (
              <div key={fIndex} className="space-y-2">
                <Skeleton className="h-3 w-1/3" />
                <Skeleton className="h-10 w-full" />
              </div>
            ))}
          </div>
        </div>
      ))}
    </div>
  );
}
