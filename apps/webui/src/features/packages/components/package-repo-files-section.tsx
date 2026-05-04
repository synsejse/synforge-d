import type { PublishedRepoFile } from "../../../lib/types";
import PaginationControls from "../../../components/common/pagination-controls";
import EmptyState from "../../../components/ui/empty-state";
import { SkeletonCardList } from "../../../components/ui/skeleton";
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
  if (repoFilesLoading && !repoFilesLoaded) {
    return <SkeletonCardList count={4} lines={2} />;
  }
  if (repoFiles.length === 0) {
    return (
      <EmptyState>
        No repo files are currently tracked for this package.
      </EmptyState>
    );
  }

  return (
    <div className="space-y-4">
      <div className="space-y-3">
        {repoFiles.map((file) => (
          <RepoFileCard key={`${file.job_id}:${file.path}`} file={file} />
        ))}
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
  );
}
