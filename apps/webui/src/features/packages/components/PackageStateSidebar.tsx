import type { PackageResponse } from "../../../lib/types";
import { formatDurationSeconds } from "../../../lib/datetime";
import { formatMockChroots } from "../../../lib/utils";
import DetailStat from "../../../components/ui/DetailStat";
import StatusPill from "../../../components/ui/StatusPill";

interface PackageStateSidebarProps {
  pkg: PackageResponse;
}

export default function PackageStateSidebar({ pkg }: PackageStateSidebarProps) {
  return (
    <aside className="min-w-0 space-y-4 border-4 border-white bg-black p-4 sm:p-6 shadow-[6px_6px_0_rgba(255,255,255,0.18)]">
      <div>
        <h2 className="font-mono text-xl font-bold uppercase text-white">State</h2>
        <p className="mt-2 text-sm text-zinc-400">
          Current package and build status.
        </p>
      </div>
      <div>
        <div className="mb-2 text-xs uppercase tracking-[0.18em] text-zinc-500">
          Status
        </div>
        <StatusPill status={pkg.package.enabled ? "enabled" : "disabled"} />
      </div>
      <DetailStat
        label="Version"
        value={`${pkg.package.version}-${pkg.package.release}`}
      />
      <DetailStat
        label="Mock Chroots"
        value={formatMockChroots(pkg.package.mock_chroots)}
      />
      <DetailStat label="Repository" value={pkg.package.source.repo_url} mono />
      <DetailStat
        label="Poll Interval"
        value={`${pkg.package.poll_interval_seconds ?? 0}s`}
      />
      <DetailStat
        label="Build Timeout"
        value={`${pkg.package.build_timeout_seconds ?? 0}s`}
      />
      <DetailStat label="History Count" value={pkg.package.package_history_count ?? 0} />
      <DetailStat
        label="Network Access"
        value={pkg.package.network_access ? "Enabled" : "Disabled"}
      />
      <DetailStat
        label="Shared ccache"
        value={pkg.package.ccache_enabled ? "Enabled" : "Disabled"}
      />
      <DetailStat
        label="Shared ccache size"
        value={
          pkg.package.ccache_max_size_mb
            ? `${pkg.package.ccache_max_size_mb} MB`
            : "Mock default"
        }
      />
      <DetailStat label="Build Env Vars" value={String(pkg.package.build_env?.length ?? 0)} />
      <DetailStat
        label="Last Successful Revision"
        value={pkg.state.last_revision || "None yet"}
      />
      <DetailStat label="Active Job" value={pkg.state.active_job_id || "None"} />
      <DetailStat label="Spec File" value={pkg.package.spec_file} mono />
      <div className="border-2 border-zinc-700 bg-black px-4 py-3">
        <div className="font-mono text-xs uppercase tracking-[0.18em] text-zinc-500">
          Target State
        </div>
        <div className="mt-3 space-y-3">
          {pkg.state.targets.map((target) => (
            <div
              key={target.mock_chroot}
              className="flex flex-col gap-2 border-2 border-zinc-700 bg-zinc-950/60 px-3 py-3"
            >
              <div className="flex items-center justify-between gap-3">
                <span className="min-w-0 break-all font-mono text-sm text-zinc-100">
                  {target.mock_chroot}
                </span>
                <StatusPill
                  status={
                    target.active_status ||
                    (target.last_successful_build_id ? "succeeded" : "idle")
                  }
                />
              </div>
              <div className="break-all text-xs text-zinc-500">
                {target.last_revision || "No successful revision yet"}
              </div>
              {target.backoff_remaining_seconds && target.backoff_remaining_seconds > 0 && (
                <div className="border border-[var(--theme-accent-orange)] bg-black px-2 py-1 font-mono text-[11px] uppercase tracking-[0.14em] text-[var(--theme-accent-orange)]">
                  Backoff {formatDurationSeconds(target.backoff_remaining_seconds)}
                </div>
              )}
            </div>
          ))}
        </div>
      </div>
    </aside>
  );
}
