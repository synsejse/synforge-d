import { useEffect, useState, type SyntheticEvent } from "react";
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
import EmptyState from "../../components/ui/empty-state";
import LoadingBlock from "../../components/ui/loading-block";
import FaIcon from "../../components/ui/fa-icon";
import Button from "../../components/ui/button";
import SegmentedControl from "../../components/ui/segmented-control";
import MetricCard from "../../components/ui/metric-card";
import PageHeader from "../../components/ui/page-header";
import PaginationControls from "../../components/common/pagination-controls";
import FilterBar from "../../components/common/filter-bar";
import RepoFileCard from "./components/repo-file-card";

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

  function handleApply(event: SyntheticEvent) {
    event.preventDefault();
    setFilters({
      offset: 0,
      packageFilter: packageInput,
      targetFilter: targetInput,
    });
  }

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

  return (
    <div className="space-y-6">
      <PageHeader
        title="Repository Control"
        description="Published packages, builds, and files."
        color="green"
        actions={[
          { to: "/packages", label: "Packages", icon: faBoxesStacked },
          { to: "/repository/use", label: "Add Repo", icon: faPlus, variant: "primary" },
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
            variant="accent"
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

      <form onSubmit={handleApply}>
        <FilterBar
          activeCount={
            (filters.packageFilter ? 1 : 0) +
            (filters.targetFilter ? 1 : 0) +
            (filters.kindFilter !== "all" ? 1 : 0)
          }
          onClear={() => {
            setPackageInput("");
            setTargetInput("");
            setFilters({
              offset: 0,
              packageFilter: "",
              targetFilter: "",
              kindFilter: "all",
            });
          }}
          trailing={
            <Button type="submit" variant="secondary" size="md">
              Apply Filters
            </Button>
          }
        >
          <div className="grid gap-4 md:grid-cols-3">
            <label className="block">
              <span className="mb-2 block font-mono text-xs font-bold uppercase tracking-[0.2em] text-muted">
                Package
              </span>
              <input
                type="text"
                value={packageInput}
                onChange={(e) => setPackageInput(e.target.value)}
                placeholder="Filter by package name"
                className="w-full border-2 border-edge-strong bg-black px-4 py-2.5 font-mono text-sm text-white outline-none transition duration-100 ease-linear focus:border-accent-lime focus:ring-2 focus:ring-accent-lime"
              />
            </label>
            <label className="block">
              <span className="mb-2 block font-mono text-xs font-bold uppercase tracking-[0.2em] text-muted">
                Target
              </span>
              <input
                type="text"
                value={targetInput}
                onChange={(e) => setTargetInput(e.target.value)}
                placeholder="Filter by target"
                className="w-full border-2 border-edge-strong bg-black px-4 py-2.5 font-mono text-sm text-white outline-none transition duration-100 ease-linear focus:border-accent-lime focus:ring-2 focus:ring-accent-lime"
              />
            </label>
            <label className="block">
              <span className="mb-2 block font-mono text-xs font-bold uppercase tracking-[0.2em] text-muted">
                Kind
              </span>
              <SegmentedControl<KindFilter>
                value={filters.kindFilter}
                onChange={(val) => setFilters({ kindFilter: val, offset: 0 })}
                ariaLabel="Filter by file kind"
                size="md"
                fullWidth
                items={[
                  { value: "all", label: "All" },
                  { value: "rpm", label: "RPM" },
                  { value: "srpm", label: "SRPM" },
                  { value: "log", label: "Logs" },
                ]}
              />
            </label>
          </div>
        </FilterBar>
      </form>

      <section className="space-y-3">
        <div className="flex items-baseline justify-between gap-4 border-b-2 border-edge-strong pb-2">
          <h2 className="text-base font-semibold text-white">Published files</h2>
          {!inventoryLoading && repoFiles.length > 0 ? (
            <span className="font-mono text-xs uppercase tracking-[0.18em] text-soft">
              {repoFiles.length} shown
            </span>
          ) : null}
        </div>

        {inventoryLoading ? (
          <LoadingBlock label="Loading published files…" lines={4} />
        ) : repoFiles.length === 0 ? (
          <EmptyState
            title="No files match"
            description="No files match the current filters."
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
