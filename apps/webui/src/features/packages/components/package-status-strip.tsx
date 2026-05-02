import type { PackageResponse } from "../../../lib/types";
import { formatDurationSeconds } from "../../../lib/datetime";
import MetaPair from "../../../components/ui/meta-pair";
import StatusPill from "../../../components/ui/status-pill";

interface Props {
  pkg: PackageResponse;
}

/**
 * Compact read-only status header. Replaces the old right-side
 * `<PackageStateSidebar>` with a horizontal strip that surfaces the
 * "what's happening right now" data: enabled state, last successful
 * revision, active job, and per-target state.
 *
 * Settings (poll interval, build timeout, ccache, etc.) are not
 * duplicated here — they live in the edit form below.
 */
export default function PackageStatusStrip({ pkg }: Props) {
  const enabled = pkg.package.enabled ?? true;
  const lastRevision = pkg.state.last_revision || null;
  const activeJobId = pkg.state.active_job_id || null;
  const targets = pkg.state.targets ?? [];

  return (
    <section
      aria-label="Package status"
      className="border-2 border-edge-strong bg-black"
    >
      <div className="flex flex-wrap items-start gap-x-6 gap-y-3 border-b-2 border-edge px-4 py-3 sm:px-5">
        <div className="self-center">
          <StatusPill status={enabled ? "enabled" : "disabled"} />
        </div>
        <MetaPair label="Last revision">
          <span className="break-all font-mono text-xs text-strong">
            {lastRevision ? shortRevision(lastRevision) : <em className="not-italic text-soft">none yet</em>}
          </span>
        </MetaPair>
        <MetaPair label="Active job">
          {activeJobId ? (
            <span className="break-all font-mono text-xs text-accent-lime">
              {activeJobId}
            </span>
          ) : (
            <span className="font-mono text-xs text-soft">idle</span>
          )}
        </MetaPair>
        <MetaPair label="Targets">
          <span className="font-mono text-xs text-strong">{targets.length}</span>
        </MetaPair>
      </div>

      {targets.length > 0 ? (
        <ul className="grid grid-cols-1 sm:grid-cols-2 xl:grid-cols-3">
          {targets.map((target, idx) => {
            const status =
              target.active_status ||
              (target.last_successful_build_id ? "succeeded" : "idle");
            const backoffSec = target.backoff_remaining_seconds ?? 0;
            return (
              <li
                key={target.mock_chroot}
                className={
                  idx % 1 === 0
                    ? "flex flex-col gap-1 px-4 py-3 sm:px-5 [&:not(:last-child)]:border-b-2 [&:not(:last-child)]:border-edge sm:[&:not(:last-child)]:border-b-0 sm:border-r-2 sm:border-edge sm:last:border-r-0 xl:[&:nth-child(3n)]:border-r-0"
                    : ""
                }
              >
                <div className="flex items-center justify-between gap-3">
                  <span className="min-w-0 break-all font-mono text-sm text-white">
                    {target.mock_chroot}
                  </span>
                  <StatusPill status={status} />
                </div>
                <div className="flex flex-wrap items-center gap-x-3 gap-y-1 font-mono text-[11px] text-soft">
                  <span className="break-all">
                    {target.last_revision
                      ? shortRevision(target.last_revision)
                      : "no revision yet"}
                  </span>
                  {backoffSec > 0 ? (
                    <span className="border border-accent-orange px-1.5 py-0.5 uppercase tracking-[0.14em] text-accent-orange">
                      backoff {formatDurationSeconds(backoffSec)}
                    </span>
                  ) : null}
                </div>
              </li>
            );
          })}
        </ul>
      ) : null}
    </section>
  );
}

function shortRevision(rev: string): string {
  // Show up to 12 chars; if it's a git sha it'll cleanly show short form.
  return rev.length > 12 ? rev.slice(0, 12) : rev;
}
