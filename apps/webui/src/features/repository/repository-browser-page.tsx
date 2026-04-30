import { useState, type SyntheticEvent } from "react";
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
import Select from "../../components/ui/select";
import MetricCard from "../../components/ui/metric-card";
import PageHeader from "../../components/ui/page-header";
import Badge from "../../components/ui/badge";

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

  const setFilters = (update: Partial<typeof search>) =>
    navigate({ search: (prev) => ({ ...prev, ...update }) });

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
        eyebrow="MANAGED_REPOSITORY"
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
      <form onSubmit={handleApply} className="border-2 border-white bg-black p-5">
        <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-[1fr_1fr_auto_auto]">
          <div>
            <label className="block">
              <span className="mb-2 block font-mono text-xs font-bold uppercase tracking-[0.2em] text-zinc-400">
                Package
              </span>
              <input
                type="text"
                value={packageInput}
                onChange={(e) => setPackageInput(e.target.value)}
                placeholder="Filter by package name"
                className="w-full border-2 border-zinc-700 bg-black px-4 py-2.5 font-mono text-sm text-white outline-none transition duration-100 ease-linear focus:border-[var(--theme-accent-lime)] focus:ring-2 focus:ring-[var(--theme-accent-lime)]"
              />
            </label>
          </div>
          <div>
            <label className="block">
              <span className="mb-2 block font-mono text-xs font-bold uppercase tracking-[0.2em] text-zinc-400">
                Target
              </span>
              <input
                type="text"
                value={targetInput}
                onChange={(e) => setTargetInput(e.target.value)}
                placeholder="Filter by target"
                className="w-full border-2 border-zinc-700 bg-black px-4 py-2.5 font-mono text-sm text-white outline-none transition duration-100 ease-linear focus:border-[var(--theme-accent-lime)] focus:ring-2 focus:ring-[var(--theme-accent-lime)]"
              />
            </label>
          </div>
          <div>
            <label className="block">
              <span className="mb-2 block font-mono text-xs font-bold uppercase tracking-[0.2em] text-zinc-400">
                Kind
              </span>
              <Select
                value={filters.kindFilter}
                onValueChange={(val) =>
                  setFilters({ kindFilter: val as KindFilter, offset: 0 })
                }
                options={[
                  { value: "all", label: "All" },
                  { value: "rpm", label: "RPM" },
                  { value: "srpm", label: "SRPM" },
                  { value: "log", label: "Logs" },
                ]}
              />
            </label>
          </div>
          <div className="flex items-end md:col-span-2 xl:col-span-1">
            <Button type="submit" variant="secondary" size="md">
              Apply Filters
            </Button>
          </div>
        </div>
      </form>

      {/* Files Table */}
      <div className="border-2 border-white bg-black">
        <div className="border-b-2 border-zinc-800 bg-black px-6 py-4">
          <h2 className="font-mono text-sm font-bold uppercase tracking-[0.2em] text-white">
            Published Files
          </h2>
        </div>
        
        {inventoryQuery.data.repo_files.length === 0 ? (
          <div className="px-6 py-12 text-center">
            <p className="font-mono text-sm text-zinc-500">
              No files match the current filters.
            </p>
          </div>
        ) : (
          <>
            <div className="space-y-3 p-4 md:hidden">
              {inventoryQuery.data.repo_files.map((file) => {
                const fileName = file.path.split("/").pop() || file.path;
                const signingState = getSigningState(file);
                return (
                  <article
                    key={`mobile:${file.job_id}:${file.path}`}
                    className="border-2 border-zinc-700 bg-black p-4"
                  >
                    <div className="flex items-start justify-between gap-3">
                      <div className="min-w-0">
                        <div className="font-mono text-sm text-white">{file.package_name}</div>
                        <div className="mt-1 font-mono text-xs text-zinc-500">
                          {file.mock_chroot || "unknown"}
                        </div>
                      </div>
                      <Badge variant={signingState.variant} title={signingState.title}>
                        {signingState.label}
                      </Badge>
                    </div>
                    <div className="mt-3">
                      <a
                        href={`/repo/${file.path}`}
                        className="break-all font-mono text-sm text-[var(--theme-accent-lime)] transition duration-100 ease-linear hover:text-white"
                      >
                        {fileName}
                      </a>
                      <div className="mt-1 break-all font-mono text-xs text-zinc-500">
                        {file.path}
                      </div>
                    </div>
                    <div className="mt-3 font-mono text-xs text-zinc-400">
                      <span className="text-zinc-500">Size:</span> {formatBytes(file.size_bytes)}
                    </div>
                  </article>
                );
              })}
            </div>
            <div className="hidden overflow-x-auto md:block">
              <table className="w-full min-w-[640px] lg:min-w-[900px]">
                <thead>
                  <tr className="border-b-2 border-zinc-800">
                    <th className="px-6 py-4 text-left font-mono text-xs font-bold uppercase tracking-[0.2em] text-zinc-500">
                      Package
                    </th>
                    <th className="px-6 py-4 text-left font-mono text-xs font-bold uppercase tracking-[0.2em] text-zinc-500">
                      Target
                    </th>
                    <th className="px-6 py-4 text-left font-mono text-xs font-bold uppercase tracking-[0.2em] text-zinc-500">
                      File
                    </th>
                    <th className="px-6 py-4 text-left font-mono text-xs font-bold uppercase tracking-[0.2em] text-zinc-500">
                      Repo Path
                    </th>
                    <th className="px-6 py-4 text-right font-mono text-xs font-bold uppercase tracking-[0.2em] text-zinc-500">
                      Size
                    </th>
                    <th className="px-6 py-4 text-left font-mono text-xs font-bold uppercase tracking-[0.2em] text-zinc-500">
                      Signing
                    </th>
                  </tr>
                </thead>
                <tbody>
                  {inventoryQuery.data.repo_files.map((file) => {
                    const fileName = file.path.split("/").pop() || file.path;
                    const signingState = getSigningState(file);
                    return (
                      <tr
                        key={`${file.job_id}:${file.path}`}
                        className="border-b border-zinc-900 transition duration-100 ease-linear hover:bg-zinc-950"
                      >
                        <td className="px-6 py-4 font-mono text-sm text-white">
                          {file.package_name}
                        </td>
                        <td className="px-6 py-4 font-mono text-sm text-zinc-400">
                          {file.mock_chroot || "unknown"}
                        </td>
                        <td className="px-6 py-4">
                          <a
                            href={`/repo/${file.path}`}
                            className="break-all font-mono text-sm text-[var(--theme-accent-lime)] transition duration-100 ease-linear hover:text-white"
                          >
                            {fileName}
                          </a>
                        </td>
                        <td className="px-6 py-4 font-mono text-sm text-zinc-500">
                          {file.path}
                        </td>
                        <td className="px-6 py-4 text-right font-mono text-sm text-zinc-400">
                          {formatBytes(file.size_bytes)}
                        </td>
                        <td className="px-6 py-4">
                          <Badge variant={signingState.variant} title={signingState.title}>
                            {signingState.label}
                          </Badge>
                        </td>
                      </tr>
                    );
                  })}
                </tbody>
              </table>
            </div>
          </>
        )}

        {/* Pagination */}
        {inventoryQuery.data.repo_files.length > 0 && (
          <div className="flex flex-col gap-3 border-t-2 border-zinc-800 bg-black px-6 py-4 sm:flex-row sm:items-center sm:justify-between">
            <Button
              variant="secondary"
              size="md"
              onClick={() =>
                setFilters({ offset: Math.max(0, filters.offset - PAGE_SIZE) })
              }
              disabled={inventoryQuery.isFetching || filters.offset === 0}
            >
              Previous
            </Button>
            <span className="font-mono text-sm text-zinc-400">
              Offset: {filters.offset}
            </span>
            <Button
              variant="secondary"
              size="md"
              onClick={() => setFilters({ offset: filters.offset + PAGE_SIZE })}
              disabled={
                inventoryQuery.isFetching || !inventoryQuery.data.page.has_more
              }
            >
              Next
            </Button>
          </div>
        )}
      </div>
    </div>
  );
}

export default function RepositoryBrowserPage() {
  return <RepositoryBrowser />;
}
