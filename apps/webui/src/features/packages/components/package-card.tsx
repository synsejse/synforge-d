import {
  faFolderOpen,
  faHammer,
  faRotate,
  faTrash,
} from "@fortawesome/free-solid-svg-icons";
import { formatDurationSeconds } from "../../../lib/datetime";
import type { PackageResponse, PackageTargetState } from "../../../lib/types";
import { Link } from "@tanstack/react-router";
import ActionButton from "../../../components/ui/action-button";
import StatusPill from "../../../components/ui/status-pill";
import {
  compactRevision,
  formatMockChroots,
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
  const backoffSummary =
    backoffTargets.length > 0 ? summarizeBackoffTargets(backoffTargets) : null;
  const selectable = onToggleSelected !== undefined;

  return (
    <article
      key={entry.package.name}
      className={`border-2 bg-black transition-colors ${selected ? "border-accent-lime" : "border-edge-strong"}`}
    >
      <div className="flex flex-col gap-5 border-b-2 border-edge-strong px-4 py-4 sm:px-5 sm:py-5 xl:flex-row xl:items-start xl:justify-between">
        <div className="min-w-0 flex-1 space-y-4">
          <div className="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
            <div className="flex min-w-0 items-start gap-3">
              {selectable ? (
                <input
                  type="checkbox"
                  checked={selected}
                  onChange={(event) =>
                    onToggleSelected?.(
                      entry.package.name,
                      event.target.checked,
                    )
                  }
                  aria-label={`Select package ${entry.package.name}`}
                  className="mt-1.5 shrink-0"
                />
              ) : null}
              <div className="min-w-0">
                <Link
                  to="/packages/view"
                  search={{ name: entry.package.name }}
                  className="font-mono text-lg font-bold uppercase text-white transition-all duration-100 ease-linear hover:text-accent-lime"
                >
                  {entry.package.name}
                </Link>
                <div className="mt-1 max-w-3xl text-sm text-soft">
                  {entry.package.description || "No description"}
                </div>
              </div>
            </div>
            <div className="flex items-center gap-3">
              {backoffSummary ? (
                <span
                  title={backoffSummary.details}
                  className="border-2 border-accent-orange bg-black px-3 py-1 font-mono text-[11px] font-bold uppercase tracking-[0.14em] text-accent-orange"
                >
                  Backoff {backoffSummary.count}
                </span>
              ) : null}
              <StatusPill status={status} />
            </div>
          </div>

          <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-4">
            <div className="border-2 border-edge-strong bg-surface-alt/40 px-4 py-3">
              <div className="font-mono text-[11px] uppercase tracking-[0.18em] text-soft">
                Version
              </div>
              <div className="mt-2 font-mono text-sm text-strong">
                {entry.package.version}-{entry.package.release}
              </div>
            </div>
            <div className="border-2 border-edge-strong bg-surface-alt/40 px-4 py-3">
              <div className="font-mono text-[11px] uppercase tracking-[0.18em] text-soft">
                Targets
              </div>
              <div className="mt-2 font-mono text-sm text-muted">
                {formatMockChroots(entry.package.mock_chroots)}
              </div>
            </div>
            <div className="border-2 border-edge-strong bg-surface-alt/40 px-4 py-3 md:col-span-2">
              <div className="font-mono text-[11px] uppercase tracking-[0.18em] text-soft">
                Repository
              </div>
              <div className="mt-2 break-all font-mono text-sm text-muted">
                {entry.package.source.repo_url}
              </div>
            </div>
            <div className="border-2 border-edge-strong bg-surface-alt/40 px-4 py-3 md:col-span-2">
              <div className="font-mono text-[11px] uppercase tracking-[0.18em] text-soft">
                Spec File
              </div>
              <div className="mt-2 break-all font-mono text-sm text-muted">
                {entry.package.source.spec_file}
              </div>
            </div>
            <div className="border-2 border-edge-strong bg-surface-alt/40 px-4 py-3 md:col-span-2">
              <div className="font-mono text-[11px] uppercase tracking-[0.18em] text-soft">
                Last Revision
              </div>
              <div className="mt-2 break-all font-mono text-sm text-muted">
                {entry.state.last_revision || "None yet"}
              </div>
            </div>
          </div>
        </div>

        <div className="grid shrink-0 gap-2 sm:flex sm:flex-wrap sm:justify-end xl:max-w-[460px]">
          <ActionButton
            to="/packages/view"
            search={{ name: entry.package.name }}
            icon={faFolderOpen}
            aria-label={`Open package ${entry.package.name}`}
            className="w-full sm:w-auto"
          >
            Open
          </ActionButton>
          <ActionButton
            onClick={() => onRefresh(entry.package.name)}
            icon={faRotate}
            aria-label={`Refresh package ${entry.package.name}`}
            aria-busy={refreshing || undefined}
            className="w-full sm:w-auto"
            disabled={refreshDisabled || refreshing}
          >
            {refreshing ? "Refreshing" : "Refresh"}
          </ActionButton>
          <ActionButton
            onClick={() => onRebuild(entry.package.name)}
            icon={faHammer}
            aria-label={`Rebuild package ${entry.package.name}`}
            className="w-full sm:w-auto"
          >
            Rebuild
          </ActionButton>
          <ActionButton
            onClick={() => onDelete(entry.package.name)}
            icon={faTrash}
            aria-label={`Delete package ${entry.package.name}`}
             className="w-full text-muted sm:w-auto"
           >
             Delete
           </ActionButton>
        </div>
      </div>

      {backoffSummary ? (
        <div className="border-b-2 border-edge-strong bg-surface-alt px-4 py-3 text-xs text-muted sm:px-5">
          <span className="font-mono uppercase tracking-[0.14em] text-accent-orange">
            Failure backoff active:
          </span>{" "}
          <span className="font-mono">{backoffSummary.details}</span>
        </div>
      ) : null}

      <div className="px-4 py-4 sm:px-5">
        <div className="mb-3 font-mono text-[11px] uppercase tracking-[0.18em] text-soft">
          Target State
        </div>
        <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-3">
          {entry.state.targets.map((target) => (
            <div
              key={`${entry.package.name}:${target.mock_chroot}`}
              className="border-2 border-edge-strong bg-surface-alt/40 px-4 py-3"
            >
              <div className="flex items-start justify-between gap-3">
                <div className="min-w-0">
                  <div className="font-mono text-sm text-strong">
                    {target.mock_chroot}
                  </div>
                  <div className="mt-1 text-xs text-soft">
                    {target.active_job_id
                      ? `Active job ${target.active_job_id}`
                      : target.last_successful_build_id
                        ? `Last success ${target.last_successful_build_id}`
                        : "No successful build yet"}
                  </div>
                </div>
                <StatusPill status={targetStatus(target)} />
              </div>
              <div className="mt-3 border-t-2 border-edge-strong pt-3">
                <div className="font-mono text-[11px] uppercase tracking-[0.18em] text-soft">
                  Revision
                </div>
                <div className="mt-2 break-all font-mono text-sm text-muted">
                  {compactRevision(target.last_revision)}
                </div>
              </div>
            </div>
          ))}
        </div>
      </div>
    </article>
  );
}

function isBackoffActive(target: PackageTargetState): boolean {
  const remaining = target.backoff_remaining_seconds ?? null;
  return remaining !== null && remaining > 0;
}

function summarizeBackoffTargets(targets: PackageTargetState[]): {
  count: number;
  details: string;
} {
  const details = targets
    .map((target) => {
      const remaining = target.backoff_remaining_seconds ?? 0;
      return `${target.mock_chroot} (${formatDurationSeconds(remaining)})`;
    })
    .join(", ");

  return { count: targets.length, details };
}
