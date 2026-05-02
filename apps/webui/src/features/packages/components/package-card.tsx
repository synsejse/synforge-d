import {
  faHammer,
  faRotate,
  faTrash,
} from "@fortawesome/free-solid-svg-icons";
import { Link } from "@tanstack/react-router";
import { formatDurationSeconds } from "../../../lib/datetime";
import type {
  PackageResponse,
  PackageTargetState,
} from "../../../lib/types";
import Button from "../../../components/ui/button";
import FaIcon from "../../../components/ui/fa-icon";
import MetaPair from "../../../components/ui/meta-pair";
import StatusPill from "../../../components/ui/status-pill";
import Tooltip from "../../../components/ui/tooltip";
import {
  compactRevision,
  summarizePackageStatus,
  targetStatus,
} from "./package-state";
import { packageAccent } from "./package-accent";

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

  const accent = packageAccent(entry.package.name);

  return (
    <article
      className={`relative bg-black transition-colors ${selected ? "border-2 border-accent-lime" : "border-2 border-edge-strong"}`}
    >
      {/* Per-package accent rail — deterministic hue from the name. */}
      <span
        aria-hidden="true"
        className="absolute inset-y-0 left-0 w-1"
        style={{ background: accent }}
      />
      {/* Top: identity, status, meta, actions */}
      <div className="flex flex-col gap-3 pl-4 pr-4 py-3 sm:pl-6 sm:pr-5 sm:py-4 lg:flex-row lg:items-start lg:gap-4">
        {selectable ? (
          <input
            type="checkbox"
            checked={selected}
            onChange={(event) =>
              onToggleSelected?.(entry.package.name, event.target.checked)
            }
            aria-label={`Select package ${entry.package.name}`}
            className="mt-1 shrink-0"
          />
        ) : null}

        <div className="min-w-0 flex-1">
          <div className="flex flex-wrap items-center gap-x-3 gap-y-1.5">
            <Link
              to="/packages/view"
              search={{ name: entry.package.name }}
              className="break-all font-mono text-base font-bold uppercase text-white transition-colors hover:text-accent-lime sm:text-lg"
            >
              {entry.package.name}
            </Link>
            <StatusPill status={status} />
            {backoffTargets.length > 0 ? (
              <Tooltip
                content={summarizeBackoffTargets(backoffTargets)}
                side="top"
              >
                <span className="border-2 border-accent-orange bg-black px-2 py-0.5 font-mono text-[10px] font-bold uppercase tracking-[0.14em] text-accent-orange">
                  Backoff {backoffTargets.length}
                </span>
              </Tooltip>
            ) : null}
          </div>

          <div className="mt-2 flex flex-wrap items-start gap-x-6 gap-y-2 font-mono text-xs">
            <MetaPair label="Version">
              <span className="text-strong">{version}</span>
            </MetaPair>
            <MetaPair label="Revision">
              {lastRevision ? (
                <span className="break-all text-strong">
                  {compactRevision(lastRevision)}
                </span>
              ) : (
                <span className="text-soft">none yet</span>
              )}
            </MetaPair>
            <MetaPair label="Targets">
              <span className="text-strong">{targets.length}</span>
            </MetaPair>
          </div>

          {description ? (
            <p className="mt-2 line-clamp-1 text-xs text-soft">
              {description}
            </p>
          ) : null}
        </div>

        <div className="flex shrink-0 gap-1">
          <Tooltip content="Refresh sources" side="top">
            <Button
              variant="ghost"
              size="icon-sm"
              onClick={() => onRefresh(entry.package.name)}
              aria-label={`Refresh package ${entry.package.name}`}
              disabled={refreshDisabled || refreshing}
              loading={refreshing}
            >
              {refreshing ? null : <FaIcon icon={faRotate} />}
            </Button>
          </Tooltip>
          <Tooltip content="Rebuild package" side="top">
            <Button
              variant="ghost"
              size="icon-sm"
              onClick={() => onRebuild(entry.package.name)}
              aria-label={`Rebuild package ${entry.package.name}`}
              className="hover:border-accent-lime hover:text-accent-lime"
            >
              <FaIcon icon={faHammer} />
            </Button>
          </Tooltip>
          <Tooltip content="Delete package" side="top">
            <Button
              variant="ghost"
              size="icon-sm"
              onClick={() => onDelete(entry.package.name)}
              aria-label={`Delete package ${entry.package.name}`}
              className="hover:border-error hover:text-error"
            >
              <FaIcon icon={faTrash} />
            </Button>
          </Tooltip>
        </div>
      </div>

      {/* Targets strip — flat list, only when there are targets */}
      {targets.length > 0 ? (
        <ul className="flex flex-wrap gap-x-5 gap-y-2 border-t-2 border-edge bg-surface-alt/40 px-4 py-2.5 sm:px-5">
          {targets.map((target) => (
            <li key={target.mock_chroot} className="min-w-0">
              <TargetChip target={target} />
            </li>
          ))}
        </ul>
      ) : null}
    </article>
  );
}

function TargetChip({ target }: { target: PackageTargetState }) {
  const status = targetStatus(target);
  const backoff = target.backoff_remaining_seconds ?? 0;
  return (
    <div className="inline-flex flex-wrap items-center gap-x-2 gap-y-1 font-mono text-[11px]">
      <span className="break-all text-white">{target.mock_chroot}</span>
      <StatusDot status={status} />
      <span className="break-all text-soft">
        {target.last_revision
          ? compactRevision(target.last_revision)
          : "no revision yet"}
      </span>
      {backoff > 0 ? (
        <span className="border border-accent-orange px-1.5 py-0.5 text-[10px] uppercase tracking-[0.14em] text-accent-orange">
          backoff {formatDurationSeconds(backoff)}
        </span>
      ) : null}
    </div>
  );
}

const STATUS_DOTS: Record<string, string> = {
  pending: "bg-accent-orange",
  running: "bg-accent-lime",
  succeeded: "bg-success",
  failed: "bg-error",
  timed_out: "bg-error",
  enabled: "bg-success",
  disabled: "bg-soft",
};

function StatusDot({ status }: { status: string }) {
  const dot = STATUS_DOTS[status] ?? "bg-muted";
  return (
    <span
      title={status}
      aria-label={`status ${status}`}
      className={`inline-block h-2 w-2 ${dot}`}
    />
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
