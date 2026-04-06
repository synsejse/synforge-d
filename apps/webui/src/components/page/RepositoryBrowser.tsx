import { useEffect, useState, type FormEvent } from "react";
import { faBoxesStacked } from "@fortawesome/free-solid-svg-icons";
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

const PAGE_SIZE = 50;

export default function RepositoryBrowser() {
  const [summary, setSummary] = useState<RepoSummaryResponse | null>(null);
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

  function handleApply(event: FormEvent) {
    event.preventDefault();
    load(0, packageFilter, targetFilter, kindFilter);
  }

  if (loading && !summary) {
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
          value={summary?.package_count ?? 0}
          detail="Published package names"
        />
        <MetricCard
          label="Targets"
          value={summary?.target_count ?? 0}
          detail="Active build targets"
          accent="lime"
        />
        <MetricCard
          label="Builds"
          value={summary?.build_count ?? 0}
          detail="Recorded publish jobs"
        />
        <MetricCard
          label="Stored Size"
          value={formatBytes(summary?.stored_bytes ?? 0)}
          detail={`${summary?.published_file_count ?? 0} published files`}
          icon
        />
      </div>

      {/* Filters */}
      <form onSubmit={handleApply} className="border-2 border-white bg-black p-5">
        <div className="grid gap-4 md:grid-cols-[1fr_1fr_auto_auto]">
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
          <div className="flex items-end">
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
        
        <div className="overflow-x-auto">
          <table className="w-full min-w-[900px]">
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
              </tr>
            </thead>
            <tbody>
              {files.length === 0 ? (
                <tr>
                  <td colSpan={5} className="px-6 py-12 text-center">
                    <p className="font-mono text-sm text-zinc-500">
                      No files match the current filters.
                    </p>
                  </td>
                </tr>
              ) : (
                files.map((file) => {
                  const fileName = file.path.split("/").pop() || file.path;
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
                    </tr>
                  );
                })
              )}
            </tbody>
          </table>
        </div>

        {/* Pagination */}
        {files.length > 0 && (
          <div className="flex items-center justify-between border-t-2 border-zinc-800 bg-black px-6 py-4">
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
