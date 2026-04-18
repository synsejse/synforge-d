import { useEffect, useMemo, useState, type SyntheticEvent } from "react";
import { packagesApi } from "../api";
import {
  summarizePackageAction,
  summarizePackageTargetAction,
} from "../../../lib/package-actions";
import ErrorMessage from "../../../components/common/ErrorMessage";
import LoadingBlock from "../../../components/ui/LoadingBlock";
import PackageBuildHistorySection from "./PackageBuildHistorySection";
import PackageEditFormSection, {
  type PackageEditFormState,
} from "./PackageEditFormSection";
import PackageDetailHeader from "./PackageDetailHeader";
import PackageRepoFilesSection from "./PackageRepoFilesSection";
import PackageStateSidebar from "./PackageStateSidebar";
import SyncHistoryTable from "./SyncHistoryTable";
import type {
  BuildEnvVar,
  PackageBuildInventoryEntry,
  PackageResponse,
  PublishedRepoFile,
  ServerHardwareResponse,
  SpecSource,
  UpdatePackageRequest,
} from "../../../lib/types";

interface Props {
  packageName: string;
}

function encodeBuildEnv(entries: BuildEnvVar[]) {
  return entries.map((entry) => `${entry.key}=${entry.value}`).join("\n");
}

function parseBuildEnv(input: string): BuildEnvVar[] {
  return input
    .split("\n")
    .map((line) => line.trim())
    .filter((line) => line.length > 0)
    .map((line) => {
      const separator = line.indexOf("=");
      if (separator === -1) {
        return { key: line, value: "" };
      }
      return {
        key: line.slice(0, separator).trim(),
        value: line.slice(separator + 1),
      };
    });
}

function parseUpdateCpuLimit(value: string, maxCpuCores: number | null): number {
  const trimmed = value.trim();
  if (!trimmed) {
    return 0;
  }
  const parsed = Number(trimmed);
  if (!Number.isFinite(parsed) || parsed <= 0) {
    return 0;
  }
  const millicores = Math.floor(parsed * 1000);
  if (!maxCpuCores || maxCpuCores <= 0) {
    return millicores;
  }
  return Math.min(millicores, Math.floor(maxCpuCores * 1000));
}

function parseUpdateMemoryLimit(value: string, maxMemoryMb: number | null): number {
  const trimmed = value.trim();
  if (!trimmed) {
    return 0;
  }
  const parsed = Number(trimmed);
  if (!Number.isFinite(parsed) || parsed <= 0) {
    return 0;
  }
  const memoryLimitMb = Math.floor(parsed);
  if (!maxMemoryMb || maxMemoryMb <= 0) {
    return memoryLimitMb;
  }
  return Math.min(memoryLimitMb, Math.floor(maxMemoryMb));
}

function parseOptionalMegabytes(value: string): number | undefined {
  const trimmed = value.trim();
  if (!trimmed) {
    return undefined;
  }
  const parsed = Number(trimmed);
  if (!Number.isFinite(parsed) || parsed <= 0) {
    return undefined;
  }
  return Math.floor(parsed);
}

function formatCpuLimitCores(value?: number | null): string {
  if (!value || value <= 0) {
    return "";
  }
  const cores = value / 1000;
  return Number.isInteger(cores)
    ? String(cores)
    : cores.toFixed(2).replace(/\.?0+$/, "");
}

