import { faBoxesStacked } from "@fortawesome/free-solid-svg-icons";
import { Link } from "@tanstack/react-router";
import { formatBytes } from "../../../lib/bytes";
import { formatDateTime } from "../../../lib/datetime";
import type { PublishedRepoFile } from "../../../lib/types";
import PaginationControls from "../../../components/common/pagination-controls";
import EmptyState from "../../../components/ui/empty-state";
import FaIcon from "../../../components/ui/fa-icon";
import LoadingBlock from "../../../components/ui/loading-block";
import Badge from "../../../components/ui/badge";

interface PackageRepoFilesSectionProps {
  repoFilesLoaded: boolean;
  repoFilesTotal: number | null;
  repoFilesLoading: boolean;
  repoFiles: PublishedRepoFile[];
  repoFilesOffset: number;
  repoFilesHasMore: boolean;
  onLoadPrevious: () => void;
  onLoadNext: () => void;
}

export default function PackageRepoFilesSection({
  repoFilesLoaded,
  repoFilesTotal,
  repoFilesLoading,
  repoFiles,
  repoFilesOffset,
  repoFilesHasMore,
  onLoadPrevious,
  onLoadNext,
}: PackageRepoFilesSectionProps) {
  const getSigningState = (file: PublishedRepoFile) => {
    if (file.signing_status === "signed") {
      return { label: "SIGNED", variant: "success" as const };
    }
    if (file.signing_status === "failed") {
      return {
        label: "SIGN FAILED",
        variant: "error" as const,
        title: file.signing_error_message || "Artifact signing failed",
      };
    }
    return { label: "NOT SIGNED", variant: "warning" as const };
  };

  return (
    <section className="border-4 border-white bg-black p-6 shadow-[6px_6px_0_rgba(255,255,255,0.2)]">
      <div className="mb-5 flex flex-col gap-2 md:flex-row md:items-end md:justify-between">
        <div>
          <h2 className="font-mono text-xl font-bold uppercase text-white">Repository Files</h2>
          <p className="mt-2 text-sm text-zinc-400">
            Build-owned files currently present in the repo namespace for this
            package.
          </p>
        </div>
        {repoFilesLoaded ? (
          <div className="border-2 border-zinc-700 bg-black px-4 py-2 font-mono text-xs uppercase tracking-[0.2em] text-zinc-400">
            {repoFilesTotal ?? repoFiles.length} tracked files
          </div>
        ) : null}
      </div>

      {repoFilesLoading && !repoFilesLoaded ? (
        <LoadingBlock label="Loading repository files…" lines={4} />
      ) : repoFiles.length === 0 ? (
        <EmptyState>
          No repo files are currently tracked for this package.
        </EmptyState>
      ) : (
        <div className="space-y-4">
          <div className="space-y-3 md:hidden">
            {repoFiles.map((file) => {
              const signingState = getSigningState(file);
              return (
                <article
                  key={`mobile:${file.job_id}:${file.path}`}
                  className="border-2 border-zinc-700 bg-black p-4"
                >
                  <div className="flex items-start justify-between gap-3">
                    <div className="min-w-0">
                      <div className="break-all font-mono text-sm text-zinc-200">
                        {file.path}
                      </div>
                      <div className="mt-1 font-mono text-xs uppercase tracking-[0.18em] text-zinc-500">
                        {file.kind}
                      </div>
                    </div>
                    <Badge variant={signingState.variant} title={signingState.title}>
                      {signingState.label}
                    </Badge>
                  </div>
                  <div className="mt-3 space-y-2 font-mono text-xs text-zinc-400">
                    <div>
                      <span className="text-zinc-500">Build:</span>{" "}
                      <Link
                        to="/jobs/view"
                        search={{ id: file.job_id }}
                        className="break-all text-zinc-300 transition hover:text-[var(--theme-accent-lime)]"
                      >
                        <FaIcon icon={faBoxesStacked} className="mr-2" />
                        {file.job_id}
                      </Link>
                    </div>
                    <div>
                      <span className="text-zinc-500">Size:</span>{" "}
                      {formatBytes(file.size_bytes)}
                    </div>
                    <div>
                      <span className="text-zinc-500">Published:</span>{" "}
                      {formatDateTime(file.published_at)}
                    </div>
                  </div>
                </article>
              );
            })}
          </div>
          <div className="hidden overflow-x-auto border-2 border-zinc-700 md:block">
            <table className="w-full min-w-[640px] lg:min-w-[980px]">
              <thead className="border-b-2 border-zinc-700 bg-zinc-950 text-left font-mono text-xs uppercase tracking-[0.2em] text-zinc-500">
                <tr>
                  <th className="px-4 py-3">Repo Path</th>
                  <th className="px-4 py-3">Build</th>
                  <th className="px-4 py-3">Kind</th>
                  <th className="px-4 py-3">Size</th>
                  <th className="px-4 py-3">Published</th>
                  <th className="px-4 py-3">Signing</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-zinc-800 bg-black">
                {repoFiles.map((file) => {
                  const signingState = getSigningState(file);
                  return (
                    <tr key={`${file.job_id}:${file.path}`} className="hover:bg-zinc-950">
                      <td className="px-4 py-3 font-mono text-sm text-zinc-200">
                        {file.path}
                      </td>
                      <td className="px-4 py-3">
                        <Link
                          to="/jobs/view"
                          search={{ id: file.job_id }}
                          className="font-mono text-sm text-zinc-300 transition hover:text-[var(--theme-accent-lime)]"
                        >
                          <FaIcon icon={faBoxesStacked} className="mr-2" />
                          {file.job_id}
                        </Link>
                      </td>
                      <td className="px-4 py-3 font-mono text-xs uppercase tracking-[0.18em] text-zinc-500">
                        {file.kind}
                      </td>
                      <td className="px-4 py-3 font-mono text-sm text-zinc-400">
                        {formatBytes(file.size_bytes)}
                      </td>
                      <td className="px-4 py-3 font-mono text-sm text-zinc-400">
                        {formatDateTime(file.published_at)}
                      </td>
                      <td className="px-4 py-3">
                        <Badge variant={signingState.variant} title={signingState.title}>
                          {signingState.label}
                        </Badge>
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
          <div className="border-2 border-zinc-700 bg-black px-4 py-3">
            <PaginationControls
              onPrevious={onLoadPrevious}
              onNext={onLoadNext}
              previousDisabled={repoFilesLoading || repoFilesOffset === 0}
              nextDisabled={repoFilesLoading || !repoFilesHasMore}
              summary={
                <>
                  Showing {repoFilesOffset + 1}-{repoFilesOffset + repoFiles.length}
                  {repoFilesTotal !== null ? ` of ${repoFilesTotal}` : ""}
                </>
              }
            />
          </div>
        </div>
      )}
    </section>
  );
}
