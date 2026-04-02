import { useEffect, useMemo, useState } from "react";
import {
  faBoxesStacked,
  faCube,
  faDownload,
  faFolderTree,
  faHardDrive,
  faLayerGroup,
  faUpRightFromSquare,
} from "@fortawesome/free-solid-svg-icons";
import api from "../lib/api";
import { compareTimestampsDesc, formatDateTime } from "../lib/datetime";
import type { PublishedRepoFile } from "../lib/types";
import EmptyState from "./EmptyState";
import FaIcon from "./FaIcon";
import PageHeader from "./PageHeader";

type RepoBuildGroup = {
  jobId: string;
  files: PublishedRepoFile[];
};

type RepoTargetGroup = {
  mockChroot: string;
  builds: RepoBuildGroup[];
};

type RepoPackageGroup = {
  packageName: string;
  targets: RepoTargetGroup[];
};

export default function RepositoryBrowser() {
  const [files, setFiles] = useState<PublishedRepoFile[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    async function load() {
      try {
        setLoading(true);
        const res = await api.getRepoInventory();
        setFiles(res.repo_files);
        setError(null);
      } catch (e) {
        setError(e instanceof Error ? e.message : "Failed to load repository inventory");
      } finally {
        setLoading(false);
      }
    }

    load();
  }, []);

  const grouped = useMemo(() => groupRepoFiles(files), [files]);
  const summary = useMemo(() => summarizeInventory(grouped), [grouped]);
  const totalBytes = files.reduce((sum, file) => sum + file.size_bytes, 0);
  const recentFiles = useMemo(
    () =>
      [...files]
        .sort((left, right) => compareTimestampsDesc(left.published_at, right.published_at))
        .slice(0, 10),
    [files]
  );

  if (loading) {
    return <div className="text-zinc-400">Loading repository inventory…</div>;
  }

  if (error) {
    return <div className="border border-zinc-800 bg-black p-4 text-zinc-200">Error: {error}</div>;
  }

  return (
    <div className="space-y-8">
      <PageHeader
        eyebrow="Managed Repository"
        title="Repository Control"
        description="Published packages, builds, and files."
        actions={[{ href: "/packages/", label: "Packages", icon: faBoxesStacked }]}
      />

      <section className="grid gap-4 xl:grid-cols-4">
        <MetricCard label="Packages" value={grouped.length} icon={faBoxesStacked} />
        <MetricCard label="Targets" value={summary.targetCount} icon={faLayerGroup} />
        <MetricCard label="Builds" value={summary.buildCount} icon={faCube} />
        <MetricCard label="Stored Size" value={formatBytes(totalBytes)} icon={faHardDrive} />
      </section>

      <section className="grid gap-6 xl:grid-cols-[1.25fr_0.85fr]">
        <article className="border border-zinc-800 bg-black p-6">
          <div className="flex items-center justify-between gap-4">
            <div>
              <div className="text-xs uppercase tracking-[0.24em] text-zinc-500">Coverage</div>
              <h2 className="mt-2 text-2xl font-semibold text-white">Repository footprint</h2>
            </div>
            <div className="border border-zinc-800 px-3 py-2 text-xs uppercase tracking-[0.2em] text-zinc-400">
              {files.length} published files
            </div>
          </div>

          <div className="mt-6">
            <div>
              <div className="mb-3 text-xs uppercase tracking-[0.22em] text-zinc-500">Build targets</div>
              <div className="grid gap-3">
                {summary.targets.map((target) => (
                  <div key={target.mockChroot} className="border border-zinc-800 bg-zinc-950/40 p-4">
                    <div className="flex items-center justify-between gap-4">
                      <div className="text-lg font-semibold font-mono text-white">{target.mockChroot}</div>
                      <span className="inline-flex items-center gap-2 border border-emerald-700/60 bg-emerald-500/10 px-3 py-1 text-xs font-medium text-emerald-300">
                        <span className="h-2 w-2 bg-emerald-400"></span>
                        Active
                      </span>
                    </div>
                    <div className="mt-4 grid gap-3 sm:grid-cols-3">
                      <MiniStat label="Packages" value={target.packageCount} />
                      <MiniStat label="Builds" value={target.buildCount} />
                      <MiniStat label="Size" value={formatBytes(target.sizeBytes)} />
                    </div>
                  </div>
                ))}
              </div>
            </div>
          </div>
        </article>

        <article className="border border-zinc-800 bg-black p-6">
          <div className="flex items-center justify-between gap-4">
            <div>
              <div className="text-xs uppercase tracking-[0.24em] text-zinc-500">Recent output</div>
              <h2 className="mt-2 text-2xl font-semibold text-white">Latest published files</h2>
            </div>
            <FaIcon icon={faFolderTree} className="text-zinc-500" />
          </div>

          {recentFiles.length === 0 ? (
            <div className="mt-6">
              <EmptyState>No published files have been recorded yet.</EmptyState>
            </div>
          ) : (
            <div className="mt-6 space-y-3">
              {recentFiles.map((file) => (
                <div key={`${file.job_id}:${file.repo_path}`} className="border border-zinc-800 bg-zinc-950/40 p-4">
                  <div className="flex items-start justify-between gap-4">
                    <div className="min-w-0">
                      <div className="truncate font-mono text-sm text-white">{file.repo_path}</div>
                      <div className="mt-2 flex flex-wrap gap-2 text-xs uppercase tracking-[0.18em] text-zinc-500">
                        <span>{file.package_name}</span>
                        <span>{file.kind}</span>
                        <span>{formatBytes(file.size_bytes)}</span>
                      </div>
                    </div>
                    <a
                      href={`/repo/${file.repo_path}`}
                      className="inline-flex items-center border border-zinc-800 px-3 py-2 text-sm text-zinc-200 transition hover:border-zinc-600 hover:bg-zinc-950"
                    >
                      <FaIcon icon={faDownload} className="mr-2" />
                      Download
                    </a>
                  </div>
                  <div className="mt-3 text-xs text-zinc-500">{formatDateTime(file.published_at)}</div>
                </div>
              ))}
            </div>
          )}
        </article>
      </section>

      {grouped.length === 0 ? (
        <EmptyState>No managed repository files are published yet.</EmptyState>
      ) : (
        <section className="space-y-4">
          <div className="flex items-end justify-between gap-4">
            <div>
              <div className="text-xs uppercase tracking-[0.24em] text-zinc-500">Ownership</div>
              <h2 className="mt-2 text-2xl font-semibold text-white">Package publication map</h2>
            </div>
            <div className="text-sm text-zinc-500">Collapsed by default for faster navigation</div>
          </div>

          {grouped.map((pkg) => {
            const packageFiles = flattenPackageFiles(pkg);
            const packageBuildCount = pkg.targets.reduce((sum, target) => sum + target.builds.length, 0);
            const packageSize = packageFiles.reduce((sum, file) => sum + file.size_bytes, 0);

            return (
              <details key={pkg.packageName} className="border border-zinc-800 bg-black">
                <summary className="cursor-pointer list-none px-5 py-5">
                  <div className="flex flex-col gap-4 xl:flex-row xl:items-start xl:justify-between">
                    <div className="min-w-0">
                      <div className="text-xs uppercase tracking-[0.24em] text-zinc-500">Package</div>
                      <div className="mt-2 text-2xl font-semibold text-white">{pkg.packageName}</div>
                      <div className="mt-3 flex flex-wrap gap-2">
                        {pkg.targets.map((target) => (
                          <span
                            key={`${pkg.packageName}:${target.mockChroot}`}
                            className="inline-flex items-center gap-2 border border-zinc-800 px-3 py-1 text-xs uppercase tracking-[0.18em] text-zinc-400"
                          >
                            <span className="h-2 w-2 bg-emerald-400"></span>
                            {target.mockChroot}
                          </span>
                        ))}
                      </div>
                    </div>

                    <div className="grid gap-3 sm:grid-cols-4 xl:min-w-[520px]">
                      <MiniStat label="Targets" value={pkg.targets.length} />
                      <MiniStat label="Builds" value={packageBuildCount} />
                      <MiniStat label="Files" value={packageFiles.length} />
                      <MiniStat label="Size" value={formatBytes(packageSize)} />
                    </div>
                  </div>
                </summary>

                <div className="border-t border-zinc-800 p-5">
                  <div className="mb-5 flex flex-wrap gap-2">
                    <a
                      href={`/packages/view/?name=${encodeURIComponent(pkg.packageName)}`}
                      className="inline-flex items-center border border-zinc-800 px-4 py-2 text-sm text-zinc-200 transition hover:border-zinc-600 hover:bg-zinc-950"
                    >
                      <FaIcon icon={faUpRightFromSquare} className="mr-2" />
                      Open package
                    </a>
                  </div>

                  <div className="space-y-3">
                    {pkg.targets.map((target) => {
                      const targetSize = target.builds
                        .flatMap((build) => build.files)
                        .reduce((sum, file) => sum + file.size_bytes, 0);
                      return (
                        <details key={`${pkg.packageName}:${target.mockChroot}`} className="border border-zinc-800 bg-zinc-950/40">
                          <summary className="cursor-pointer list-none px-4 py-4">
                            <div className="flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between">
                              <div>
                                <div className="text-xs uppercase tracking-[0.2em] text-zinc-500">Build target</div>
                                <div className="mt-1 text-lg font-semibold font-mono text-white">{target.mockChroot}</div>
                              </div>
                              <div className="grid gap-3 sm:grid-cols-3 lg:min-w-[360px]">
                                <MiniStat label="Builds" value={target.builds.length} />
                                <MiniStat
                                  label="Files"
                                  value={target.builds.reduce((sum, build) => sum + build.files.length, 0)}
                                />
                                <MiniStat label="Size" value={formatBytes(targetSize)} />
                              </div>
                            </div>
                          </summary>

                          <div className="border-t border-zinc-800 p-4 space-y-3">
                            {target.builds.map((build) => {
                              const latestPublishedAt = build.files[0]?.published_at ?? null;
                              const buildSize = build.files.reduce((sum, file) => sum + file.size_bytes, 0);
                              return (
                                <details key={build.jobId} className="border border-zinc-800 bg-zinc-950/40">
                                  <summary className="cursor-pointer list-none px-4 py-4">
                                    <div className="flex flex-col gap-3 xl:flex-row xl:items-center xl:justify-between">
                                      <div className="min-w-0">
                                        <div className="text-xs uppercase tracking-[0.2em] text-zinc-500">Build</div>
                                        <div className="mt-1 font-mono text-sm text-white">{build.jobId}</div>
                                        <div className="mt-2 text-xs text-zinc-500">
                                          {latestPublishedAt ? formatDateTime(latestPublishedAt) : "No publish timestamp"}
                                        </div>
                                      </div>
                                      <div className="flex flex-wrap items-center gap-3">
                                        <MiniStat label="Files" value={build.files.length} />
                                        <MiniStat label="Size" value={formatBytes(buildSize)} />
                                        <a
                                          href={`/jobs/view/?id=${encodeURIComponent(build.jobId)}`}
                                          className="inline-flex items-center border border-zinc-800 px-4 py-2 text-sm text-zinc-200 transition hover:border-zinc-600 hover:bg-zinc-950"
                                          onClick={(event) => event.stopPropagation()}
                                        >
                                          <FaIcon icon={faUpRightFromSquare} className="mr-2" />
                                          Open build
                                        </a>
                                      </div>
                                    </div>
                                  </summary>

                                  <div className="border-t border-zinc-800">
                                    <div className="overflow-x-auto">
                                      <table className="min-w-[920px] w-full">
                                        <thead className="bg-zinc-950 text-left text-xs uppercase tracking-[0.2em] text-zinc-500">
                                          <tr>
                                            <th className="px-4 py-3">Repo Path</th>
                                            <th className="px-4 py-3">Kind</th>
                                            <th className="px-4 py-3">Target</th>
                                            <th className="px-4 py-3">Size</th>
                                            <th className="px-4 py-3">Published</th>
                                            <th className="px-4 py-3">Actions</th>
                                          </tr>
                                        </thead>
                                        <tbody className="divide-y divide-white/8">
                                          {build.files.map((file) => (
                                            <tr key={`${file.job_id}:${file.repo_path}`} className="hover:bg-zinc-950">
                                              <td className="px-4 py-3 font-mono text-sm text-zinc-200">{file.repo_path}</td>
                                              <td className="px-4 py-3 text-sm uppercase tracking-[0.18em] text-zinc-500">{file.kind}</td>
                                              <td className="px-4 py-3 font-mono text-sm text-zinc-400">{file.mock_chroot || "unknown"}</td>
                                              <td className="px-4 py-3 text-sm text-zinc-400">{formatBytes(file.size_bytes)}</td>
                                              <td className="px-4 py-3 text-sm text-zinc-400">{formatDateTime(file.published_at)}</td>
                                              <td className="px-4 py-3">
                                                <div className="flex flex-wrap gap-2">
                                                  <a
                                                    href={`/repo/${file.repo_path}`}
                                                    className="inline-flex items-center border border-zinc-800 px-3 py-2 text-sm text-zinc-200 transition hover:border-zinc-600 hover:bg-zinc-950"
                                                  >
                                                    <FaIcon icon={faDownload} className="mr-2" />
                                                    Download
                                                  </a>
                                                  <a
                                                    href={`/packages/view/?name=${encodeURIComponent(file.package_name)}`}
                                                    className="inline-flex items-center border border-zinc-800 px-3 py-2 text-sm text-zinc-400 transition hover:border-zinc-600 hover:bg-zinc-950 hover:text-zinc-200"
                                                  >
                                                    <FaIcon icon={faBoxesStacked} className="mr-2" />
                                                    Package
                                                  </a>
                                                </div>
                                              </td>
                                            </tr>
                                          ))}
                                        </tbody>
                                      </table>
                                    </div>
                                  </div>
                                </details>
                              );
                            })}
                          </div>
                        </details>
                      );
                    })}
                  </div>
                </div>
              </details>
            );
          })}
        </section>
      )}
    </div>
  );
}

