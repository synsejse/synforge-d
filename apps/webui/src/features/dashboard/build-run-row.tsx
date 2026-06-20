import { Link } from "@tanstack/react-router";
import type { BuildJobResponse } from "../../lib/types";
import { formatDateTime } from "../../lib/datetime";

interface BuildRunRowProps {
  entry: BuildJobResponse;
  /** Last row drops its divider so it sits flush with the card edge. */
  last?: boolean;
}

const STATUS_STYLE: Record<string, { cls: string; dot: string; label: string }> = {
  succeeded: { cls: "border-success text-success", dot: "bg-success", label: "Succeeded" },
  failed: { cls: "border-error text-error", dot: "bg-error", label: "Failed" },
  timed_out: { cls: "border-error text-error", dot: "bg-error", label: "Timed out" },
  running: { cls: "border-accent-lime text-accent-lime", dot: "bg-accent-lime", label: "Running" },
  pending: { cls: "border-accent-orange text-accent-orange", dot: "bg-accent-orange", label: "Pending" },
};

/**
 * Flat one-line build-run row for the dashboard "Latest build runs" list.
 * Rows are not individually boxed — they share the section frame and are
 * separated by a hairline divider, matching the design comp.
 */
export default function BuildRunRow({ entry, last = false }: BuildRunRowProps) {
  const status = STATUS_STYLE[entry.job.status] ?? {
    cls: "border-edge-strong text-soft",
    dot: "bg-soft",
    label: entry.job.status,
  };

  return (
    <Link
      to="/jobs/view"
      search={{ id: entry.job.id }}
      className={`group flex items-center gap-3.5 border-l border-l-transparent px-[18px] py-[13px] transition-colors hover:border-l-edge-strong hover:bg-[#0c0c0d] sm:gap-3.5 ${
        last ? "" : "border-b border-b-[#161618]"
      }`}
    >
      <span className="shrink-0 truncate font-mono text-[13px] font-bold leading-none text-white group-hover:text-accent-lime sm:w-[118px]">
        {entry.job.package_name}
      </span>
      <span className="hidden shrink-0 border border-edge px-[7px] py-1 font-mono text-[9px] font-medium uppercase leading-none tracking-[0.06em] text-muted sm:inline-block">
        {entry.job.mock_chroot}
      </span>
      <span className="hidden min-w-0 flex-1 truncate font-mono text-[11px] leading-none text-[#52525b] md:block">
        {entry.job.revision || "—"}
      </span>
      <span
        className={`inline-flex shrink-0 items-center gap-1.5 border px-2 py-[5px] font-mono text-[9px] font-semibold uppercase leading-none tracking-[0.1em] ${status.cls}`}
      >
        <span aria-hidden="true" className={`h-[5px] w-[5px] ${status.dot}`} />
        {status.label}
      </span>
      <span className="hidden shrink-0 text-right font-mono text-[11px] leading-none text-[#71717a] lg:block lg:w-[150px]">
        {formatDateTime(entry.job.created_at)}
      </span>
      <span aria-hidden="true" className="shrink-0 font-mono text-[#52525b] group-hover:text-accent-lime">
        ›
      </span>
    </Link>
  );
}
