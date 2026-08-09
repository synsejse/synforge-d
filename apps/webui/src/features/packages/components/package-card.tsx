import {
  faHammer,
  faRotate,
  faTrash,
} from "@fortawesome/free-solid-svg-icons";
import type { IconDefinition } from "@fortawesome/fontawesome-svg-core";
import { Link } from "@tanstack/react-router";
import type { ReactNode } from "react";
import { formatDurationSeconds } from "../../../lib/datetime";
import type {
  PackageResponse,
  PackageTargetState,
} from "../../../lib/types";
import Button from "../../../components/ui/button";
import FaIcon from "../../../components/ui/fa-icon";
import StatusPill from "../../../components/ui/status-pill";
import Tooltip from "../../../components/ui/tooltip";
import {
  compactRevision,
  summarizePackageStatus,
  targetStatus,
} from "./package-state";

interface PackageCardProps {
  entry: PackageResponse;
  onRefresh: (name: string) => void;
  onRebuild: (name: string) => void;
  onDelete: (name: string) => void;
  refreshing?: boolean;
  refreshDisabled?: boolean;
  /** When provided, renders a selection checkbox at the top of the card. */
  selected?: boolean;
  onToggleSelected?: (name: string, value: boolean) => void;
}

const STATUS_RAIL: Record<string, string> = {
  enabled: "var(--theme-terminal-green)",
  succeeded: "var(--theme-terminal-green)",
  running: "var(--theme-accent-lime)",
  pending: "var(--theme-accent-orange)",
  failed: "var(--theme-error-red)",
  timed_out: "var(--theme-error-red)",
  disabled: "var(--theme-text-soft)",
};

const STATUS_DOTS: Record<string, string> = {
  pending: "bg-accent-orange",
  running: "bg-accent-lime",
  succeeded: "bg-success",
  failed: "bg-error",
  timed_out: "bg-error",
  enabled: "bg-success",
  disabled: "bg-soft",
};

export default function PackageCard({
  entry,
  onRefresh,
  onRebuild,
  onDelete,
  refreshing = false,
  refreshDisabled = false,
  selected,
  onToggleSelected,
}: PackageCardProps) {
  const status = summarizePackageStatus(entry);
  const backoffTargets = entry.state.targets.filter(isBackoffActive);
  const selectable = onToggleSelected !== undefined;
  const description = entry.package.description?.trim() || "";
  const lastRevision = entry.state.last_revision || null;
  const targets = entry.state.targets ?? [];
  const version = `${entry.package.version}-${entry.package.release}`;
  const rail = STATUS_RAIL[status] ?? "var(--theme-terminal-green)";

  return (
    <article className="sf-row relative border border-edge bg-black py-4 pl-[22px] pr-[18px] transition-colors hover:border-edge-strong hover:bg-[#0c0c0d]">
      <span
        aria-hidden="true"
        className="absolute inset-y-0 left-0 w-[2px]"
        style={{ background: rail }}
      />

      <div className="flex flex-wrap items-center gap-3">
        {selectable ? (
          <label className="-m-2 inline-flex h-10 w-10 shrink-0 cursor-pointer items-center justify-center">
            <input
              type="checkbox"
              checked={selected}
              onChange={(event) =>
                onToggleSelected?.(entry.package.name, event.target.checked)
              }
              aria-label={`Select package ${entry.package.name}`}
              className="shrink-0 cursor-pointer"
            />
          </label>
        ) : null}
        <Link
          to="/packages/view"
          search={{ name: entry.package.name }}
          className="inline-flex min-h-10 items-center break-all font-mono text-[15px] font-bold uppercase tracking-[0.02em] text-white transition-colors hover:text-accent-lime sm:min-h-0"
        >
          {entry.package.name}
        </Link>
        <StatusPill status={status} />
        {backoffTargets.length > 0 ? (
          <Tooltip content={summarizeBackoffTargets(backoffTargets)} side="top">
            <span className="border border-accent-orange bg-black px-2 py-[5px] font-mono text-xs font-semibold uppercase leading-none tracking-[0.1em] text-accent-orange">
              Backoff {backoffTargets.length}
            </span>
          </Tooltip>
        ) : null}

        <div className="ml-auto flex gap-1.5">
          <IconButton
            icon={faRotate}
            label={`Refresh package ${entry.package.name}`}
            tooltip="Refresh sources"
            onClick={() => onRefresh(entry.package.name)}
            disabled={refreshDisabled || refreshing}
            loading={refreshing}
          />
          <IconButton
            icon={faHammer}
            label={`Rebuild package ${entry.package.name}`}
            tooltip="Rebuild package"
            onClick={() => onRebuild(entry.package.name)}
          />
          <IconButton
            icon={faTrash}
            label={`Delete package ${entry.package.name}`}
            tooltip="Delete package"
            onClick={() => onDelete(entry.package.name)}
            danger
          />
        </div>
      </div>

      <div className="mt-3.5 grid grid-cols-1 gap-4 sm:grid-cols-[minmax(0,1fr)_minmax(0,1.3fr)_90px] sm:gap-[18px]">
        <Meta label="Version">
          <span className="break-all text-muted">{version}</span>
        </Meta>
        <Meta label="Revision">
          {lastRevision ? (
            <span className="break-all text-[#52525b]">
              {compactRevision(lastRevision)}
            </span>
          ) : (
            <span className="text-[#52525b]">none yet</span>
          )}
        </Meta>
        <Meta label="Targets">
          <span className="text-base font-bold text-strong">{targets.length}</span>
        </Meta>
      </div>

      {description ? (
        <p className="font-body mt-3 line-clamp-2 text-xs text-[#71717a]">
          {description}
        </p>
      ) : null}

      {targets.length > 0 ? (
        <div className="mt-3.5 flex flex-wrap gap-2.5 border-t border-[#161618] pt-3.5">
          {targets.map((target) => (
            <TargetChip key={target.mock_chroot} target={target} />
          ))}
        </div>
      ) : null}
    </article>
  );
}

