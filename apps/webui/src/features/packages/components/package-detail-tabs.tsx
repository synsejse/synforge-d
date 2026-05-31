import Tabs from "../../../components/ui/tabs";
import PackageBuildHistorySection from "./package-build-history-section";
import PackageRepoFilesSection from "./package-repo-files-section";
import SyncHistoryTable from "./sync-history-table";
import type {
  PackageBuildInventoryEntry,
  PublishedRepoFile,
} from "../../../lib/types";

export type PackageDetailTab = "builds" | "repo" | "sync";

interface PackageDetailTabsProps {
  packageName: string;
  activeTab: PackageDetailTab;
  onTabChange: (tab: PackageDetailTab) => void;

  buildsLoaded: boolean;
  buildsTotal: number | null;
  buildsLoading: boolean;
  builds: PackageBuildInventoryEntry[];
  buildsOffset: number;
  buildsHasMore: boolean;
  includeDeleted: boolean;
  deletingJobId: string | null;
  onIncludeDeletedChange: (next: boolean) => void;
  onLoadPreviousBuilds: () => void;
  onLoadNextBuilds: () => void;
  onRefreshTarget: (mockChroot: string) => void;
  onRebuildTarget: (mockChroot: string) => void;
  onDeleteJob: (jobId: string) => void;

  repoFilesLoaded: boolean;
  repoFilesTotal: number | null;
  repoFilesLoading: boolean;
  repoFiles: PublishedRepoFile[];
  repoFilesOffset: number;
  repoFilesHasMore: boolean;
  onLoadPreviousRepoFiles: () => void;
  onLoadNextRepoFiles: () => void;
}

export default function PackageDetailTabs({
  packageName,
  activeTab,
  onTabChange,
  buildsLoaded,
  buildsTotal,
  buildsLoading,
  builds,
  buildsOffset,
  buildsHasMore,
  includeDeleted,
  deletingJobId,
  onIncludeDeletedChange,
  onLoadPreviousBuilds,
  onLoadNextBuilds,
  onRefreshTarget,
  onRebuildTarget,
  onDeleteJob,
  repoFilesLoaded,
  repoFilesTotal,
  repoFilesLoading,
  repoFiles,
  repoFilesOffset,
  repoFilesHasMore,
  onLoadPreviousRepoFiles,
  onLoadNextRepoFiles,
}: PackageDetailTabsProps) {
  return (
    <Tabs
      ariaLabel="Package detail sections"
      value={activeTab}
      onChange={onTabChange}
      items={[
        { value: "builds", label: "Build History", count: buildsTotal },
        { value: "repo", label: "Repository Files", count: repoFilesTotal },
        { value: "sync", label: "Sync History" },
      ]}
    >
      {activeTab === "builds" ? (
        <PackageBuildHistorySection
          buildsLoaded={buildsLoaded}
          buildsTotal={buildsTotal}
          buildsLoading={buildsLoading}
          builds={builds}
          buildsOffset={buildsOffset}
          buildsHasMore={buildsHasMore}
          includeDeleted={includeDeleted}
          onIncludeDeletedChange={onIncludeDeletedChange}
          onLoadPrevious={onLoadPreviousBuilds}
          onLoadNext={onLoadNextBuilds}
          onRefreshTarget={onRefreshTarget}
          onRebuildTarget={onRebuildTarget}
          onDeleteJob={onDeleteJob}
          deletingJobId={deletingJobId}
        />
      ) : null}
      {activeTab === "repo" ? (
        <PackageRepoFilesSection
          repoFilesLoaded={repoFilesLoaded}
          repoFilesTotal={repoFilesTotal}
          repoFilesLoading={repoFilesLoading}
          repoFiles={repoFiles}
          repoFilesOffset={repoFilesOffset}
          repoFilesHasMore={repoFilesHasMore}
          onLoadPrevious={onLoadPreviousRepoFiles}
          onLoadNext={onLoadNextRepoFiles}
        />
      ) : null}
      {activeTab === "sync" ? (
        <SyncHistoryTable packageName={packageName} />
      ) : null}
    </Tabs>
  );
}
