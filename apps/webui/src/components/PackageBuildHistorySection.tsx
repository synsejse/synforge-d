import {
  faFolderOpen,
  faHammer,
  faRotate,
  faTrash,
} from "@fortawesome/free-solid-svg-icons";
import { formatDateTime } from "../lib/datetime";
import type { PackageBuildInventoryEntry } from "../lib/types";
import EmptyState from "./EmptyState";
import FaIcon from "./FaIcon";
import LoadingBlock from "./LoadingBlock";
import StatusPill from "./StatusPill";

interface PackageBuildHistorySectionProps {
  buildsLoaded: boolean;
  buildsTotal: number | null;
  buildsOpen: boolean;
  onToggleOpen: () => void;
  buildsLoading: boolean;
  builds: PackageBuildInventoryEntry[];
  buildsOffset: number;
  buildsHasMore: boolean;
  onLoadPrevious: () => void;
  onLoadNext: () => void;
  onRefreshTarget: (mockChroot: string) => void;
  onRebuildTarget: (mockChroot: string) => void;
  onDeleteJob: (jobId: string) => void;
  deletingJobId: string | null;
}

export default function PackageBuildHistorySection({
  buildsLoaded,
  buildsTotal,
  buildsOpen,
  onToggleOpen,
  buildsLoading,
  builds,
  buildsOffset,
  buildsHasMore,
  onLoadPrevious,
  onLoadNext,
  onRefreshTarget,
  onRebuildTarget,
  onDeleteJob,
  deletingJobId,
}: PackageBuildHistorySectionProps) {
  return (
    <section className="border border-zinc-800 bg-black p-6">
      <div className="mb-5 flex flex-col gap-2 md:flex-row md:items-end md:justify-between">
        <div>
          <h2 className="text-xl font-semibold text-white">Build History</h2>
          <p className="mt-2 text-sm text-zinc-400">
            Build activity for this package, including revisions, outcomes, and
            managed repo ownership.
          </p>
        </div>
        <div className="flex items-center gap-3">
          {buildsLoaded ? (
            <div className="border border-zinc-800 bg-black px-4 py-2 text-xs uppercase tracking-[0.2em] text-zinc-400">
              {buildsTotal ?? builds.length} total builds
            </div>
          ) : null}
          <button
            type="button"
            onClick={onToggleOpen}
            className="border border-zinc-800 bg-black px-4 py-2 text-sm text-zinc-300 transition hover:border-zinc-600 hover:bg-zinc-950"
          >
            {buildsOpen ? "Hide History" : "Open History"}
          </button>
        </div>
      </div>

      {!buildsOpen ? (
        <EmptyState>Open build history to load recent package builds.</EmptyState>
      ) : buildsLoading && !buildsLoaded ? (
        <LoadingBlock label="Loading build history…" lines={4} />
      ) : builds.length === 0 ? (
        <EmptyState>No build history yet.</EmptyState>
      ) : (
        <div className="space-y-4">
          <div className="overflow-hidden border border-zinc-800">
            <table className="w-full">
              <thead className="bg-zinc-950 text-left text-xs uppercase tracking-[0.2em] text-zinc-500">
                <tr>
                  <th className="px-4 py-3">Target</th>
                  <th className="px-4 py-3">Revision</th>
                  <th className="px-4 py-3">Status</th>
                  <th className="px-4 py-3">Trigger</th>
                  <th className="px-4 py-3">Created</th>
                  <th className="px-4 py-3">Repo Files</th>
                  <th className="px-4 py-3">Actions</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-white/10 bg-black">
                {builds.map((entry) => {
                  const publishedFiles = entry.repo_files;
                  const live =
                    entry.build.job.status === "pending" ||
                    entry.build.job.status === "running";
                  return (
                    <tr key={entry.build.job.id} className="hover:bg-zinc-950">
                      <td className="px-4 py-3 text-sm font-mono text-zinc-300">
                        {entry.build.job.mock_chroot}
                      </td>
                      <td className="px-4 py-3">
                        <div className="font-mono text-sm text-zinc-200">
                          {entry.build.job.revision}
                        </div>
                        <div className="mt-1 text-xs text-zinc-500">
                          {entry.build.job.id}
                        </div>
                      </td>
                      <td className="px-4 py-3">
                        <StatusPill status={entry.build.job.status} />
                      </td>
                      <td className="px-4 py-3 text-sm text-zinc-400">
                        {entry.build.job.trigger}
                      </td>
                      <td className="px-4 py-3 text-sm text-zinc-400">
                        {formatDateTime(entry.build.job.created_at)}
                      </td>
                      <td className="px-4 py-3 text-sm text-zinc-300">
                        {publishedFiles.length}
                      </td>
                      <td className="px-4 py-3">
                        <div className="flex flex-wrap gap-3">
                          <a
                            href={`/jobs/view/?id=${encodeURIComponent(entry.build.job.id)}`}
                            className="text-sm font-medium text-zinc-300 transition hover:text-white"
                          >
                            <FaIcon icon={faFolderOpen} className="mr-2" />
                            Open Job
                          </a>
                          {!live ? (
                            <>
                              <button
                                onClick={() =>
                                  onRefreshTarget(entry.build.job.mock_chroot)
                                }
                                className="text-sm font-medium text-zinc-500 transition hover:text-white"
                              >
                                <FaIcon icon={faRotate} className="mr-2" />
                                Refresh Target
                              </button>
                              <button
                                onClick={() =>
                                  onRebuildTarget(entry.build.job.mock_chroot)
                                }
                                className="text-sm font-medium text-zinc-500 transition hover:text-white"
                              >
                                <FaIcon icon={faHammer} className="mr-2" />
                                Rebuild Target
                              </button>
                            </>
                          ) : null}
                          <button
                            onClick={() => onDeleteJob(entry.build.job.id)}
                            disabled={live || deletingJobId === entry.build.job.id}
                            className="text-sm font-medium text-zinc-500 transition hover:text-white disabled:cursor-not-allowed disabled:opacity-50"
                          >
                            <FaIcon icon={faTrash} className="mr-2" />
                            {deletingJobId === entry.build.job.id
                              ? "Deleting…"
                              : "Delete Build"}
                          </button>
                        </div>
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
          <div className="flex items-center justify-between border border-zinc-800 bg-black px-4 py-3 text-sm text-zinc-400">
            <span>
              Showing {buildsOffset + 1}-{buildsOffset + builds.length}
              {buildsTotal !== null ? ` of ${buildsTotal}` : ""}
            </span>
            <div className="flex items-center gap-3">
              <button
                type="button"
                onClick={onLoadPrevious}
                disabled={buildsLoading || buildsOffset === 0}
                className="border border-zinc-800 bg-black px-4 py-2 text-sm text-zinc-300 transition hover:border-zinc-600 hover:bg-zinc-950 disabled:cursor-not-allowed disabled:opacity-40"
              >
                Previous
              </button>
              <button
                type="button"
                onClick={onLoadNext}
                disabled={buildsLoading || !buildsHasMore}
                className="border border-zinc-800 bg-black px-4 py-2 text-sm text-zinc-300 transition hover:border-zinc-600 hover:bg-zinc-950 disabled:cursor-not-allowed disabled:opacity-40"
              >
                Next
              </button>
            </div>
          </div>
        </div>
      )}
    </section>
  );
}