export default function PackageDetail({ packageName }: Props) {
  const BUILD_HISTORY_PAGE_SIZE = 12;
  const REPO_FILES_PAGE_SIZE = 20;
  const [pkg, setPkg] = useState<PackageResponse | null>(null);
  const [builds, setBuilds] = useState<PackageBuildInventoryEntry[]>([]);
  const [buildsOffset, setBuildsOffset] = useState(0);
  const [buildsHasMore, setBuildsHasMore] = useState(false);
  const [buildsTotal, setBuildsTotal] = useState<number | null>(null);
  const [repoFiles, setRepoFiles] = useState<PublishedRepoFile[]>([]);
  const [repoFilesOffset, setRepoFilesOffset] = useState(0);
  const [repoFilesHasMore, setRepoFilesHasMore] = useState(false);
  const [repoFilesTotal, setRepoFilesTotal] = useState<number | null>(null);
  const [buildsLoaded, setBuildsLoaded] = useState(false);
  const [repoFilesLoaded, setRepoFilesLoaded] = useState(false);
  const [buildsLoading, setBuildsLoading] = useState(false);
  const [repoFilesLoading, setRepoFilesLoading] = useState(false);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [deleting, setDeleting] = useState(false);
  const [refreshing, setRefreshing] = useState(false);
  const [deletingJobId, setDeletingJobId] = useState<string | null>(null);
  const [availableChroots, setAvailableChroots] = useState<string[]>([]);
  const [showSpecPicker, setShowSpecPicker] = useState(false);
  const [showChrootPicker, setShowChrootPicker] = useState(false);
  const [browseFiles, setBrowseFiles] = useState<string[]>([]);
  const [browseError, setBrowseError] = useState<string | null>(null);
  const [browsing, setBrowsing] = useState(false);
  const [serverHardware, setServerHardware] =
    useState<ServerHardwareResponse | null>(null);
  const [form, setForm] = useState<PackageEditFormState>({
    repoUrl: "",
    specPath: "",
    poll: true,
    mockChroots: [] as string[],
    pollIntervalSeconds: "900",
    buildTimeoutSeconds: "7200",
    packageHistoryCount: "3",
    cpuLimitCores: "",
    cpuLimitEnabled: false,
    memoryLimitEnabled: false,
    memoryLimitMb: "",
    ccache_enabled: false,
    ccacheMaxSizeMb: "",
    buildEnv: "",
    enabled: true,
    publish_srpm: true,
    publish_debuginfo: true,
    network_access: false,
  });

  const selectableFiles = useMemo(
    () => browseFiles.filter((file) => file.endsWith(".spec")),
    [browseFiles],
  );
  const maxCpuCores = serverHardware?.cpu_cores ?? null;
  const maxMemoryMb = serverHardware?.total_memory_mb ?? null;

  function applyPackageState(packageRes: PackageResponse) {
    setPkg(packageRes);
    const definition = packageRes.package;
    setForm({
      repoUrl: definition.source.repo_url,
      specPath: definition.source.spec_file,
      poll: definition.source.poll ?? true,
      mockChroots: definition.mock_chroots ?? [],
      pollIntervalSeconds: String(definition.poll_interval_seconds ?? 900),
      buildTimeoutSeconds: String(definition.build_timeout_seconds ?? 7200),
      packageHistoryCount: String(definition.package_history_count ?? 3),
      cpuLimitCores: formatCpuLimitCores(definition.cpu_limit_millicores),
      cpuLimitEnabled:
        Number(definition.cpu_limit_millicores ?? 0) > 0,
      memoryLimitEnabled:
        Number(definition.memory_limit_mb ?? 0) > 0,
      memoryLimitMb: definition.memory_limit_mb
        ? String(definition.memory_limit_mb)
        : "",
      ccache_enabled: definition.ccache_enabled ?? false,
      ccacheMaxSizeMb: definition.ccache_max_size_mb
        ? String(definition.ccache_max_size_mb)
        : "",
      buildEnv: encodeBuildEnv(definition.build_env ?? []),
      enabled: definition.enabled ?? true,
      publish_srpm: definition.publish_srpm ?? true,
      publish_debuginfo: definition.publish_debuginfo ?? true,
      network_access: definition.network_access ?? false,
    });
  }

  async function loadPrimary() {
    try {
      setLoading(true);
      const packageRes = await packagesApi.getPackage(packageName);
      packagesApi
        .getServerHardware()
        .then((response) => setServerHardware(response))
        .catch(() => undefined);
      packagesApi
        .listMockChroots()
        .then((response) => setAvailableChroots(response.chroots))
        .catch(() => undefined);
      applyPackageState(packageRes);
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to load package");
    } finally {
      setLoading(false);
    }
  }

  async function loadBuildHistory(force = false, offset = buildsOffset) {
    if (buildsLoading || (buildsLoaded && !force && offset === buildsOffset)) {
      return;
    }
    try {
      setBuildsLoading(true);
      const buildsRes = await packagesApi.getPackageBuilds(
        packageName,
        BUILD_HISTORY_PAGE_SIZE,
        offset,
      );
      setBuilds(buildsRes.builds);
      setBuildsOffset(buildsRes.page.offset);
      setBuildsHasMore(buildsRes.page.has_more);
      setBuildsTotal(buildsRes.page.total ?? null);
      setBuildsLoaded(true);
      setError(null);
    } catch (e) {
      setError(
        e instanceof Error ? e.message : "Failed to load package build history",
      );
    } finally {
      setBuildsLoading(false);
    }
  }

  async function loadPackageRepoFiles(force = false, offset = repoFilesOffset) {
    if (repoFilesLoading || (repoFilesLoaded && !force && offset === repoFilesOffset)) {
      return;
    }
    try {
      setRepoFilesLoading(true);
      const repoFilesRes = await packagesApi.getRepoInventory(
        REPO_FILES_PAGE_SIZE,
        offset,
        {
          packageName,
        },
      );
      setRepoFiles(repoFilesRes.repo_files);
      setRepoFilesOffset(repoFilesRes.page.offset);
      setRepoFilesHasMore(repoFilesRes.page.has_more);
      setRepoFilesTotal(repoFilesRes.page.total ?? null);
      setRepoFilesLoaded(true);
      setError(null);
    } catch (e) {
      setError(
        e instanceof Error
          ? e.message
          : "Failed to load package repository files",
      );
    } finally {
      setRepoFilesLoading(false);
    }
  }

  async function refreshVisibleData() {
    await loadPrimary();
    await loadBuildHistory(true);
    await loadPackageRepoFiles(true);
  }

  useEffect(() => {
    setBuilds([]);
    setBuildsOffset(0);
    setBuildsHasMore(false);
    setBuildsTotal(null);
    setRepoFiles([]);
    setRepoFilesOffset(0);
    setRepoFilesHasMore(false);
    setRepoFilesTotal(null);
    setBuildsLoaded(false);
    setRepoFilesLoaded(false);
    loadPrimary();
    loadBuildHistory(false, 0);
    loadPackageRepoFiles(false, 0);
  }, [packageName]);



  async function handleSave(event: SyntheticEvent) {
    event.preventDefault();
    setSaving(true);
    try {
      const source: SpecSource = {
        repo_url: form.repoUrl,
        spec_file: form.specPath,
        poll: form.poll,
      };
      const request: UpdatePackageRequest = {
        source,
        enabled: form.enabled,
        publish_srpm: form.publish_srpm,
        publish_debuginfo: form.publish_debuginfo,
        network_access: form.network_access,
        mock_chroots: form.mockChroots,
        poll_interval_seconds: Number(form.pollIntervalSeconds),
        build_timeout_seconds: Number(form.buildTimeoutSeconds),
        package_history_count: Number(form.packageHistoryCount),
        cpu_limit_millicores: form.cpuLimitEnabled
          ? parseUpdateCpuLimit(form.cpuLimitCores, maxCpuCores)
          : 0,
        memory_limit_mb: form.memoryLimitEnabled
          ? parseUpdateMemoryLimit(form.memoryLimitMb, maxMemoryMb)
          : 0,
        ccache_enabled: form.ccache_enabled,
        ccache_max_size_mb: parseOptionalMegabytes(form.ccacheMaxSizeMb) ?? 0,
        build_env: parseBuildEnv(form.buildEnv),
      };
      await packagesApi.updatePackage(packageName, request);
      await loadPrimary();
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to save package");
    } finally {
      setSaving(false);
    }
  }

  async function handleDelete() {
    if (
      !confirm(
        `Delete package "${packageName}"? This removes its stored spec sources.`,
      )
    ) {
      return;
    }
    setDeleting(true);
    try {
      await packagesApi.deletePackage(packageName);
      window.location.href = "/packages/";
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to delete package");
      setDeleting(false);
    }
  }

  async function trigger(action: "rebuild" | "refresh") {
    if (action === "refresh" && refreshing) {
      return;
    }
    if (action === "refresh") {
      setRefreshing(true);
    }
    try {
      const response =
        action === "rebuild"
          ? await packagesApi.rebuildPackage(packageName)
          : await packagesApi.refreshPackage(packageName);
      alert(summarizePackageAction(response));
      await refreshVisibleData();
    } catch (e) {
      setError(e instanceof Error ? e.message : `Failed to ${action}`);
    } finally {
      if (action === "refresh") {
        setRefreshing(false);
      }
    }
  }

  async function triggerBuildTarget(
    mockChroot: string,
    action: "rebuild" | "refresh",
  ) {
    try {
      const response =
        action === "rebuild"
          ? await packagesApi.rebuildPackageTarget(packageName, mockChroot)
          : await packagesApi.refreshPackageTarget(packageName, mockChroot);
      alert(
        summarizePackageTargetAction(
          response,
          action === "rebuild" ? "Rebuild" : "Refresh",
        ),
      );
      await refreshVisibleData();
    } catch (e) {
      setError(e instanceof Error ? e.message : `Failed to ${action}`);
    }
  }

  async function handleDeleteJob(jobId: string) {
    if (
      !confirm(
        `Delete build ${jobId}? This also removes repo files published by that build.`,
      )
    ) {
      return;
    }
    setDeletingJobId(jobId);
    try {
      await packagesApi.deleteJob(jobId);
      await refreshVisibleData();
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to delete build");
    } finally {
      setDeletingJobId(null);
    }
  }

  function toggleChroot(chroot: string, checked: boolean) {
    setForm((current) => ({
      ...current,
      mockChroots: checked
        ? Array.from(new Set([...current.mockChroots, chroot]))
        : current.mockChroots.filter((value) => value !== chroot),
    }));
  }

  async function handleBrowse() {
    if (!form.repoUrl.trim()) {
      setBrowseError("Repository URL is required before browsing.");
      return;
    }
    setBrowsing(true);
    setBrowseError(null);
    try {
      const response = await packagesApi.browseRepository({
        repo_url: form.repoUrl.trim(),
      });
      setBrowseFiles(response.files);
      if (!form.specPath && response.spec_files.length > 0) {
        setForm((current) => ({
          ...current,
          specPath: response.spec_files[0],
        }));
      }
    } catch (e) {
      setBrowseError(
        e instanceof Error ? e.message : "Failed to browse repository",
      );
    } finally {
      setBrowsing(false);
    }
  }

  if (loading) {
    return <LoadingBlock label="Loading package…" lines={4} />;
  }

  if (error || !pkg) {
    return <ErrorMessage message={error || "Failed to load package"} />;
  }

  return (
    <div className="min-w-0 space-y-8">
      <PackageDetailHeader
        packageName={pkg.package.name}
        description={pkg.package.description || "No description"}
        deleting={deleting}
        refreshing={refreshing}
        onRefresh={() => void trigger("refresh")}
        onRebuild={() => void trigger("rebuild")}
        onDelete={() => void handleDelete()}
      />

      <section className="grid min-w-0 gap-6 xl:grid-cols-[minmax(0,1.1fr)_minmax(320px,0.9fr)]">
        <div className="min-w-0">
          <PackageEditFormSection
            form={form}
            maxCpuCores={maxCpuCores}
            maxMemoryMb={maxMemoryMb}
            saving={saving}
            availableChroots={availableChroots}
            showSpecPicker={showSpecPicker}
            showChrootPicker={showChrootPicker}
            browsing={browsing}
            browseError={browseError}
            selectableFiles={selectableFiles}
            onSubmit={handleSave}
            onFormChange={(next) =>
              setForm((current) => ({
                ...current,
                ...next,
              }))
            }
            onToggleChroot={toggleChroot}
            onOpenSpecPicker={() => setShowSpecPicker(true)}
            onCloseSpecPicker={() => setShowSpecPicker(false)}
            onOpenChrootPicker={() => setShowChrootPicker(true)}
            onCloseChrootPicker={() => setShowChrootPicker(false)}
            onBrowseRepository={() => void handleBrowse()}
          />
        </div>

        <div className="min-w-0">
          <PackageStateSidebar pkg={pkg} />
        </div>
      </section>

      <PackageBuildHistorySection
        buildsLoaded={buildsLoaded}
        buildsTotal={buildsTotal}
        buildsLoading={buildsLoading}
        builds={builds}
        buildsOffset={buildsOffset}
        buildsHasMore={buildsHasMore}
        onLoadPrevious={() =>
          void loadBuildHistory(
            true,
            Math.max(0, buildsOffset - BUILD_HISTORY_PAGE_SIZE),
          )
        }
        onLoadNext={() =>
          void loadBuildHistory(true, buildsOffset + BUILD_HISTORY_PAGE_SIZE)
        }
        onRefreshTarget={(mockChroot) =>
          void triggerBuildTarget(mockChroot, "refresh")
        }
        onRebuildTarget={(mockChroot) =>
          void triggerBuildTarget(mockChroot, "rebuild")
        }
        onDeleteJob={(jobId) => void handleDeleteJob(jobId)}
        deletingJobId={deletingJobId}
      />

      <section className="border-4 border-[var(--theme-border-strong)] bg-black p-6 shadow-[6px_6px_0_rgba(255,255,255,0.2)]">
        <div className="mb-5">
          <h2 className="font-mono text-xl font-bold uppercase text-white">Sync History</h2>
          <p className="mt-2 text-sm text-zinc-400">
            Source sync outcomes for this package across poll and manual triggers.
          </p>
        </div>
        <SyncHistoryTable packageName={packageName} />
      </section>

      <PackageRepoFilesSection
        repoFilesLoaded={repoFilesLoaded}
        repoFilesTotal={repoFilesTotal}
        repoFilesLoading={repoFilesLoading}
        repoFiles={repoFiles}
        repoFilesOffset={repoFilesOffset}
        repoFilesHasMore={repoFilesHasMore}
        onLoadPrevious={() =>
          void loadPackageRepoFiles(
            true,
            Math.max(0, repoFilesOffset - REPO_FILES_PAGE_SIZE),
          )
        }
        onLoadNext={() =>
          void loadPackageRepoFiles(true, repoFilesOffset + REPO_FILES_PAGE_SIZE)
        }
      />
    </div>
  );
}
