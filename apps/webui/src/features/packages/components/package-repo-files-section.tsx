import type { PublishedRepoFile } from "../../../lib/types";
import PaginationControls from "../../../components/common/pagination-controls";
import EmptyState from "../../../components/ui/empty-state";
import { SkeletonListRow } from "../../../components/ui/skeleton";
import RepoFileCard from "../../repository/components/repo-file-card";

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
  const loading = repoFilesLoading && !repoFilesLoaded;

  return (
    <div className="space-y-4">
      {loading ? (
        <div className="space-y-3">
          {Array.from({ length: 4 }).map((_, i) => (
            <SkeletonListRow key={i} />
          ))}
        </div>
      ) : repoFiles.length === 0 ? (
        <EmptyState>
          No repo files are currently tracked for this package.
        </EmptyState>
      ) : (
        <div className="space-y-3">
          {repoFiles.map((file) => (
            <RepoFileCard key={`${file.job_id}:${file.path}`} file={file} />
          ))}
        </div>
      )}
      {!loading && repoFiles.length > 0 ? (
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
      ) : null}
    </div>
  );
}