function MetricCard({
  label,
  value,
  icon,
}: {
  label: string;
  value: string | number;
  icon: typeof faBoxesStacked;
}) {
  return (
    <article className="border border-zinc-800 bg-black p-5">
      <div className="flex items-center justify-between gap-4">
        <div className="text-xs uppercase tracking-[0.24em] text-zinc-400">{label}</div>
        <FaIcon icon={icon} className="text-zinc-500" />
      </div>
      <div className="mt-4 text-4xl font-semibold tracking-tight text-white">{value}</div>
    </article>
  );
}

function MiniStat({ label, value }: { label: string; value: string | number }) {
  return (
    <div className="border border-zinc-800 bg-black px-3 py-3">
      <div className="text-[10px] uppercase tracking-[0.18em] text-zinc-500">{label}</div>
      <div className="mt-2 text-sm font-medium text-zinc-200">{value}</div>
    </div>
  );
}

function summarizeInventory(grouped: RepoPackageGroup[]) {
  const targetMap = new Map<string, { packageNames: Set<string>; buildIds: Set<string>; sizeBytes: number }>();
  let buildCount = 0;

  for (const pkg of grouped) {
    for (const target of pkg.targets) {
      const targetEntry =
        targetMap.get(target.mockChroot) ??
        { packageNames: new Set<string>(), buildIds: new Set<string>(), sizeBytes: 0 };
      targetEntry.packageNames.add(pkg.packageName);

      for (const build of target.builds) {
        buildCount += 1;
        targetEntry.buildIds.add(build.jobId);
        for (const file of build.files) {
          targetEntry.sizeBytes += file.size_bytes;
        }
      }

      targetMap.set(target.mockChroot, targetEntry);
    }
  }

  return {
    targetCount: targetMap.size,
    buildCount,
    targets: Array.from(targetMap.entries())
      .sort(([left], [right]) => right.localeCompare(left))
      .map(([mockChroot, entry]) => ({
        mockChroot,
        packageCount: entry.packageNames.size,
        buildCount: entry.buildIds.size,
        sizeBytes: entry.sizeBytes,
      })),
  };
}

