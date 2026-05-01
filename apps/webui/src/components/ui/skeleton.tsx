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
