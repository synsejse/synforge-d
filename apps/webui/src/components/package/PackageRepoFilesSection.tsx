import { faBoxesStacked } from "@fortawesome/free-solid-svg-icons";
import { formatBytes } from "../../lib/bytes";
import { formatDateTime } from "../../lib/datetime";
import type { PublishedRepoFile } from "../../lib/types";
import PaginationControls from "../common/PaginationControls";
import EmptyState from "../ui/EmptyState";
import FaIcon from "../ui/FaIcon";
import LoadingBlock from "../ui/LoadingBlock";

interface PackageRepoFilesSectionProps {
  repoFilesLoaded: boolean;
  repoFilesTotal: number | null;
  repoFilesOpen: boolean;
  onToggleOpen: () => void;
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
  repoFilesOpen,
  onToggleOpen,
  repoFilesLoading,
  repoFiles,
  repoFilesOffset,
  repoFilesHasMore,
  onLoadPrevious,
  onLoadNext,
}: PackageRepoFilesSectionProps) {
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
        <div className="flex items-center gap-3">
          {repoFilesLoaded ? (
            <div className="border-2 border-zinc-700 bg-black px-4 py-2 font-mono text-xs uppercase tracking-[0.2em] text-zinc-400">
              {repoFilesTotal ?? repoFiles.length} tracked files
            </div>
          ) : null}
          <button
            type="button"
            onClick={onToggleOpen}
            className="border-2 border-zinc-700 bg-black px-4 py-2 font-mono text-xs font-bold uppercase tracking-[0.15em] text-zinc-300 transition duration-100 ease-linear hover:-translate-x-[1px] hover:-translate-y-[1px] hover:border-white hover:bg-zinc-950"
          >
            {repoFilesOpen ? "Hide Files" : "Open Files"}
          </button>
        </div>
      </div>

      {!repoFilesOpen ? (
        <EmptyState>
          Open repository files to load the repo-owned outputs for this package.
        </EmptyState>
      ) : repoFilesLoading && !repoFilesLoaded ? (
        <LoadingBlock label="Loading repository files…" lines={4} />
      ) : repoFiles.length === 0 ? (
        <EmptyState>
          No repo files are currently tracked for this package.
        </EmptyState>
      ) : (
        <div className="space-y-4">
          <div className="overflow-x-auto border-2 border-zinc-700">
            <table className="min-w-[980px] w-full">
              <thead className="border-b-2 border-zinc-700 bg-zinc-950 text-left font-mono text-xs uppercase tracking-[0.2em] text-zinc-500">
                <tr>
                  <th className="px-4 py-3">Repo Path</th>
                  <th className="px-4 py-3">Build</th>
                  <th className="px-4 py-3">Kind</th>
                  <th className="px-4 py-3">Size</th>
                  <th className="px-4 py-3">Published</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-zinc-800 bg-black">
                {repoFiles.map((file) => (
                  <tr key={`${file.job_id}:${file.path}`} className="hover:bg-zinc-950">
                    <td className="px-4 py-3 font-mono text-sm text-zinc-200">
                      {file.path}
                    </td>
                    <td className="px-4 py-3">
                      <a
                        href={`/jobs/view/?id=${encodeURIComponent(file.job_id)}`}
                        className="font-mono text-sm text-zinc-300 transition hover:text-[var(--theme-accent-lime)]"
                      >
                        <FaIcon icon={faBoxesStacked} className="mr-2" />
                        {file.job_id}
                      </a>
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
                  </tr>
                ))}
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
