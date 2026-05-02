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
    <>
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
                  className="border-2 border-edge-strong bg-black p-4"
                >
                  <div className="flex items-start justify-between gap-3">
                    <div className="min-w-0">
                      <div className="break-all font-mono text-sm text-strong">
                        {file.path}
                      </div>
                      <div className="mt-1 font-mono text-xs uppercase tracking-[0.18em] text-soft">
                        {file.kind}
                      </div>
                    </div>
                    <Badge variant={signingState.variant} title={signingState.title}>
                      {signingState.label}
                    </Badge>
                  </div>
                  <div className="mt-3 space-y-2 font-mono text-xs text-muted">
                    <div>
                      <span className="text-soft">Build:</span>{" "}
                      <Link
                        to="/jobs/view"
                        search={{ id: file.job_id }}
                        className="break-all text-muted transition hover:text-accent-lime"
                      >
                        <FaIcon icon={faBoxesStacked} className="mr-2" />
                        {file.job_id}
                      </Link>
                    </div>
                    <div>
                      <span className="text-soft">Size:</span>{" "}
                      {formatBytes(file.size_bytes)}
                    </div>
                    <div>
                      <span className="text-soft">Published:</span>{" "}
                      {formatDateTime(file.published_at)}
                    </div>
                  </div>
                </article>
              );
            })}
          </div>
          <div className="hidden overflow-x-auto border-2 border-edge-strong md:block">
            <table className="w-full min-w-[640px] lg:min-w-[980px]">
              <thead className="border-b-2 border-edge-strong bg-surface-alt text-left font-mono text-xs uppercase tracking-[0.2em] text-soft">
                <tr>
                  <th className="px-4 py-3">Repo Path</th>
                  <th className="px-4 py-3">Build</th>
                  <th className="px-4 py-3">Kind</th>
                  <th className="px-4 py-3">Size</th>
                  <th className="px-4 py-3">Published</th>
                  <th className="px-4 py-3">Signing</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-edge bg-black">
                {repoFiles.map((file) => {
                  const signingState = getSigningState(file);
                  return (
                    <tr key={`${file.job_id}:${file.path}`} className="hover:bg-surface-alt">
                      <td className="px-4 py-3 font-mono text-sm text-strong">
                        {file.path}
                      </td>
                      <td className="px-4 py-3">
                        <Link
                          to="/jobs/view"
                          search={{ id: file.job_id }}
                          className="font-mono text-sm text-muted transition hover:text-accent-lime"
                        >
                          <FaIcon icon={faBoxesStacked} className="mr-2" />
                          {file.job_id}
                        </Link>
                      </td>
                      <td className="px-4 py-3 font-mono text-xs uppercase tracking-[0.18em] text-soft">
                        {file.kind}
                      </td>
                      <td className="px-4 py-3 font-mono text-sm text-muted">
                        {formatBytes(file.size_bytes)}
                      </td>
                      <td className="px-4 py-3 font-mono text-sm text-muted">
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
          <div className="border-2 border-edge-strong bg-black px-4 py-3">
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
    </>
  );
}