function Meta({ label, children }: { label: string; children: ReactNode }) {
  return (
    <div className="min-w-0">
      <div className="font-mono text-xs font-semibold uppercase leading-none tracking-[0.18em] text-[#6b6b73]">
        {label}
      </div>
      <div className="mt-[7px] font-mono text-xs leading-[1.3]">{children}</div>
    </div>
  );
}

function TargetChip({ target }: { target: PackageTargetState }) {
  const status = targetStatus(target);
  const backoff = target.backoff_remaining_seconds ?? 0;
  const dot = STATUS_DOTS[status] ?? "bg-muted";
  return (
    <div className="inline-flex items-center gap-2 border border-edge px-2.5 py-1.5">
      <span className="font-mono text-xs font-semibold leading-none text-muted">
        {target.mock_chroot}
      </span>
      <span
        title={status}
        aria-label={`status ${status}`}
        className={`h-1.5 w-1.5 shrink-0 ${dot}`}
      />
      <span className="font-mono text-xs leading-none text-[#52525b]">
        {target.last_revision
          ? compactRevision(target.last_revision)
          : "no revision yet"}
      </span>
      {backoff > 0 ? (
        <span className="border border-accent-orange px-1.5 py-0.5 font-mono text-xs uppercase leading-none tracking-[0.1em] text-accent-orange">
          backoff {formatDurationSeconds(backoff)}
        </span>
      ) : null}
    </div>
  );
}

function IconButton({
  icon,
  label,
  tooltip,
  onClick,
  disabled,
  loading,
  danger,
}: {
  icon: IconDefinition;
  label: string;
  tooltip: string;
  onClick: () => void;
  disabled?: boolean;
  loading?: boolean;
  danger?: boolean;
}) {
  return (
    <Tooltip content={tooltip} side="top">
      <Button
        variant="ghost"
        size="icon-sm"
        onClick={onClick}
        disabled={disabled}
        loading={loading}
        aria-label={label}
        className={`h-10 w-10 sm:h-9 sm:w-9 border-edge text-soft ${
          danger
            ? "hover:border-error hover:text-error"
            : "hover:border-accent-lime hover:text-accent-lime"
        }`}
      >
        {loading ? null : <FaIcon icon={icon} className="text-[13px]" />}
      </Button>
    </Tooltip>
  );
}

function isBackoffActive(target: PackageTargetState): boolean {
  const remaining = target.backoff_remaining_seconds ?? null;
  return remaining !== null && remaining > 0;
}

function summarizeBackoffTargets(targets: PackageTargetState[]): string {
  return targets
    .map((target) => {
      const remaining = target.backoff_remaining_seconds ?? 0;
      return `${target.mock_chroot} (${formatDurationSeconds(remaining)})`;
    })
    .join(", ");
}
