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
  includeDeleted: boolean;
  onIncludeDeletedChange: (next: boolean) => void;
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
  includeDeleted,
  onIncludeDeletedChange,
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
  const showDeletedToggle = (
    <label className="inline-flex cursor-pointer items-center gap-2 border-2 border-edge-strong bg-black px-3 py-2 font-mono text-xs font-bold uppercase tracking-[0.18em] text-soft transition-colors hover:border-accent-lime hover:text-strong">
      <input
        type="checkbox"
        className="h-4 w-4 cursor-pointer accent-accent-lime"
        checked={includeDeleted}
        onChange={(e) => onIncludeDeletedChange(e.target.checked)}
      />
      Show deleted
    </label>
  );

  if (builds.length === 0) {
    return (
      <div className="space-y-4">
        <div className="flex justify-end">{showDeletedToggle}</div>
        <EmptyState>
          {includeDeleted
            ? "No build history yet."
            : "No build history yet. Deleted builds may be hidden — toggle 'Show deleted' to include them."}
        </EmptyState>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <div className="flex justify-end">{showDeletedToggle}</div>
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
