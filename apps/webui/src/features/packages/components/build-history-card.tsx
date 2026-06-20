import {
  faFolderOpen,
  faHammer,
  faRotate,
  faTrash,
} from "@fortawesome/free-solid-svg-icons";
import { Link } from "@tanstack/react-router";
import type { PackageBuildInventoryEntry } from "../../../lib/types";
import { formatDateTime, formatJobDuration } from "../../../lib/datetime";
import Badge from "../../../components/ui/badge";
import Button from "../../../components/ui/button";
import FaIcon from "../../../components/ui/fa-icon";
import MetaPair from "../../../components/ui/meta-pair";
import StatusPill from "../../../components/ui/status-pill";
import Tooltip from "../../../components/ui/tooltip";

interface BuildHistoryCardProps {
  entry: PackageBuildInventoryEntry;
  deleting: boolean;
  onRefreshTarget: (mockChroot: string) => void;
  onRebuildTarget: (mockChroot: string) => void;
  onDeleteJob: (jobId: string) => void;
}

const STATUS_RAIL: Record<string, string> = {
  succeeded: "var(--theme-terminal-green)",
  failed: "var(--theme-error-red)",
  timed_out: "var(--theme-error-red)",
  running: "var(--theme-accent-lime)",
  pending: "var(--theme-accent-orange)",
};

export default function BuildHistoryCard({
  entry,
  deleting,
  onRefreshTarget,
  onRebuildTarget,
  onDeleteJob,
}: BuildHistoryCardProps) {
  const job = entry.build.job;
  const live = job.status === "pending" || job.status === "running";
  const isDeleted = job.deleted_at != null;
  const accent = isDeleted
    ? "var(--theme-text-soft)"
    : (STATUS_RAIL[job.status] ?? "var(--theme-text-soft)");
  const signing = getBuildSigningSummary(entry);
  const duration = formatJobDuration(job);

  return (
    <article
      className={`relative bg-black transition-colors ${
        live
          ? "synforge-row-live border border-edge"
          : "border border-edge"
      } ${isDeleted ? "opacity-60" : ""}`}
    >
      <span
        aria-hidden="true"
        className="absolute inset-y-0 left-0 w-1"
        style={{ background: accent }}
      />

      <div className="flex flex-col gap-3 pl-4 pr-4 py-3 sm:pl-6 sm:pr-5 sm:py-4 lg:flex-row lg:items-start lg:gap-4">
        <div className="min-w-0 flex-1">
          <div className="flex flex-wrap items-center gap-x-3 gap-y-1.5">
            <Link
              to="/jobs/view"
              search={{ id: job.id }}
              className="break-all font-mono text-base font-bold uppercase text-white transition-colors hover:text-accent-lime sm:text-lg"
            >
              {job.mock_chroot}
            </Link>
            <StatusPill status={job.status} />
            {isDeleted ? (
              <span
                className="border border-edge bg-black px-2 py-0.5 font-mono text-[10px] font-bold uppercase tracking-[0.18em] text-soft"
                title="Build artifacts and logs were pruned. Row kept for statistics."
              >
                Deleted
              </span>
            ) : (
              <Badge variant={signing.variant}>{signing.label}</Badge>
            )}
          </div>

          <div className="mt-2 flex flex-wrap items-start gap-x-6 gap-y-2 font-mono text-xs">
            <MetaPair label="Revision">
              <span className="break-all text-strong">{job.revision || "—"}</span>
            </MetaPair>
            <MetaPair label="Trigger">
              <span className="text-strong">{job.trigger}</span>
            </MetaPair>
            <MetaPair label="Created">
              <span className="text-strong">{formatDateTime(job.created_at)}</span>
            </MetaPair>
            <MetaPair label={duration.label}>
              <span
                className={live ? "text-accent-lime" : "text-strong"}
              >
                {duration.value}
              </span>
            </MetaPair>
            <MetaPair label="Repo files">
              <span className="text-strong">{entry.repo_files.length}</span>
            </MetaPair>
            <MetaPair label="Job">
              <span className="break-all text-soft">{job.id}</span>
            </MetaPair>
          </div>
        </div>

        <div className="flex shrink-0 gap-1">
          <Tooltip content="Open job detail" side="top">
            <Link
              to="/jobs/view"
              search={{ id: job.id }}
              aria-label={`Open job ${job.id}`}
              className="inline-flex h-8 w-8 items-center justify-center border border-edge bg-transparent text-strong transition-colors hover:border-muted hover:bg-surface-hover hover:text-white focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent-lime"
            >
              <FaIcon icon={faFolderOpen} />
            </Link>
          </Tooltip>
          {!live && !isDeleted ? (
            <>
              <Tooltip content="Refresh target" side="top">
                <Button
                  variant="ghost"
                  size="icon-sm"
                  onClick={() => onRefreshTarget(job.mock_chroot)}
                  aria-label={`Refresh target ${job.mock_chroot}`}
                >
                  <FaIcon icon={faRotate} />
                </Button>
              </Tooltip>
              <Tooltip content="Rebuild target" side="top">
                <Button
                  variant="ghost"
                  size="icon-sm"
                  onClick={() => onRebuildTarget(job.mock_chroot)}
                  aria-label={`Rebuild target ${job.mock_chroot}`}
                  className="hover:border-accent-lime hover:text-accent-lime"
                >
                  <FaIcon icon={faHammer} />
                </Button>
              </Tooltip>
            </>
          ) : null}
          {!isDeleted ? (
            <Tooltip content="Delete build" side="top">
              <Button
                variant="ghost"
                size="icon-sm"
                onClick={() => onDeleteJob(job.id)}
                disabled={live || deleting}
                loading={deleting}
                aria-label={`Delete build ${job.id}`}
                className="hover:border-error hover:text-error"
              >
                {deleting ? null : <FaIcon icon={faTrash} />}
              </Button>
            </Tooltip>
          ) : null}
        </div>
      </div>
    </article>
  );
}

function getBuildSigningSummary(entry: PackageBuildInventoryEntry) {
  const signableFiles = entry.repo_files.filter(
    (file) =>
      file.kind === "rpm" ||
      file.kind === "srpm" ||
      file.kind === "debuginfo" ||
      file.kind === "debugsource",
  );
  if (signableFiles.length === 0) {
    return { label: "NOT SIGNED", variant: "warning" as const };
  }
  if (signableFiles.some((file) => file.signing_status === "failed")) {
    return { label: "SIGN FAILED", variant: "error" as const };
  }
  if (signableFiles.every((file) => file.signing_status === "signed")) {
    return { label: "SIGNED", variant: "success" as const };
  }
  return { label: "NOT SIGNED", variant: "warning" as const };
}
