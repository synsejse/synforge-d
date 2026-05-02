import type { PackageBuildInventoryEntry } from "../../../lib/types";
import PaginationControls from "../../../components/common/pagination-controls";
import EmptyState from "../../../components/ui/empty-state";
import LoadingBlock from "../../../components/ui/loading-block";
import BuildHistoryCard from "./build-history-card";

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
  if (buildsLoading && !buildsLoaded) {
    return <LoadingBlock label="Loading build history…" lines={4} />;
  }
  if (builds.length === 0) {
    return <EmptyState>No build history yet.</EmptyState>;
  }

  return (
    <div className="space-y-4">
      <div className="space-y-3">
        {builds.map((entry) => (
          <BuildHistoryCard
            key={entry.build.job.id}
            entry={entry}
            deleting={deletingJobId === entry.build.job.id}
            onRefreshTarget={onRefreshTarget}
            onRebuildTarget={onRebuildTarget}
            onDeleteJob={onDeleteJob}
          />
        ))}
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
  );
}
