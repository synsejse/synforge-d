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
import type { PublishedRepoFile } from "../../lib/types";
import ErrorMessage from "../../components/common/error-message";
import LoadingBlock from "../../components/ui/loading-block";
import FaIcon from "../../components/ui/fa-icon";
import Button from "../../components/ui/button";
import SegmentedControl from "../../components/ui/segmented-control";
import MetricCard from "../../components/ui/metric-card";
import PageHeader from "../../components/ui/page-header";
import Badge from "../../components/ui/badge";
import PaginationControls from "../../components/common/pagination-controls";
import FilterBar from "../../components/common/filter-bar";
import DataTable, { type DataTableColumn } from "../../components/ui/data-table";

const PAGE_SIZE = 50;

const route = getRouteApi("/_authed/repository/");

type KindFilter = "all" | "rpm" | "srpm" | "log";

function getSigningState(file: PublishedRepoFile) {
  if (file.signing_status === "signed") {
    return { label: "SIGNED", variant: "success" as const, title: undefined };
  }
  if (file.signing_status === "failed") {
    return {
      label: "SIGN FAILED",
      variant: "error" as const,
      title: file.signing_error_message || "Artifact signing failed",
    };
  }
  return { label: "NOT SIGNED", variant: "warning" as const, title: undefined };
}

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

  if (summaryQuery.isPending || inventoryQuery.isPending) {
    return <LoadingBlock label="Loading repository inventory…" lines={4} />;
  }

  const loadError = summaryQuery.error ?? inventoryQuery.error;
  if (loadError || !summaryQuery.data || !inventoryQuery.data) {
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

  return (
    <div className="space-y-8">
      {/* Header */}
      <PageHeader
        title="Repository Control"
        description="Published packages, builds, and files."
        color="green"
        actions={[
          { to: "/packages", label: "Packages", icon: faBoxesStacked },
          { to: "/repository/use", label: "Add Repo", icon: faPlus, variant: "primary" },
        ]}
      />

      {/* Metrics */}
      <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
        <MetricCard
          label="Packages"
          value={summaryQuery.data.package_count}
          detail="Published package names"
          icon={<FaIcon icon={faBoxesStacked} />}
        />
        <MetricCard
          label="Targets"
          value={summaryQuery.data.target_count}
          detail="Active build targets"
          variant="accent"
          icon={<FaIcon icon={faBullseye} />}
        />
        <MetricCard
          label="Builds"
          value={summaryQuery.data.build_count}
          detail="Recorded publish jobs"
          icon={<FaIcon icon={faHammer} />}
        />
        <MetricCard
          label="Stored Size"
          value={formatBytes(summaryQuery.data.stored_bytes)}
          detail={`${summaryQuery.data.published_file_count} published files`}
          icon={<FaIcon icon={faHardDrive} />}
        />
      </div>

      {/* Filters */}
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

      {/* Files Table */}
      <div className="border-2 border-white bg-black">
        <div className="border-b-2 border-edge bg-black px-6 py-4">
          <h2 className="font-mono text-sm font-bold uppercase tracking-[0.2em] text-white">
            Published Files
          </h2>
        </div>

        <div className="p-4">
          <DataTable
            columns={publishedFileColumns}
            rows={inventoryQuery.data.repo_files}
            rowKey={(file) => `${file.job_id}:${file.path}`}
            empty={{
              title: "No files match",
              description: "No files match the current filters.",
            }}
          />
        </div>

        {/* Pagination */}
        {inventoryQuery.data.repo_files.length > 0 && (
          <div className="border-t-2 border-edge bg-black px-6 py-4">
            <PaginationControls
              onPrevious={() =>
                setFilters({ offset: Math.max(0, filters.offset - PAGE_SIZE) })
              }
              onNext={() =>
                setFilters({ offset: filters.offset + PAGE_SIZE })
              }
              previousDisabled={inventoryQuery.isFetching || filters.offset === 0}
              nextDisabled={
                inventoryQuery.isFetching || !inventoryQuery.data.page.has_more
              }
              summary={`Offset: ${filters.offset}`}
            />
          </div>
        )}
      </div>
    </div>
  );
}

const publishedFileColumns: DataTableColumn<PublishedRepoFile>[] = [
  {
    key: "package",
    header: "Package",
    mobile: "title",
    cell: (file) => (
      <div>
        <div className="font-mono text-sm text-white">{file.package_name}</div>
        <div className="mt-1 font-mono text-xs text-soft md:hidden">
          {file.mock_chroot || "unknown"}
        </div>
      </div>
    ),
  },
  {
    key: "target",
    header: "Target",
    mobile: "hidden",
    className: "font-mono text-sm text-muted",
    cell: (file) => file.mock_chroot || "unknown",
  },
  {
    key: "file",
    header: "File",
    mobile: "field",
    cell: (file) => {
      const fileName = file.path.split("/").pop() || file.path;
      return (
        <a
          href={`/repo/${file.path}`}
          className="break-all font-mono text-sm text-accent-lime transition duration-100 ease-linear hover:text-white"
        >
          {fileName}
        </a>
      );
    },
  },
  {
    key: "path",
    header: "Repo Path",
    mobile: "field",
    className: "font-mono text-sm text-soft break-all",
    cell: (file) => file.path,
  },
  {
    key: "size",
    header: "Size",
    mobile: "field",
    className: "text-right font-mono text-sm text-muted",
    headerClassName: "text-right",
    cell: (file) => formatBytes(file.size_bytes),
  },
  {
    key: "signing",
    header: "Signing",
    mobile: "badge",
    cell: (file) => {
      const signingState = getSigningState(file);
      return (
        <Badge variant={signingState.variant} title={signingState.title}>
          {signingState.label}
        </Badge>
      );
    },
  },
];

export default function RepositoryBrowserPage() {
  return <RepositoryBrowser />;
}
