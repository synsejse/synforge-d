import type { PublishedRepoFile } from "../../../lib/types";
import PaginationControls from "../../../components/common/pagination-controls";
import EmptyState from "../../../components/ui/empty-state";
import LoadingBlock from "../../../components/ui/loading-block";
import RepoFileCard from "../../repository/components/repo-file-card";

interface PackageRepoFilesSectionProps {
  repoFilesLoaded: boolean;
  repoFilesTotal: number | null;
  repoFilesLoading: boolean;
  repoFiles: PublishedRepoFile[];
  repoFilesOffset: number;
  repoFilesPageSize: number;
  repoFilesHasMore: boolean;
  onOffsetChange: (offset: number) => void;
}

export default function PackageRepoFilesSection({
  repoFilesLoaded,
  repoFilesTotal,
  repoFilesLoading,
  repoFiles,
  repoFilesOffset,
  repoFilesPageSize,
  repoFilesHasMore,
  onOffsetChange,
}: PackageRepoFilesSectionProps) {
  const loading = repoFilesLoading && !repoFilesLoaded;

  return (
    <div className="space-y-4">
      {loading ? (
        <LoadingBlock label="Loading repository files…" lines={3} />
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
        <PaginationControls
          offset={repoFilesOffset}
          pageSize={repoFilesPageSize}
          count={repoFiles.length}
          hasMore={repoFilesHasMore}
          total={repoFilesTotal}
          isFetching={repoFilesLoading}
          onOffsetChange={onOffsetChange}
        />
      ) : null}
    </div>
  );
}
