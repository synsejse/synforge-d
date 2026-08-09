import {
  faFolderOpen,
  faRotateRight,
  faStop,
  faTrash,
} from "@fortawesome/free-solid-svg-icons";
import type { IconDefinition } from "@fortawesome/fontawesome-svg-core";
import { Link } from "@tanstack/react-router";
import type { ReactNode } from "react";
import type {
  BuildJobResponse,
  JobResourceUsageSample,
} from "../../../lib/types";
import { formatBytes } from "../../../lib/bytes";
import { formatDateTime, formatJobDuration } from "../../../lib/datetime";
import Button from "../../../components/ui/button";
import FaIcon from "../../../components/ui/fa-icon";
import StatusPill from "../../../components/ui/status-pill";
import Tooltip from "../../../components/ui/tooltip";
import CompactId from "../../../components/ui/compact-id";
import {
  formatCcacheCount,
  formatCcacheRate,
  getCcacheMetrics,
} from "../../cache/ccache-metrics";
import type { JobViewMode } from "../types";

interface JobCardProps {
  entry: BuildJobResponse;
  mode: JobViewMode;
  killing: boolean;
  usage: JobResourceUsageSample | null;
  queuePosition: number | null;
  queueLength: number;
  onKill: (job: BuildJobResponse) => void;
  onRetry: (job: BuildJobResponse) => void;
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
  queuePosition,
  queueLength,
  onKill,
  onRetry,
  onDelete,
}: JobCardProps) {
  const live = isLiveJob(entry);
  const isDeleted = entry.job.deleted_at != null;
  const isActiveCard = mode === "active" && live;

  if (isActiveCard) {
    return (
      <ActiveJobCard
        entry={entry}
        usage={usage}
        killing={killing}
        queuePosition={queuePosition}
        queueLength={queueLength}
        onKill={onKill}
      />
    );
  }

  // Soft-deleted rows: pin the rail to grey and dim the body so they read as
  // "kept for stats, not actionable" rather than a fresh failure.
  const accent = isDeleted
    ? "var(--theme-text-soft)"
    : (STATUS_RAIL[entry.job.status] ?? "var(--theme-text-soft)");
  const duration = formatJobDuration(entry.job);
  const cacheMetrics = entry.ccache_stats
    ? getCcacheMetrics(entry.ccache_stats)
    : null;

  return (
    <article
      className={`sf-row relative border border-edge bg-black pl-[22px] pr-[18px] py-4 transition-colors hover:border-edge-strong hover:bg-[#0c0c0d] ${
        isDeleted ? "opacity-60" : ""
      }`}
    >
      <span
        aria-hidden="true"
        className="absolute inset-y-0 left-0 w-[2px]"
        style={{ background: accent }}
      />

      <div className="flex flex-wrap items-center gap-3">
        <Link
          to="/jobs/view"
          search={{ id: entry.job.id }}
          className="break-all font-mono text-[15px] font-bold uppercase tracking-[0.02em] text-white transition-colors hover:text-accent-lime"
        >
          {entry.job.package_name}
        </Link>
        <StatusPill status={entry.job.status} />
        {isDeleted ? (
          <span
            className="border border-edge bg-black px-2 py-[5px] font-mono text-xs font-semibold uppercase leading-none tracking-[0.1em] text-soft"
            title="This job's artifacts and logs were pruned. The row is kept so historical statistics still see it."
          >
            Deleted
          </span>
        ) : null}
        <span className="border border-edge bg-black px-2 py-[5px] font-mono text-xs font-medium uppercase leading-none tracking-[0.06em] text-muted">
          {entry.job.mock_chroot}
        </span>

        <div className="ml-auto flex gap-1.5">
          <IconLink
            to="/jobs/view"
            search={{ id: entry.job.id }}
            icon={faFolderOpen}
            label={`Open job ${entry.job.id}`}
            tooltip="Open job detail"
          />
          {!isDeleted ? (
            <IconButton
              icon={faRotateRight}
              label={`Retry job ${entry.job.id}`}
              tooltip="Retry build"
              onClick={() => onRetry(entry)}
            />
          ) : null}
          {!isDeleted ? (
            <IconButton
              icon={faTrash}
              label={`Delete job ${entry.job.id}`}
              tooltip="Delete build"
              onClick={() => onDelete(entry)}
              danger
            />
          ) : null}
        </div>
      </div>

      <div className="mt-3.5 grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-[minmax(0,2.2fr)_0.9fr_1.3fr_0.8fr] lg:gap-[18px]">
        <div className="min-w-0">
          <JobMeta label="Revision">
            <span className="break-all text-[#71717a]">
              {entry.job.revision || "—"}
            </span>
          </JobMeta>
          <div className="mt-2.5">
            <JobMeta label="Job">
              <CompactId value={entry.job.id} className="text-soft" />
            </JobMeta>
          </div>
        </div>
        <JobMeta label="Trigger">
          <span className="text-muted">{entry.job.trigger}</span>
        </JobMeta>
        <JobMeta label="Created">
          <span className="text-muted">{formatDateTime(entry.job.created_at)}</span>
        </JobMeta>
        <JobMeta label={duration.label}>
          <span className="font-semibold text-strong">{duration.value}</span>
        </JobMeta>
      </div>

      {entry.job.error_message ? (
        <div className="mt-3.5 border-l-2 border-error bg-error/5 px-3 py-2 font-mono text-xs text-error">
          {entry.job.error_message}
        </div>
      ) : null}

      {cacheMetrics && entry.ccache_stats ? (
        <div className="mt-3.5 border border-edge bg-surface-alt px-3 py-2 font-mono text-xs text-muted">
          ccache {formatCcacheRate(cacheMetrics.hitRate)} hit rate ·{" "}
          {formatCcacheCount(cacheMetrics.hits)} hits /{" "}
          {formatCcacheCount(cacheMetrics.cacheableCalls)} cacheable calls
        </div>
      ) : null}
    </article>
  );
}

