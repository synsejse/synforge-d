import { Link } from "@tanstack/react-router";
import { faChevronRight } from "@fortawesome/free-solid-svg-icons";
import type { BuildJobResponse } from "../../lib/types";
import { formatDateTime } from "../../lib/datetime";
import Badge from "../../components/ui/badge";
import FaIcon from "../../components/ui/fa-icon";

interface MiniJobRowProps {
  entry: BuildJobResponse;
}

const STATUS_RAIL: Record<string, string> = {
  succeeded: "var(--theme-terminal-green)",
  failed: "var(--theme-error-red)",
  timed_out: "var(--theme-error-red)",
  running: "var(--theme-accent-lime)",
  pending: "var(--theme-accent-orange)",
};

export default function MiniJobRow({ entry }: MiniJobRowProps) {
  const accent = STATUS_RAIL[entry.job.status] ?? "var(--theme-text-soft)";
  const isLive =
    entry.job.status === "running" || entry.job.status === "pending";

  return (
    <Link
      to="/jobs/view"
      search={{ id: entry.job.id }}
      className={`group relative block bg-black transition-colors hover:bg-surface-alt ${
        isLive
          ? "synforge-row-live border-2 border-edge-strong"
          : "border-2 border-edge"
      }`}
    >
      <span
        aria-hidden="true"
        className="absolute inset-y-0 left-0 w-1"
        style={{ background: accent }}
      />
      <div className="flex flex-col gap-2 pl-4 pr-3 py-2.5 sm:pl-5 sm:pr-4 md:flex-row md:items-center md:gap-4">
        <div className="flex min-w-0 flex-1 flex-wrap items-center gap-x-3 gap-y-1.5">
          <span className="font-display font-bold text-white transition-colors group-hover:text-accent-lime">
            {entry.job.package_name}
          </span>
          <span className="border-2 border-edge-strong bg-black px-2 py-0.5 font-mono text-[10px] font-bold uppercase tracking-[0.14em] text-soft">
            {entry.job.mock_chroot}
          </span>
          <span className="break-all font-mono text-xs text-soft">
            {entry.job.revision || "—"}
          </span>
        </div>
        <div className="flex shrink-0 items-center gap-3">
          <Badge variant={getStatusVariant(entry.job.status)} pulse={isLive}>
            {entry.job.status}
          </Badge>
          <span className="hidden font-mono text-[11px] text-soft md:inline">
            {formatDateTime(entry.job.created_at)}
          </span>
          <FaIcon
            icon={faChevronRight}
            className="hidden text-[11px] text-soft transition-colors group-hover:text-accent-lime md:inline"
          />
        </div>
      </div>
    </Link>
  );
}

function getStatusVariant(status: string) {
  if (status === "succeeded") return "success" as const;
  if (status === "failed" || status === "timed_out") return "error" as const;
  if (status === "running") return "lime" as const;
  if (status === "pending") return "warning" as const;
  return "default" as const;
}
