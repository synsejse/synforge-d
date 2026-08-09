import {
  faFolderOpen,
  faHammer,
  faRotate,
  faTrash,
} from "@fortawesome/free-solid-svg-icons";
import { Link } from "@tanstack/react-router";
import type { PackageBuildInventoryEntry } from "../../../lib/types";
import { formatDateTime, formatJobDuration } from "../../../lib/datetime";
import Button from "../../../components/ui/button";
import FaIcon from "../../../components/ui/fa-icon";
import StatusPill from "../../../components/ui/status-pill";
import Tooltip from "../../../components/ui/tooltip";
import {
  RecordCard,
  RecordMeta,
  SigningBadge,
} from "../../../components/ui/record-card";
import {
  STATUS_RAIL,
  rowActionClass,
} from "../../../components/ui/record-card-styles";
import {
  formatCcacheCount,
  formatCcacheRate,
  getCcacheMetrics,
} from "../../cache/ccache-metrics";

interface BuildHistoryCardProps {
  entry: PackageBuildInventoryEntry;
  deleting: boolean;
  onRefreshTarget: (mockChroot: string) => void;
  onRebuildTarget: (mockChroot: string) => void;
  onDeleteJob: (jobId: string) => void;
}

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
  const rail = isDeleted
    ? "var(--theme-text-soft)"
    : (STATUS_RAIL[job.status] ?? "var(--theme-text-soft)");
  const duration = formatJobDuration(job);
  const cacheMetrics = entry.build.ccache_stats
    ? getCcacheMetrics(entry.build.ccache_stats)
    : null;

  return (
    <RecordCard
      rail={rail}
      live={live}
      dimmed={isDeleted}
      title={
        <Link
          to="/jobs/view"
          search={{ id: job.id }}
          className="break-all font-mono text-[15px] font-bold uppercase tracking-[0.02em] text-white transition-colors hover:text-accent-lime"
        >
          {job.mock_chroot}
        </Link>
      }
      badges={
        <>
          <StatusPill status={job.status} />
          {isDeleted ? (
            <span
              className="border border-edge bg-black px-[7px] py-[3px] font-mono text-xs font-semibold uppercase leading-none tracking-[0.1em] text-soft"
              title="Build artifacts and logs were pruned. Row kept for statistics."
            >
              Deleted
            </span>
          ) : (
            <SigningBadge status={buildSignStatus(entry)} />
          )}
        </>
      }
      actions={
        <>
          <Tooltip content="Open job detail" side="top">
            <Link
              to="/jobs/view"
              search={{ id: job.id }}
              aria-label={`Open job ${job.id}`}
              className={rowActionClass}
            >
              <FaIcon icon={faFolderOpen} className="text-[13px]" />
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
                  className="h-10 w-10 sm:h-9 sm:w-9 border-edge text-soft hover:border-accent-lime hover:text-accent-lime"
                >
                  <FaIcon icon={faRotate} className="text-[13px]" />
                </Button>
              </Tooltip>
              <Tooltip content="Rebuild target" side="top">
                <Button
                  variant="ghost"
                  size="icon-sm"
                  onClick={() => onRebuildTarget(job.mock_chroot)}
                  aria-label={`Rebuild target ${job.mock_chroot}`}
                  className="h-10 w-10 sm:h-9 sm:w-9 border-edge text-soft hover:border-accent-lime hover:text-accent-lime"
                >
                  <FaIcon icon={faHammer} className="text-[13px]" />
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
                className="h-10 w-10 sm:h-9 sm:w-9 border-edge text-soft hover:border-error hover:text-error"
              >
                {deleting ? null : (
                  <FaIcon icon={faTrash} className="text-[13px]" />
                )}
              </Button>
            </Tooltip>
          ) : null}
        </>
      }
    >
      <RecordMeta
        items={[
          { label: "Revision", value: job.revision || "—" },
          { label: "Trigger", value: job.trigger },
          { label: "Created", value: formatDateTime(job.created_at) },
          {
            label: duration.label,
            value: (
              <span className={live ? "text-accent-lime" : undefined}>
                {duration.value}
              </span>
            ),
          },
          { label: "Repo files", value: entry.repo_files.length },
          ...(cacheMetrics && entry.build.ccache_stats
            ? [
                {
                  label: "ccache",
                  value: `${formatCcacheRate(cacheMetrics.hitRate)} · ${formatCcacheCount(entry.build.ccache_stats.compiler_calls)} calls`,
                },
              ]
            : []),
          {
            label: "Job",
            value: <span className="text-[#52525b]">{job.id}</span>,
          },
        ]}
      />
    </RecordCard>
  );
}

function buildSignStatus(
  entry: PackageBuildInventoryEntry,
): "signed" | "failed" | null {
  const signable = entry.repo_files.filter(
    (file) =>
      file.kind === "rpm" ||
      file.kind === "srpm" ||
      file.kind === "debuginfo" ||
      file.kind === "debugsource",
  );
  if (signable.length === 0) return null;
  if (signable.some((file) => file.signing_status === "failed"))
    return "failed";
  if (signable.every((file) => file.signing_status === "signed"))
    return "signed";
  return null;
}
