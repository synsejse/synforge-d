import type { ReactNode } from "react";
import Tabs from "../../../components/ui/tabs";
import PackageBuildHistorySection from "./package-build-history-section";
import PackageRepoFilesSection from "./package-repo-files-section";
import SyncHistoryTable from "./sync-history-table";
import type {
  PackageBuildInventoryEntry,
  PackageTargetCcacheStats,
  PublishedRepoFile,
} from "../../../lib/types";

export type PackageDetailTab = "builds" | "repo" | "sync" | "configuration";

interface PackageDetailTabsProps {
  packageName: string;
  activeTab: PackageDetailTab;
  onTabChange: (tab: PackageDetailTab) => void;
  configuration: ReactNode;
  configurationDirty: boolean;

  buildsLoaded: boolean;
  buildsTotal: number | null;
  buildsLoading: boolean;
  builds: PackageBuildInventoryEntry[];
  buildsOffset: number;
  buildsPageSize: number;
  buildsHasMore: boolean;
  includeDeleted: boolean;
  deletingJobId: string | null;
  ccacheEnabled: boolean;
  ccacheStatsByTarget: PackageTargetCcacheStats[];
  onIncludeDeletedChange: (next: boolean) => void;
  onBuildsOffsetChange: (offset: number) => void;
  onRefreshTarget: (mockChroot: string) => void;
  onRebuildTarget: (mockChroot: string) => void;
  onDeleteJob: (jobId: string) => void;

  repoFilesLoaded: boolean;
  repoFilesTotal: number | null;
  repoFilesLoading: boolean;
  repoFiles: PublishedRepoFile[];
  repoFilesOffset: number;
  repoFilesPageSize: number;
  repoFilesHasMore: boolean;
  onRepoFilesOffsetChange: (offset: number) => void;
}

export default function PackageDetailTabs({
  packageName,
  activeTab,
  onTabChange,
  configuration,
  configurationDirty,
  buildsLoaded,
  buildsTotal,
  buildsLoading,
  builds,
  buildsOffset,
  buildsPageSize,
  buildsHasMore,
  includeDeleted,
  deletingJobId,
  ccacheEnabled,
  ccacheStatsByTarget,
  onIncludeDeletedChange,
  onBuildsOffsetChange,
  onRefreshTarget,
  onRebuildTarget,
  onDeleteJob,
  repoFilesLoaded,
  repoFilesTotal,
  repoFilesLoading,
  repoFiles,
  repoFilesOffset,
  repoFilesPageSize,
  repoFilesHasMore,
  onRepoFilesOffsetChange,
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
        {
          value: "configuration",
          label: configurationDirty ? "Configuration •" : "Configuration",
        },
      ]}
    >
      {activeTab === "builds" ? (
        <PackageBuildHistorySection
          buildsLoaded={buildsLoaded}
          buildsTotal={buildsTotal}
          buildsLoading={buildsLoading}
          builds={builds}
          buildsOffset={buildsOffset}
          buildsPageSize={buildsPageSize}
          buildsHasMore={buildsHasMore}
          includeDeleted={includeDeleted}
          onIncludeDeletedChange={onIncludeDeletedChange}
          onOffsetChange={onBuildsOffsetChange}
          onRefreshTarget={onRefreshTarget}
          onRebuildTarget={onRebuildTarget}
          onDeleteJob={onDeleteJob}
          deletingJobId={deletingJobId}
          ccacheEnabled={ccacheEnabled}
          ccacheStatsByTarget={ccacheStatsByTarget}
        />
      ) : null}
      {activeTab === "repo" ? (
        <PackageRepoFilesSection
          repoFilesLoaded={repoFilesLoaded}
          repoFilesTotal={repoFilesTotal}
          repoFilesLoading={repoFilesLoading}
          repoFiles={repoFiles}
          repoFilesOffset={repoFilesOffset}
          repoFilesPageSize={repoFilesPageSize}
          repoFilesHasMore={repoFilesHasMore}
          onOffsetChange={onRepoFilesOffsetChange}
        />
      ) : null}
      {activeTab === "sync" ? (
        <SyncHistoryTable packageName={packageName} />
      ) : null}
      {activeTab === "configuration" ? configuration : null}
    </Tabs>
  );
}