function groupRepoFiles(files: PublishedRepoFile[]): RepoPackageGroup[] {
  const packages = new Map<string, Map<string, Map<string, PublishedRepoFile[]>>>();

  for (const file of files) {
    const target = file.mock_chroot || "unknown";
    const packageMap = packages.get(file.package_name) ?? new Map();
    packages.set(file.package_name, packageMap);
    const targetMap = packageMap.get(target) ?? new Map();
    packageMap.set(target, targetMap);
    const buildFiles = targetMap.get(file.job_id) ?? [];
    buildFiles.push(file);
    targetMap.set(file.job_id, buildFiles);
  }

  return Array.from(packages.entries()).map(([packageName, targetMap]) => ({
    packageName,
    targets: Array.from(targetMap.entries())
      .sort(([left], [right]) => right.localeCompare(left))
      .map(([mockChroot, buildMap]) => ({
        mockChroot,
        builds: Array.from(buildMap.entries())
          .sort(([left], [right]) => left.localeCompare(right))
          .map(([jobId, groupedFiles]) => ({
            jobId,
            files: groupedFiles.sort((a, b) => a.repo_path.localeCompare(b.repo_path)),
          })),
      })),
  }));
}

function flattenPackageFiles(pkg: RepoPackageGroup) {
  return pkg.targets.flatMap((target) => target.builds.flatMap((build) => build.files));
}

function formatBytes(bytes: number) {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KiB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MiB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GiB`;
}