function ActiveJobCard({
  entry,
  usage,
  killing,
  queuePosition,
  queueLength,
  onKill,
}: {
  entry: BuildJobResponse;
  usage: JobResourceUsageSample | null;
  killing: boolean;
  queuePosition: number | null;
  queueLength: number;
  onKill: (job: BuildJobResponse) => void;
}) {
  const duration = formatJobDuration(entry.job);
  const cpuPct = usage
    ? clamp((usage.cpu_percent / (Math.max(1, usage.online_cpus) * 100)) * 100)
    : 0;
  const memPct =
    usage && usage.memory_limit_bytes > 0
      ? clamp((usage.memory_usage_bytes / usage.memory_limit_bytes) * 100)
      : 0;
  const pending = entry.job.status === "pending";
  const accentClass = pending ? "bg-accent-orange" : "bg-accent-lime";

  return (
    <article
      className={`${pending ? "" : "synforge-row-live"} relative border border-edge bg-[#070708] pl-[22px] pr-[18px] py-4`}
    >
      <span
        aria-hidden="true"
        className={`absolute inset-y-0 left-0 w-[2px] ${accentClass}`}
      />
      <div className="flex flex-wrap items-center gap-3">
        <Link
          to="/jobs/view"
          search={{ id: entry.job.id }}
          className="font-mono text-[15px] font-bold uppercase tracking-[0.02em] text-white transition-colors hover:text-accent-lime"
        >
          {entry.job.package_name}
        </Link>
        <StatusPill status={entry.job.status} />
        <span className="border border-edge bg-black px-2 py-[5px] font-mono text-xs font-medium uppercase leading-none tracking-[0.06em] text-muted">
          {entry.job.mock_chroot}
        </span>
        <div className="ml-auto flex items-center gap-3">
          <span className="font-mono text-xs font-semibold tabular-nums text-soft">
            {duration.label} for {duration.value}
          </span>
          <IconButton
            icon={faStop}
            label={`${pending ? "Cancel" : "Kill"} job ${entry.job.id}`}
            tooltip={pending ? "Cancel queued job" : "Kill active job"}
            onClick={() => onKill(entry)}
            disabled={killing}
            danger
          />
        </div>
      </div>

      {pending && queuePosition != null ? (
        <div className="mt-3 border border-accent-orange/50 bg-black px-3 py-2 font-mono text-xs text-accent-orange">
          Queue position {queuePosition} of {queueLength}
        </div>
      ) : null}

      {usage ? (
        <div className="mt-3.5 grid grid-cols-1 gap-4 sm:grid-cols-2 sm:gap-[18px]">
          <Meter
            label="CPU Load"
            value={`${Math.round(cpuPct)}%`}
            valueClass="text-accent-lime"
            fillClass="bg-accent-lime"
            pct={cpuPct}
          />
          <Meter
            label="Memory"
            value={formatBytes(usage.memory_usage_bytes)}
            valueClass="text-accent-cyan"
            fillClass="bg-accent-cyan"
            pct={memPct}
          />
        </div>
      ) : null}

      <div className="mt-3 font-mono text-xs leading-none text-[#52525b]">
        <CompactId value={entry.job.id} className="text-soft" />
      </div>
    </article>
  );
}

