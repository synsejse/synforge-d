import { useEffect, useState, type FormEvent } from "react";
import {
  faBoxesStacked,
  faMagnifyingGlass,
  faDownload,
  faFolderTree,
} from "@fortawesome/free-solid-svg-icons";
import api from "../lib/api";
import { formatDateTime } from "../lib/datetime";
import type { PublishedRepoFile, RepoSummaryResponse } from "../lib/types";
import EmptyState from "./EmptyState";
import FaIcon from "./FaIcon";
import LoadingBlock from "./LoadingBlock";
import MetricCard from "./MetricCard";
import PageHeader from "./PageHeader";

const PAGE_SIZE = 50;

export default function RepositoryBrowser() {
  const [summary, setSummary] = useState<RepoSummaryResponse | null>(null);
  const [files, setFiles] = useState<PublishedRepoFile[]>([]);
  const [hasMore, setHasMore] = useState(false);
  const [offset, setOffset] = useState(() => {
    if (typeof window === "undefined") {
      return 0;
    }
    return Number(
      new URLSearchParams(window.location.search).get("offset") || "0",
    );
  });
  const [packageFilter, setPackageFilter] = useState(() => {
    if (typeof window === "undefined") {
      return "";
    }
    return new URLSearchParams(window.location.search).get("package") || "";
  });
  const [targetFilter, setTargetFilter] = useState(() => {
    if (typeof window === "undefined") {
      return "";
    }
    return new URLSearchParams(window.location.search).get("target") || "";
  });
  const [kindFilter, setKindFilter] = useState<"all" | "rpm" | "srpm" | "log">(
    () => {
      if (typeof window === "undefined") {
        return "all";
      }
      const value = new URLSearchParams(window.location.search).get("kind");
      return value === "rpm" || value === "srpm" || value === "log"
        ? value
        : "all";
    },
  );
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
        if (nextOffset > 0) {
          params.set("offset", String(nextOffset));
        }
        if (nextPackageFilter.trim()) {
          params.set("package", nextPackageFilter.trim());
        }
        if (nextTargetFilter.trim()) {
          params.set("target", nextTargetFilter.trim());
        }
        if (nextKindFilter !== "all") {
          params.set("kind", nextKindFilter);
        }
        const query = params.toString();
        window.history.replaceState(
          {},
          "",
          `/repository/${query ? `?${query}` : ""}`,
        );
      }
    } catch (e) {
      setError(
        e instanceof Error ? e.message : "Failed to load repository inventory",
      );
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
    return (
      <div className="border border-zinc-800 bg-black p-4 text-zinc-200">
        Error: {error}
      </div>
    );
  }

  const targets = summary?.targets ?? [];
  const recentFiles = summary?.recent_files ?? [];

  return (
    <div className="space-y-8">
      <PageHeader
        eyebrow="Managed Repository"
        title="Repository Control"
        description="Published packages, builds, and files."
        actions={[
          { href: "/packages/", label: "Packages", icon: faBoxesStacked },
        ]}
      />

      <section className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
        <MetricCard
          label="Packages"
          value={summary?.package_count ?? 0}
          detail="Published package names"
        />
        <MetricCard
          label="Targets"
          value={summary?.target_count ?? 0}
          detail="Active build targets"
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
        />
      </section>

      <section className="border border-zinc-800 bg-black p-6">
        <div className="flex items-end justify-between gap-4">
          <div>
            <div className="text-xs uppercase tracking-[0.22em] text-zinc-500">
              Build Targets
            </div>
            <h2 className="mt-2 text-2xl font-semibold text-white">
              Published target coverage
            </h2>
          </div>
          <div className="text-sm text-zinc-500">
            Per-target package, build, and size totals
          </div>
        </div>

        {targets.length === 0 ? (
          <div className="mt-5">
            <EmptyState>No published repository targets yet.</EmptyState>
          </div>
        ) : (
          <div className="mt-5 grid gap-4 md:grid-cols-2 xl:grid-cols-3">
            {targets.map((target) => (
              <article
                key={target.mock_chroot}
                className="border border-zinc-800 bg-zinc-950/40 p-5"
              >
                <div className="font-mono text-lg font-semibold text-white">
                  {target.mock_chroot}
                </div>
                <div className="mt-4 grid gap-3 sm:grid-cols-3">
                  <TargetStat label="Packages" value={target.package_count} />
                  <TargetStat label="Builds" value={target.build_count} />
                  <TargetStat
                    label="Size"
                    value={formatBytes(target.size_bytes)}
                  />
                </div>
              </article>
            ))}
          </div>
        )}
      </section>

      <section className="border border-zinc-800 bg-black p-6">
        <div className="flex items-end justify-between gap-4">
          <div>
            <div className="text-xs uppercase tracking-[0.22em] text-zinc-500">
              Recent Output
            </div>
            <h2 className="mt-2 text-2xl font-semibold text-white">
              Latest published files
            </h2>
          </div>
          <FaIcon icon={faFolderTree} className="text-zinc-500" />
        </div>

        {recentFiles.length === 0 ? (
          <div className="mt-5">
            <EmptyState>No published files have been recorded yet.</EmptyState>
          </div>
        ) : (
          <div className="mt-5 grid gap-3">
            {recentFiles.slice(0, 6).map((file) => (
              <div
                key={`${file.job_id}:${file.path}`}
                className="grid gap-3 border border-zinc-800 bg-zinc-950/40 p-4 md:grid-cols-[minmax(0,1fr)_auto]"
              >
                <div className="min-w-0">
                  <div className="truncate font-mono text-sm text-white">
                    {file.path}
                  </div>
                  <div className="mt-2 flex flex-wrap gap-2 text-xs uppercase tracking-[0.18em] text-zinc-500">
                    <span>{file.package_name}</span>
                    <span>{file.kind}</span>
                    <span>{formatBytes(file.size_bytes)}</span>
                  </div>
                  <div className="mt-2 text-xs text-zinc-500">
                    {formatDateTime(file.published_at)}
                  </div>
                </div>
                <div className="flex items-start md:justify-end">
                  <a
                    href={`/repo/${file.path}`}
                    className="inline-flex items-center border border-zinc-800 px-3 py-2 text-sm text-zinc-200 transition hover:border-zinc-600 hover:bg-zinc-950"
                  >
                    <FaIcon icon={faDownload} className="mr-2" />
                    Download
                  </a>
                </div>
              </div>
            ))}
          </div>
        )}
      </section>

      <section className="space-y-4">
        <form
          onSubmit={handleApply}
          className="grid gap-3 border border-zinc-800 bg-black p-4 md:grid-cols-[minmax(0,1fr)_240px_180px_auto]"
        >
          <label className="block">
            <span className="sr-only">Filter by package name</span>
            <input
              type="search"
              value={packageFilter}
              onChange={(event) => setPackageFilter(event.target.value)}
              placeholder="Filter by package"
              className="w-full border border-zinc-800 bg-black px-4 py-2.5 text-sm text-white outline-none transition focus:border-zinc-600"
            />
          </label>
          <label className="block">
            <span className="sr-only">Filter by target</span>
            <input
              type="search"
              value={targetFilter}
              onChange={(event) => setTargetFilter(event.target.value)}
              placeholder="Filter by target"
              className="w-full border border-zinc-800 bg-black px-4 py-2.5 text-sm text-white outline-none transition focus:border-zinc-600"
            />
          </label>
          <label className="block">
            <span className="sr-only">Filter by artifact kind</span>
            <select
              value={kindFilter}
              onChange={(event) =>
                setKindFilter(
                  event.target.value as "all" | "rpm" | "srpm" | "log",
                )
              }
              className="w-full border border-zinc-800 bg-black px-4 py-2.5 text-sm text-white outline-none transition focus:border-zinc-600"
            >
              <option value="all">All kinds</option>
              <option value="rpm">RPM</option>
              <option value="srpm">SRPM</option>
              <option value="log">Log</option>
            </select>
          </label>
          <button
            type="submit"
            className="border border-zinc-800 bg-black px-4 py-2.5 text-sm text-zinc-200 transition hover:border-zinc-600 hover:bg-zinc-950"
          >
            <FaIcon icon={faMagnifyingGlass} className="mr-2" />
            Apply
          </button>
        </form>

        <div className="flex items-end justify-between gap-4">
          <div>
            <div className="text-xs uppercase tracking-[0.24em] text-zinc-500">
              Inventory
            </div>
            <h2 className="mt-2 text-2xl font-semibold text-white">
              Published files
            </h2>
          </div>
          <div className="text-sm text-zinc-500">
            Server-paginated inventory view
          </div>
        </div>

        {files.length === 0 ? (
          <EmptyState>
            No managed repository files are published yet.
          </EmptyState>
        ) : (
          <div className="overflow-x-auto border border-zinc-800 bg-black">
            <table className="min-w-[980px] w-full">
              <caption className="sr-only">
                Published repository files with package, target, type, size,
                publication date, and actions.
              </caption>
              <thead className="bg-zinc-950 text-left text-xs uppercase tracking-[0.2em] text-zinc-500">
                <tr>
                  <th scope="col" className="px-4 py-3">Package</th>
                  <th scope="col" className="px-4 py-3">Repo Path</th>
                  <th scope="col" className="px-4 py-3">Target</th>
                  <th scope="col" className="px-4 py-3">Kind</th>
                  <th scope="col" className="px-4 py-3">Size</th>
                  <th scope="col" className="px-4 py-3">Published</th>
                  <th scope="col" className="px-4 py-3">Actions</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-white/8">
                {files.map((file) => (
                  <tr
                    key={`${file.job_id}:${file.path}`}
                    className="hover:bg-zinc-950"
                  >
                    <td className="px-4 py-3">
                      <a
                        href={`/packages/view/?name=${encodeURIComponent(file.package_name)}`}
                        className="text-white transition hover:text-zinc-300"
                      >
                        {file.package_name}
                      </a>
                    </td>
                    <td className="px-4 py-3 font-mono text-sm text-zinc-200">
                      {file.path}
                    </td>
                    <td className="px-4 py-3 font-mono text-sm text-zinc-400">
                      {file.mock_chroot || "unknown"}
                    </td>
                    <td className="px-4 py-3 text-sm uppercase tracking-[0.18em] text-zinc-500">
                      {file.kind}
                    </td>
                    <td className="px-4 py-3 text-sm text-zinc-400">
                      {formatBytes(file.size_bytes)}
                    </td>
                    <td className="px-4 py-3 text-sm text-zinc-400">
                      {formatDateTime(file.published_at)}
                    </td>
                    <td className="px-4 py-3">
                      <div className="flex flex-wrap gap-2">
                        <a
                          href={`/repo/${file.path}`}
                          className="inline-flex items-center border border-zinc-800 px-3 py-2 text-sm text-zinc-200 transition hover:border-zinc-600 hover:bg-zinc-950"
                        >
                          <FaIcon icon={faDownload} className="mr-2" />
                          Download
                        </a>
                        <a
                          href={`/jobs/view/?id=${encodeURIComponent(file.job_id)}`}
                          className="inline-flex items-center border border-zinc-800 px-3 py-2 text-sm text-zinc-400 transition hover:border-zinc-600 hover:bg-zinc-950 hover:text-zinc-200"
                        >
                          Build
                        </a>
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}

        {files.length > 0 ? (
          <div className="flex items-center justify-between gap-3">
            <button
              onClick={() => load(Math.max(0, offset - PAGE_SIZE))}
              disabled={loading || offset === 0}
              className="border border-zinc-800 px-4 py-2 text-sm text-zinc-300 disabled:cursor-not-allowed disabled:opacity-50"
            >
              Previous
            </button>
            <button
              onClick={() => load(offset + PAGE_SIZE)}
              disabled={loading || !hasMore}
              className="border border-zinc-800 px-4 py-2 text-sm text-zinc-300 disabled:cursor-not-allowed disabled:opacity-50"
            >
              Next
            </button>
          </div>
        ) : null}
      </section>
    </div>
  );
}

function TargetStat({
  label,
  value,
}: {
  label: string;
  value: string | number;
}) {
  return (
    <div className="border border-zinc-800 bg-black px-3 py-3">
      <div className="text-[10px] uppercase tracking-[0.18em] text-zinc-500">
        {label}
      </div>
      <div className="mt-2 text-sm font-medium text-zinc-200">{value}</div>
    </div>
  );
}

function formatBytes(bytes: number) {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KiB`;
  if (bytes < 1024 * 1024 * 1024)
    return `${(bytes / (1024 * 1024)).toFixed(1)} MiB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GiB`;
}
