import { useEffect, useState } from "react";
import { useDebounce } from "../../lib/hooks/use-debounce";
import { useQuery } from "@tanstack/react-query";
import { getRouteApi } from "@tanstack/react-router";
import {
  faBoxesStacked,
  faBullseye,
  faHardDrive,
  faHammer,
  faPlus,
} from "@fortawesome/free-solid-svg-icons";
import { repositoryQueries } from "../../lib/queries";
import { formatBytes } from "../../lib/bytes";
import ErrorMessage from "../../components/common/error-message";
import LoadingBlock from "../../components/ui/loading-block";
import FaIcon from "../../components/ui/fa-icon";
import SegmentedControl from "../../components/ui/segmented-control";
import MetricCard from "../../components/ui/metric-card";
import PageHeader from "../../components/ui/page-header";
import PaginationControls from "../../components/common/pagination-controls";
import RepoFileCard from "./components/repo-file-card";
import FilterBar from "../../components/common/filter-bar";
import EmptyState from "../../components/ui/empty-state";
import Button from "../../components/ui/button";

const PAGE_SIZE = 50;

const route = getRouteApi("/_authed/repository/");

type KindFilter = "all" | "rpm" | "srpm" | "log";

function RepositoryBrowser() {
  const navigate = route.useNavigate();
  const search = route.useSearch();
  const filters = {
    offset: search.offset ?? 0,
    packageFilter: search.packageFilter ?? "",
    targetFilter: search.targetFilter ?? "",
    kindFilter: search.kindFilter ?? "all",
  };
  const [packageInput, setPackageInput] = useState(filters.packageFilter);
  const [targetInput, setTargetInput] = useState(filters.targetFilter);
  const debouncedPackage = useDebounce(packageInput, 250);
  const debouncedTarget = useDebounce(targetInput, 250);

  const setFilters = (update: Partial<typeof search>) =>
    navigate({ search: (prev) => ({ ...prev, ...update }) });

  const clearFilters = () => {
    setPackageInput("");
    setTargetInput("");
    setFilters({
      offset: 0,
      packageFilter: "",
      targetFilter: "",
      kindFilter: "all",
    });
  };

  useEffect(() => {
    if (debouncedPackage !== filters.packageFilter || debouncedTarget !== filters.targetFilter) {
      setFilters({
        offset: 0,
        packageFilter: debouncedPackage,
        targetFilter: debouncedTarget,
      });
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [debouncedPackage, debouncedTarget]);

  const summaryQuery = useQuery(repositoryQueries.summary());

  const inventoryQuery = useQuery(
    repositoryQueries.inventory({
      limit: PAGE_SIZE,
      offset: filters.offset,
      packageName: filters.packageFilter,
      mockChroot: filters.targetFilter,
      kind: filters.kindFilter,
    }),
  );

  const loadError = summaryQuery.error ?? inventoryQuery.error;
  if (loadError) {
    return (
      <ErrorMessage
        message={
          loadError instanceof Error
            ? loadError.message
            : "Failed to load repository inventory"
        }
      />
    );
  }

  const summaryLoading = summaryQuery.isPending;
  const inventoryLoading = inventoryQuery.isPending;
  const summary = summaryQuery.data;
  const inventory = inventoryQuery.data;
  const repoFiles = inventory?.repo_files ?? [];
  const activeFilterCount =
    Number(filters.packageFilter.trim().length > 0) +
    Number(filters.targetFilter.trim().length > 0) +
    Number(filters.kindFilter !== "all");
  const hasActiveFilters = activeFilterCount > 0;

  return (
    <div className="space-y-6">
      <PageHeader
        title="Repository"
        description="Published packages, builds, and files."
        color="lime"
        actions={[
          {
            to: "/repository/use",
            label: "Use Repository",
            icon: faPlus,
            variant: "primary",
          },
        ]}
      />

      {summaryLoading ? (
        <LoadingBlock label="Loading metrics…" lines={2} />
      ) : (
        <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
          <MetricCard
            label="Packages"
            value={summary?.package_count ?? 0}
            detail="Published package names"
            icon={<FaIcon icon={faBoxesStacked} />}
          />
          <MetricCard
            label="Targets"
            value={summary?.target_count ?? 0}
            detail="Active build targets"
            icon={<FaIcon icon={faBullseye} />}
          />
          <MetricCard
            label="Builds"
            value={summary?.build_count ?? 0}
            detail="Recorded publish jobs"
            icon={<FaIcon icon={faHammer} />}
          />
          <MetricCard
            label="Stored Size"
            value={formatBytes(summary?.stored_bytes ?? 0)}
            detail={`${summary?.published_file_count ?? 0} published files`}
            icon={<FaIcon icon={faHardDrive} />}
          />
        </div>
      )}

      <FilterBar activeCount={activeFilterCount} onClear={clearFilters}>
        <div className="grid items-end gap-4 md:grid-cols-[minmax(0,1fr)_minmax(0,1fr)_auto]">
          <label className="block min-w-0">
            <span className="block font-mono text-xs font-semibold uppercase tracking-[0.22em] text-soft">
              Package
            </span>
            <input
              type="text"
              value={packageInput}
              onChange={(e) => setPackageInput(e.target.value)}
              placeholder="Filter by package name"
              className="mt-2.5 w-full border border-edge bg-black px-3 py-2.5 font-mono text-xs text-white outline-none transition-colors focus:border-accent-lime"
            />
          </label>
          <label className="block min-w-0">
            <span className="block font-mono text-xs font-semibold uppercase tracking-[0.22em] text-soft">
              Target
            </span>
            <input
              type="text"
              value={targetInput}
              onChange={(e) => setTargetInput(e.target.value)}
              placeholder="Filter by target"
              className="mt-2.5 w-full border border-edge bg-black px-3 py-2.5 font-mono text-xs text-white outline-none transition-colors focus:border-accent-lime"
            />
          </label>
          <div className="min-w-0">
            <span className="block font-mono text-xs font-semibold uppercase tracking-[0.22em] text-soft">
              Kind
            </span>
            <div className="mt-2.5">
              <SegmentedControl<KindFilter>
                value={filters.kindFilter}
                onChange={(val) => setFilters({ kindFilter: val, offset: 0 })}
                ariaLabel="Filter by file kind"
                size="md"
                items={[
                  { value: "all", label: "All" },
                  { value: "rpm", label: "RPM" },
                  { value: "srpm", label: "SRPM" },
                  { value: "log", label: "Logs" },
                ]}
              />
            </div>
          </div>
        </div>
      </FilterBar>

      <section className="space-y-3">
        <div className="flex items-center justify-between gap-4">
          <h2 className="font-mono text-[13px] font-bold uppercase tracking-[0.04em] text-white">
            Published files
          </h2>
          {!inventoryLoading && repoFiles.length > 0 ? (
            <span className="font-mono text-xs font-semibold uppercase tracking-[0.14em] text-[#6b6b73]">
              {repoFiles.length} shown
            </span>
          ) : null}
        </div>

        {inventoryLoading ? (
          <LoadingBlock label="Loading published files…" lines={4} />
        ) : repoFiles.length === 0 ? (
          <EmptyState
            title={hasActiveFilters ? "No matching files" : "No published files"}
            description={
              hasActiveFilters
                ? "Try different package, target, or file-kind filters."
                : "RPMs, source RPMs, and logs will appear here after a successful build."
            }
            action={
              hasActiveFilters ? (
                <Button variant="subtle" onClick={clearFilters}>
                  Clear filters
                </Button>
              ) : undefined
            }
          />
        ) : (
          <div className="space-y-3">
            {repoFiles.map((file) => (
              <RepoFileCard
                key={`${file.job_id}:${file.path}`}
                file={file}
                showPackageContext
              />
            ))}
          </div>
        )}

        {!inventoryLoading && inventory && repoFiles.length > 0 && (
          <PaginationControls
            offset={filters.offset}
            pageSize={PAGE_SIZE}
            count={repoFiles.length}
            hasMore={inventory.page.has_more}
            total={inventory.page.total}
            isFetching={inventoryQuery.isFetching}
            onOffsetChange={(o) => setFilters({ offset: o })}
          />
        )}
      </section>
    </div>
  );
}

export default function RepositoryBrowserPage() {
  return <RepositoryBrowser />;
}
