import {
  faFolderOpen,
  faHammer,
  faRotate,
  faTrash,
} from "@fortawesome/free-solid-svg-icons";
import { Link } from "@tanstack/react-router";
import { formatDateTime } from "../../../lib/datetime";
import type { PackageBuildInventoryEntry } from "../../../lib/types";
import PaginationControls from "../../../components/common/pagination-controls";
import EmptyState from "../../../components/ui/empty-state";
import FaIcon from "../../../components/ui/fa-icon";
import LoadingBlock from "../../../components/ui/loading-block";
import StatusPill from "../../../components/ui/status-pill";
import Badge from "../../../components/ui/badge";

interface PackageBuildHistorySectionProps {
  buildsLoaded: boolean;
  buildsTotal: number | null;
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
  const getBuildSigningSummary = (entry: PackageBuildInventoryEntry) => {
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
  };

  return (
    <section className="border-4 border-white bg-black p-6 shadow-card-md">
      <div className="mb-5 flex flex-col gap-2 md:flex-row md:items-end md:justify-between">
        <div>
          <h2 className="font-mono text-xl font-bold uppercase text-white">Build History</h2>
          <p className="mt-2 text-sm text-muted">
            Build activity for this package, including revisions, outcomes, and
            managed repo ownership.
          </p>
        </div>
        {buildsLoaded ? (
          <div className="border-2 border-edge-strong bg-black px-4 py-2 font-mono text-xs uppercase tracking-[0.2em] text-muted">
            {buildsTotal ?? builds.length} total builds
          </div>
        ) : null}
      </div>

      {buildsLoading && !buildsLoaded ? (
        <LoadingBlock label="Loading build history…" lines={4} />
      ) : builds.length === 0 ? (
        <EmptyState>No build history yet.</EmptyState>
      ) : (
        <div className="space-y-4">
          <div className="space-y-3 md:hidden">
            {builds.map((entry) => {
              const publishedFiles = entry.repo_files;
              const signingSummary = getBuildSigningSummary(entry);
              const live =
                entry.build.job.status === "pending" ||
                entry.build.job.status === "running";
              return (
                <article
                  key={`mobile:${entry.build.job.id}`}
                  className="border-2 border-edge-strong bg-black p-4"
                >
                  <div className="flex items-start justify-between gap-3">
                    <div className="min-w-0">
                      <Link
                        to="/jobs/view"
                        search={{ id: entry.build.job.id }}
                        className="font-mono text-sm font-bold uppercase text-white transition duration-100 ease-linear hover:text-accent-lime"
                      >
                        {entry.build.job.mock_chroot}
                      </Link>
                      <div className="mt-1 break-all font-mono text-xs text-soft">
                        {entry.build.job.id}
                      </div>
                    </div>
                    <StatusPill status={entry.build.job.status} />
                  </div>
                  <div className="mt-4 space-y-2 text-sm">
                    <div className="font-mono text-muted">
                      <span className="text-soft">Revision:</span>{" "}
                      <span className="break-all">{entry.build.job.revision}</span>
                    </div>
                    <div className="font-mono text-muted">
                      <span className="text-soft">Trigger:</span> {entry.build.job.trigger}
                    </div>
                    <div className="font-mono text-muted">
                      <span className="text-soft">Created:</span>{" "}
                      {formatDateTime(entry.build.job.created_at)}
                    </div>
                    <div className="flex items-center gap-3">
                      <span className="font-mono text-muted">
                        <span className="text-soft">Repo files:</span> {publishedFiles.length}
                      </span>
                      <Badge variant={signingSummary.variant}>
                        {signingSummary.label}
                      </Badge>
                    </div>
                  </div>
                  <div className="mt-4 grid gap-2">
                    <Link
                      to="/jobs/view"
                        search={{ id: entry.build.job.id }}
                      className="font-mono text-xs font-bold uppercase tracking-[0.1em] text-muted transition-all duration-100 ease-linear hover:text-accent-lime"
                    >
                      <FaIcon icon={faFolderOpen} className="mr-2" />
                      Open Job
                    </Link>
                    {!live ? (
                      <>
                        <button
                          onClick={() => onRefreshTarget(entry.build.job.mock_chroot)}
                          className="text-left font-mono text-xs font-bold uppercase tracking-[0.1em] text-soft transition-all duration-100 ease-linear hover:text-white"
                        >
                          <FaIcon icon={faRotate} className="mr-2" />
                          Refresh Target
                        </button>
                        <button
                          onClick={() => onRebuildTarget(entry.build.job.mock_chroot)}
                          className="text-left font-mono text-xs font-bold uppercase tracking-[0.1em] text-soft transition-all duration-100 ease-linear hover:text-white"
                        >
                          <FaIcon icon={faHammer} className="mr-2" />
                          Rebuild Target
                        </button>
                      </>
                    ) : null}
                    <button
                      onClick={() => onDeleteJob(entry.build.job.id)}
                      disabled={live || deletingJobId === entry.build.job.id}
                      className="text-left font-mono text-xs font-bold uppercase tracking-[0.1em] text-soft transition-all duration-100 ease-linear hover:text-white disabled:cursor-not-allowed disabled:opacity-50"
                    >
                      <FaIcon icon={faTrash} className="mr-2" />
                      {deletingJobId === entry.build.job.id ? "Deleting…" : "Delete Build"}
                    </button>
                  </div>
                </article>
              );
            })}
          </div>
          <div className="hidden overflow-hidden border-2 border-edge-strong md:block">
            <table className="w-full">
              <thead className="border-b-2 border-edge-strong bg-surface-alt text-left font-mono text-xs uppercase tracking-[0.2em] text-soft">
                <tr>
                  <th className="px-4 py-3">Target</th>
                  <th className="px-4 py-3">Revision</th>
                  <th className="px-4 py-3">Status</th>
                  <th className="px-4 py-3">Trigger</th>
                  <th className="px-4 py-3">Created</th>
                  <th className="px-4 py-3">Repo Files</th>
                  <th className="px-4 py-3">Signing</th>
                  <th className="px-4 py-3">Actions</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-edge bg-black">
                {builds.map((entry) => {
                  const publishedFiles = entry.repo_files;
                  const signingSummary = getBuildSigningSummary(entry);
                  const live =
                    entry.build.job.status === "pending" ||
                    entry.build.job.status === "running";
                  return (
                    <tr key={entry.build.job.id} className="hover:bg-surface-alt">
                      <td className="px-4 py-3 text-sm font-mono text-muted">
                        {entry.build.job.mock_chroot}
                      </td>
                      <td className="px-4 py-3">
                        <div className="font-mono text-sm text-strong">
                          {entry.build.job.revision}
                        </div>
                        <div className="mt-1 font-mono text-xs text-soft">
                          {entry.build.job.id}
                        </div>
                      </td>
                      <td className="px-4 py-3">
                        <StatusPill status={entry.build.job.status} />
                      </td>
                      <td className="px-4 py-3 font-mono text-sm text-muted">
                        {entry.build.job.trigger}
                      </td>
                      <td className="px-4 py-3 font-mono text-sm text-muted">
                        {formatDateTime(entry.build.job.created_at)}
                      </td>
                      <td className="px-4 py-3 font-mono text-sm text-muted">
                        {publishedFiles.length}
                      </td>
                      <td className="px-4 py-3">
                        <Badge variant={signingSummary.variant}>
                          {signingSummary.label}
                        </Badge>
                      </td>
                      <td className="px-4 py-3">
                        <div className="flex flex-wrap gap-3">
                          <Link
                            to="/jobs/view"
                        search={{ id: entry.build.job.id }}
                            className="font-mono text-xs font-bold uppercase tracking-[0.1em] text-muted transition-all duration-100 ease-linear hover:text-accent-lime"
                          >
                            <FaIcon icon={faFolderOpen} className="mr-2" />
                            Open Job
                          </Link>
                          {!live ? (
                            <>
                              <button
                                onClick={() =>
                                  onRefreshTarget(entry.build.job.mock_chroot)
                                }
                                className="font-mono text-xs font-bold uppercase tracking-[0.1em] text-soft transition-all duration-100 ease-linear hover:text-white"
                              >
                                <FaIcon icon={faRotate} className="mr-2" />
                                Refresh Target
                              </button>
                              <button
                                onClick={() =>
                                  onRebuildTarget(entry.build.job.mock_chroot)
                                }
                                className="font-mono text-xs font-bold uppercase tracking-[0.1em] text-soft transition-all duration-100 ease-linear hover:text-white"
                              >
                                <FaIcon icon={faHammer} className="mr-2" />
                                Rebuild Target
                              </button>
                            </>
                          ) : null}
                          <button
                            onClick={() => onDeleteJob(entry.build.job.id)}
                            disabled={live || deletingJobId === entry.build.job.id}
                            className="font-mono text-xs font-bold uppercase tracking-[0.1em] text-soft transition-all duration-100 ease-linear hover:text-white disabled:cursor-not-allowed disabled:opacity-50"
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
           <div className="border-2 border-edge-strong bg-black px-4 py-3">
            <PaginationControls
              onPrevious={onLoadPrevious}
              onNext={onLoadNext}
              previousDisabled={buildsLoading || buildsOffset === 0}
              nextDisabled={buildsLoading || !buildsHasMore}
              summary={
                <>
                  Showing {buildsOffset + 1}-{buildsOffset + builds.length}
                  {buildsTotal !== null ? ` of ${buildsTotal}` : ""}
                </>
              }
            />
          </div>
        </div>
      )}
    </section>
  );
}
