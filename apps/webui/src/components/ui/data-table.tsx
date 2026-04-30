import type { ReactNode } from "react";
import type { IconDefinition } from "@fortawesome/fontawesome-svg-core";
import { cn } from "../../lib/utils";
import EmptyState from "./empty-state";
import { SkeletonTable } from "./skeleton";

/**
 * One column declaration. The same config drives both the desktop
 * <table> rendering and the mobile card-stack rendering.
 *
 * Mobile rendering rules:
 *   - "title" — rendered as the prominent card heading. Default for index 0.
 *   - "badge" — rendered floated to the right of the title, for statuses.
 *   - "field" — rendered as a "Label: value" row in the card body. Default
 *               for indices > 0.
 *   - "hidden" — not shown on mobile (e.g. wide action columns).
 */
export interface DataTableColumn<T> {
  key: string;
  header: string;
  cell: (row: T) => ReactNode;
  mobile?: "title" | "badge" | "field" | "hidden";
  className?: string;
  headerClassName?: string;
}

interface EmptyConfig {
  title?: string;
  description?: string;
  icon?: IconDefinition;
}

interface DataTableProps<T> {
  columns: DataTableColumn<T>[];
  rows: T[];
  rowKey: (row: T) => string;
  empty?: EmptyConfig | ReactNode;
  loading?: boolean;
  loadingRows?: number;
  rowClassName?: (row: T) => string;
  /** Min table width for the desktop <table>. Defaults to 640px. */
  minWidth?: string;
  className?: string;
}

const TH_CLASS =
  "sticky top-0 z-10 bg-[var(--theme-surface-alt)] px-4 py-3 text-left font-mono text-xs font-bold uppercase tracking-[0.2em] text-[var(--theme-text-soft)] border-b-2 border-[var(--theme-border-strong)]";

const TD_CLASS = "px-4 py-3 align-middle";

function isEmptyConfig(value: unknown): value is EmptyConfig {
  if (!value || typeof value !== "object") return false;
  const v = value as Record<string, unknown>;
  return (
    "title" in v ||
    "description" in v ||
    "icon" in v
  ) && !("$$typeof" in v);
}

export default function DataTable<T>({
  columns,
  rows,
  rowKey,
  empty,
  loading = false,
  loadingRows = 5,
  rowClassName,
  minWidth = "640px",
  className,
}: DataTableProps<T>) {
  if (loading) {
    return <SkeletonTable columns={columns.length} rows={loadingRows} />;
  }

  if (rows.length === 0) {
    if (isEmptyConfig(empty)) {
      return (
        <EmptyState
          icon={empty.icon}
          title={empty.title}
          description={empty.description}
        />
      );
    }
    return <>{empty ?? <EmptyState>No records to display.</EmptyState>}</>;
  }

  const titleCol = columns.find((c) => c.mobile === "title") ?? columns[0];
  const badgeCols = columns.filter((c) => c.mobile === "badge");
  const fieldCols = columns.filter(
    (c) => c !== titleCol && !badgeCols.includes(c) && c.mobile !== "hidden",
  );

  return (
    <div className={cn("space-y-3", className)}>
      {/* Mobile card stack */}
      <div className="space-y-3 md:hidden">
        {rows.map((row) => (
          <article
            key={`m-${rowKey(row)}`}
            className={cn(
              "border-2 border-[var(--theme-border-strong)] bg-black p-4",
              rowClassName?.(row),
            )}
          >
            <div className="flex items-start justify-between gap-3">
              <div className="min-w-0">{titleCol.cell(row)}</div>
              {badgeCols.length > 0 ? (
                <div className="flex shrink-0 flex-wrap gap-2">
                  {badgeCols.map((col) => (
                    <div key={col.key}>{col.cell(row)}</div>
                  ))}
                </div>
              ) : null}
            </div>
            {fieldCols.length > 0 ? (
              <dl className="mt-3 space-y-2 font-mono text-xs text-[var(--theme-text-muted)]">
                {fieldCols.map((col) => (
                  <div key={col.key} className="break-all">
                    <dt className="inline text-[var(--theme-text-soft)]">
                      {col.header}:
                    </dt>{" "}
                    <dd className="inline">{col.cell(row)}</dd>
                  </div>
                ))}
              </dl>
            ) : null}
          </article>
        ))}
      </div>

      {/* Desktop sticky-header table */}
      <div className="hidden overflow-auto border-2 border-[var(--theme-border-strong)] md:block">
        <table className="w-full" style={{ minWidth }}>
          <thead>
            <tr>
              {columns.map((col) => (
                <th
                  key={col.key}
                  scope="col"
                  className={cn(TH_CLASS, col.headerClassName)}
                >
                  {col.header}
                </th>
              ))}
            </tr>
          </thead>
          <tbody className="divide-y divide-[var(--theme-border)] bg-black">
            {rows.map((row) => (
              <tr
                key={rowKey(row)}
                className={cn(
                  "transition-colors hover:bg-[var(--theme-surface-alt)]",
                  rowClassName?.(row),
                )}
              >
                {columns.map((col) => (
                  <td
                    key={col.key}
                    className={cn(TD_CLASS, col.className)}
                  >
                    {col.cell(row)}
                  </td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
