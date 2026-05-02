import {
  faFolderOpen,
  faStop,
  faTrash,
} from "@fortawesome/free-solid-svg-icons";
import { Link } from "@tanstack/react-router";
import type {
  BuildJobResponse,
  JobResourceUsageSample,
  ServerHardwareResponse,
} from "../../../lib/types";
import { formatDateTime, formatDurationBetween } from "../../../lib/datetime";
import Button from "../../../components/ui/button";
import FaIcon from "../../../components/ui/fa-icon";
import MetaPair from "../../../components/ui/meta-pair";
import StatusPill from "../../../components/ui/status-pill";
import Tooltip from "../../../components/ui/tooltip";
import type { JobViewMode } from "../types";
import JobUsageBar from "./job-usage-bar";

interface JobCardProps {
  entry: BuildJobResponse;
  mode: JobViewMode;
  killing: boolean;
  usage: JobResourceUsageSample | null;
  serverHardware: ServerHardwareResponse | null;
  onKill: (job: BuildJobResponse) => void;
  onDelete: (job: BuildJobResponse) => void;
}

const STATUS_RAIL: Record<string, string> = {
  succeeded: "var(--theme-terminal-green)",
  failed: "var(--theme-error-red)",
  timed_out: "var(--theme-error-red)",
  running: "var(--theme-accent-lime)",
  pending: "var(--theme-accent-orange)",
};

export default function JobCard({
  entry,
  mode,
  killing,
  usage,
  serverHardware,
  onKill,
  onDelete,
}: JobCardProps) {
  const live = isLiveJob(entry);
  const accent = STATUS_RAIL[entry.job.status] ?? "var(--theme-text-soft)";
  const duration = formatDurationBetween(
    entry.job.created_at,
    entry.job.finished_at,
  );

  return (
    <article
      className={`relative bg-black transition-colors ${
        live
          ? "synforge-row-live border-2 border-edge-strong"
          : "border-2 border-edge-strong"
      }`}
    >
      {/* Status accent rail */}
      <span
        aria-hidden="true"
        className="absolute inset-y-0 left-0 w-1"
        style={{ background: accent }}
      />

      {/* Top: identity, status, meta, actions */}
      <div className="flex flex-col gap-3 pl-4 pr-4 py-3 sm:pl-6 sm:pr-5 sm:py-4 lg:flex-row lg:items-start lg:gap-4">
        <div className="min-w-0 flex-1">
          <div className="flex flex-wrap items-center gap-x-3 gap-y-1.5">
            <Link
              to="/jobs/view"
              search={{ id: entry.job.id }}
              className="break-all font-mono text-base font-bold uppercase text-white transition-colors hover:text-accent-lime sm:text-lg"
            >
              {entry.job.package_name}
            </Link>
            <StatusPill status={entry.job.status} />
            <span className="border-2 border-edge-strong bg-black px-2 py-0.5 font-mono text-[10px] font-bold uppercase tracking-[0.14em] text-soft">
              {entry.job.mock_chroot}
            </span>
          </div>

          <div className="mt-2 flex flex-wrap items-start gap-x-6 gap-y-2 font-mono text-xs">
            <MetaPair label="Revision">
              <span className="break-all text-strong">
                {entry.job.revision || (
                  <em className="not-italic text-soft">—</em>
                )}
              </span>
            </MetaPair>
            <MetaPair label="Trigger">
              <span className="text-strong">{entry.job.trigger}</span>
            </MetaPair>
            <MetaPair label="Created">
              <span className="text-strong">
                {formatDateTime(entry.job.created_at)}
              </span>
            </MetaPair>
            <MetaPair label="Duration">
              <span className="text-strong">{duration}</span>
            </MetaPair>
            <MetaPair label="Job">
              <span className="break-all text-soft">{entry.job.id}</span>
            </MetaPair>
          </div>
        </div>

        <div className="flex shrink-0 gap-1">
          <Tooltip content="Open job detail" side="top">
            <Link
              to="/jobs/view"
              search={{ id: entry.job.id }}
              aria-label={`Open job ${entry.job.id}`}
              className="inline-flex h-8 w-8 items-center justify-center border-2 border-edge-strong bg-transparent text-strong transition-colors hover:border-muted hover:bg-surface-hover hover:text-white focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent-lime"
            >
              <FaIcon icon={faFolderOpen} />
            </Link>
          </Tooltip>
          {mode === "active" && live ? (
            <Tooltip content="Kill active job" side="top">
              <Button
                variant="ghost"
                size="icon-sm"
                onClick={() => onKill(entry)}
                disabled={killing}
                aria-label={`Kill job ${entry.job.id}`}
                className="hover:border-accent-orange hover:text-accent-orange"
              >
                <FaIcon icon={faStop} />
              </Button>
            </Tooltip>
          ) : null}
          {mode !== "active" ? (
            <Tooltip content="Delete build" side="top">
              <Button
                variant="ghost"
                size="icon-sm"
                onClick={() => onDelete(entry)}
                disabled={live}
                aria-label={`Delete job ${entry.job.id}`}
                className="hover:border-error hover:text-error"
              >
                <FaIcon icon={faTrash} />
              </Button>
            </Tooltip>
          ) : null}
        </div>
      </div>

      {/* Live usage substrip — only when actively running and we have a sample */}
      {mode === "active" && live && usage ? (
        <div className="border-t-2 border-edge bg-surface-alt/40 px-4 py-3 sm:px-5">
          <JobUsageBar usage={usage} serverHardware={serverHardware} />
        </div>
      ) : null}
    </article>
  );
}

function isLiveJob(entry: BuildJobResponse): boolean {
  return entry.job.status === "pending" || entry.job.status === "running";
}
