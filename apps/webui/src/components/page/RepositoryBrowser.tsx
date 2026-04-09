import { useEffect, useState, type SyntheticEvent } from "react";
import {
  faBoxesStacked,
  faBullseye,
  faHardDrive,
  faHammer,
} from "@fortawesome/free-solid-svg-icons";
import api from "../../lib/api";
import { formatBytes } from "../../lib/bytes";
import type { PublishedRepoFile, RepoSummaryResponse } from "../../lib/types";
import ErrorMessage from "../common/ErrorMessage";
import LoadingBlock from "../ui/LoadingBlock";
import FaIcon from "../ui/FaIcon";
import Button from "../ui/Button";
import Select from "../ui/Select";
import MetricCard from "../ui/MetricCard";
import PageHeader from "../ui/PageHeader";
import Badge from "../ui/Badge";

const PAGE_SIZE = 50;
const EMPTY_REPO_SUMMARY: RepoSummaryResponse = {
  package_count: 0,
  target_count: 0,
  build_count: 0,
  published_file_count: 0,
  stored_bytes: 0,
  recent_files: [],
  targets: [],
};

export default function RepositoryBrowser() {
  const [summary, setSummary] = useState<RepoSummaryResponse>(EMPTY_REPO_SUMMARY);
  const [files, setFiles] = useState<PublishedRepoFile[]>([]);
  const [hasMore, setHasMore] = useState(false);
  const [offset, setOffset] = useState(() => {
    if (typeof window === "undefined") return 0;
    return Number(new URLSearchParams(window.location.search).get("offset") || "0");
  });
  const [packageFilter, setPackageFilter] = useState(() => {
    if (typeof window === "undefined") return "";
    return new URLSearchParams(window.location.search).get("package") || "";
  });
  const [targetFilter, setTargetFilter] = useState(() => {
    if (typeof window === "undefined") return "";
    return new URLSearchParams(window.location.search).get("target") || "";
  });
  const [kindFilter, setKindFilter] = useState<"all" | "rpm" | "srpm" | "log">(() => {
    if (typeof window === "undefined") return "all";
    const value = new URLSearchParams(window.location.search).get("kind");
    return value === "rpm" || value === "srpm" || value === "log" ? value : "all";
  });
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const getSigningState = (file: PublishedRepoFile) => {
    if (file.signing_status === "signed") {
      return { label: "SIGNED", variant: "success" as const };
    }
    if (file.signing_status === "failed") {
      return {
        label: "SIGN FAILED",
        variant: "error" as const,
        title: file.signing_error_message || "Artifact signing failed",
      };
    }
    return { label: "NOT SIGNED", variant: "warning" as const };
  };

  async function load(
    nextOffset = offset,
    nextPackageFilter = packageFilter,
    nextTargetFilter = targetFilter,
    nextKindFilter = kindFilter,
  ) {
    try {
      setLoading(true);
      const [summaryRes, inventoryRes] = await Promise.all([
        api.getRepoSummary(),
        api.getRepoInventory(PAGE_SIZE, nextOffset, {
          packageName: nextPackageFilter,
          mockChroot: nextTargetFilter,
          kind: nextKindFilter,
        }),
      ]);
      setSummary(summaryRes);
      setFiles(inventoryRes.repo_files);
      setHasMore(inventoryRes.page.has_more);
      setOffset(nextOffset);
      setPackageFilter(nextPackageFilter);
      setTargetFilter(nextTargetFilter);
      setKindFilter(nextKindFilter);
      setError(null);
      if (typeof window !== "undefined") {
        const params = new URLSearchParams();
        if (nextOffset > 0) params.set("offset", String(nextOffset));
        if (nextPackageFilter.trim()) params.set("package", nextPackageFilter.trim());
        if (nextTargetFilter.trim()) params.set("target", nextTargetFilter.trim());
        if (nextKindFilter !== "all") params.set("kind", nextKindFilter);
        const query = params.toString();
        window.history.replaceState({}, "", `/repository/${query ? `?${query}` : ""}`);
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to load repository inventory");
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    load();
  }, []);

  function handleApply(event: SyntheticEvent) {
    event.preventDefault();
    load(0, packageFilter, targetFilter, kindFilter);
  }

  if (loading) {
    return <LoadingBlock label="Loading repository inventory…" lines={4} />;
  }

  if (error) {
    return <ErrorMessage message={error} />;
  }

  return (
    <div className="space-y-8">
      {/* Header */}
      <PageHeader
        eyebrow="MANAGED_REPOSITORY"
        title="Repository Control"
        description="Published packages, builds, and files."
        color="green"
        actions={[{ href: "/packages/", label: "Packages", icon: faBoxesStacked }]}
      />

      {/* Metrics */}
      <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
        <MetricCard
          label="Packages"
          value={summary.package_count}
          detail="Published package names"
          icon={<FaIcon icon={faBoxesStacked} />}
        />
        <MetricCard
          label="Targets"
          value={summary.target_count}
          detail="Active build targets"
          variant="accent"
          icon={<FaIcon icon={faBullseye} />}
        />
        <MetricCard
          label="Builds"
          value={summary.build_count}
          detail="Recorded publish jobs"
          icon={<FaIcon icon={faHammer} />}
        />
        <MetricCard
          label="Stored Size"
          value={formatBytes(summary.stored_bytes)}
          detail={`${summary.published_file_count} published files`}
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
                value={packageFilter}
                onChange={(e) => setPackageFilter(e.target.value)}
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
                value={targetFilter}
                onChange={(e) => setTargetFilter(e.target.value)}
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
                value={kindFilter}
                onValueChange={(val) => setKindFilter(val as "all" | "rpm" | "srpm" | "log")}
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
        
        {files.length === 0 ? (
          <div className="px-6 py-12 text-center">
            <p className="font-mono text-sm text-zinc-500">
              No files match the current filters.
            </p>
          </div>
        ) : (
          <>
            <div className="space-y-3 p-4 md:hidden">
              {files.map((file) => {
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
                  {files.map((file) => {
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
        {files.length > 0 && (
          <div className="flex flex-col gap-3 border-t-2 border-zinc-800 bg-black px-6 py-4 sm:flex-row sm:items-center sm:justify-between">
            <Button
              variant="secondary"
              size="md"
              onClick={() => load(Math.max(0, offset - PAGE_SIZE))}
              disabled={loading || offset === 0}
            >
              Previous
            </Button>
            <span className="font-mono text-sm text-zinc-400">
              Offset: {offset}
            </span>
            <Button
              variant="secondary"
              size="md"
              onClick={() => load(offset + PAGE_SIZE)}
              disabled={loading || !hasMore}
            >
              Next
            </Button>
          </div>
        )}
      </div>
    </div>
  );
}
