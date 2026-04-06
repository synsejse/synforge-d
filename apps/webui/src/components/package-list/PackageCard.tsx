import {
  faFolderOpen,
  faHammer,
  faRotate,
  faTrash,
} from "@fortawesome/free-solid-svg-icons";
import type { PackageResponse } from "../../lib/types";
import ActionButton from "../ui/ActionButton";
import StatusPill from "../ui/StatusPill";
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
}

export default function PackageCard({
  entry,
  onRefresh,
  onRebuild,
  onDelete,
}: PackageCardProps) {
  const status = summarizePackageStatus(entry);

  return (
    <article key={entry.package.name} className="border-2 border-zinc-700 bg-black">
      <div className="flex flex-col gap-5 border-b-2 border-zinc-700 px-5 py-5 xl:flex-row xl:items-start xl:justify-between">
        <div className="min-w-0 flex-1 space-y-4">
          <div className="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
            <div className="min-w-0">
              <a
                href={`/packages/view/?name=${encodeURIComponent(entry.package.name)}`}
                className="font-mono text-lg font-bold uppercase text-white transition hover:text-[var(--theme-accent-lime)]"
              >
                {entry.package.name}
              </a>
              <div className="mt-1 max-w-3xl text-sm text-zinc-500">
                {entry.package.description || "No description"}
              </div>
            </div>
            <div className="flex items-center gap-3">
              <StatusPill status={status} />
            </div>
          </div>

          <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-4">
            <div className="border-2 border-zinc-700 bg-zinc-950/40 px-4 py-3">
              <div className="font-mono text-[11px] uppercase tracking-[0.18em] text-zinc-500">
                Version
              </div>
              <div className="mt-2 font-mono text-sm text-zinc-200">
                {entry.package.version}-{entry.package.release}
              </div>
            </div>
            <div className="border-2 border-zinc-700 bg-zinc-950/40 px-4 py-3">
              <div className="font-mono text-[11px] uppercase tracking-[0.18em] text-zinc-500">
                Targets
              </div>
              <div className="mt-2 font-mono text-sm text-zinc-300">
                {formatMockChroots(entry.package.mock_chroots)}
              </div>
            </div>
            <div className="border-2 border-zinc-700 bg-zinc-950/40 px-4 py-3 md:col-span-2">
              <div className="font-mono text-[11px] uppercase tracking-[0.18em] text-zinc-500">
                Repository
              </div>
              <div className="mt-2 break-all font-mono text-sm text-zinc-300">
                {entry.package.source.repo_url}
              </div>
            </div>
            <div className="border-2 border-zinc-700 bg-zinc-950/40 px-4 py-3 md:col-span-2">
              <div className="font-mono text-[11px] uppercase tracking-[0.18em] text-zinc-500">
                Spec File
              </div>
              <div className="mt-2 break-all font-mono text-sm text-zinc-300">
                {entry.package.source.spec_file}
              </div>
            </div>
            <div className="border-2 border-zinc-700 bg-zinc-950/40 px-4 py-3 md:col-span-2">
              <div className="font-mono text-[11px] uppercase tracking-[0.18em] text-zinc-500">
                Last Revision
              </div>
              <div className="mt-2 break-all font-mono text-sm text-zinc-400">
                {entry.state.last_revision || "None yet"}
              </div>
            </div>
          </div>
        </div>

        <div className="flex shrink-0 flex-wrap justify-end gap-2 xl:max-w-[460px]">
          <ActionButton
            href={`/packages/view/?name=${encodeURIComponent(entry.package.name)}`}
            icon={faFolderOpen}
            aria-label={`Open package ${entry.package.name}`}
          >
            Open
          </ActionButton>
          <ActionButton
            onClick={() => onRefresh(entry.package.name)}
            icon={faRotate}
            aria-label={`Refresh package ${entry.package.name}`}
          >
            Refresh
          </ActionButton>
          <ActionButton
            onClick={() => onRebuild(entry.package.name)}
            icon={faHammer}
            aria-label={`Rebuild package ${entry.package.name}`}
          >
            Rebuild
          </ActionButton>
          <ActionButton
            onClick={() => onDelete(entry.package.name)}
            icon={faTrash}
            aria-label={`Delete package ${entry.package.name}`}
            className="text-zinc-300"
          >
            Delete
          </ActionButton>
        </div>
      </div>

      <div className="px-5 py-4">
        <div className="mb-3 font-mono text-[11px] uppercase tracking-[0.18em] text-zinc-500">
          Target State
        </div>
        <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-3">
          {entry.state.targets.map((target) => (
            <div
              key={`${entry.package.name}:${target.mock_chroot}`}
              className="border-2 border-zinc-700 bg-zinc-950/40 px-4 py-3"
            >
              <div className="flex items-start justify-between gap-3">
                <div className="min-w-0">
                  <div className="font-mono text-sm text-zinc-100">
                    {target.mock_chroot}
                  </div>
                  <div className="mt-1 text-xs text-zinc-500">
                    {target.active_job_id
                      ? `Active job ${target.active_job_id}`
                      : target.last_successful_build_id
                        ? `Last success ${target.last_successful_build_id}`
                        : "No successful build yet"}
                  </div>
                </div>
                <StatusPill status={targetStatus(target)} />
              </div>
              <div className="mt-3 border-t-2 border-zinc-700 pt-3">
                <div className="font-mono text-[11px] uppercase tracking-[0.18em] text-zinc-500">
                  Revision
                </div>
                <div className="mt-2 break-all font-mono text-sm text-zinc-400">
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