function JobMeta({ label, children }: { label: string; children: ReactNode }) {
  return (
    <div className="min-w-0">
      <div className="font-mono text-xs font-semibold uppercase leading-none tracking-[0.18em] text-[#6b6b73]">
        {label}
      </div>
      <div className="mt-[7px] font-mono text-xs leading-[1.4]">
        {children}
      </div>
    </div>
  );
}

function Meter({
  label,
  value,
  valueClass,
  fillClass,
  pct,
}: {
  label: string;
  value: string;
  valueClass: string;
  fillClass: string;
  pct: number;
}) {
  return (
    <div>
      <div className="flex items-center justify-between font-mono text-xs font-semibold uppercase leading-none tracking-[0.14em] text-soft">
        <span>{label}</span>
        <span className={`tabular-nums ${valueClass}`}>{value}</span>
      </div>
      <div className="mt-[7px] h-1.5 border border-edge bg-[#161618]">
        <div className={`h-full ${fillClass}`} style={{ width: `${pct}%` }} />
      </div>
    </div>
  );
}

function IconLink({
  to,
  search,
  icon,
  label,
  tooltip,
}: {
  to: string;
  search: Record<string, unknown>;
  icon: IconDefinition;
  label: string;
  tooltip: string;
}) {
  return (
    <Tooltip content={tooltip} side="top">
      <Link
        to={to}
        search={search}
        aria-label={label}
        className="sf-ic inline-flex h-10 w-10 sm:h-9 sm:w-9 items-center justify-center border border-edge bg-transparent text-soft transition-colors hover:border-accent-lime hover:text-accent-lime focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent-lime"
      >
        <FaIcon icon={icon} className="text-[13px]" />
      </Link>
    </Tooltip>
  );
}

function IconButton({
  icon,
  label,
  tooltip,
  onClick,
  disabled,
  danger,
}: {
  icon: IconDefinition;
  label: string;
  tooltip: string;
  onClick: () => void;
  disabled?: boolean;
  danger?: boolean;
}) {
  return (
    <Tooltip content={tooltip} side="top">
      <Button
        variant="ghost"
        size="icon-sm"
        onClick={onClick}
        disabled={disabled}
        aria-label={label}
        className={`h-10 w-10 sm:h-9 sm:w-9 border-edge text-soft ${
          danger
            ? "hover:border-error hover:text-error"
            : "hover:border-accent-lime hover:text-accent-lime"
        }`}
      >
        <FaIcon icon={icon} className="text-[13px]" />
      </Button>
    </Tooltip>
  );
}

function clamp(v: number): number {
  if (!Number.isFinite(v)) return 0;
  return Math.max(0, Math.min(100, v));
}

function isLiveJob(entry: BuildJobResponse): boolean {
  return entry.job.status === "pending" || entry.job.status === "running";
}
