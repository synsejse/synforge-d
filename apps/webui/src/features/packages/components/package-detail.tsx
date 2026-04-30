import { useEffect, useMemo, useState, type SyntheticEvent } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import api from "../../../lib/api";
import { packagesQueries } from "../../../lib/queries";
import {
  summarizePackageAction,
  summarizePackageTargetAction,
} from "../../../lib/package-actions";
import ErrorMessage from "../../../components/common/error-message";
import { useDialogs } from "../../../components/common/dialogs-provider";
import { useServerHardware } from "../../../components/common/server-hardware-provider";
import LoadingBlock from "../../../components/ui/loading-block";
import PackageBuildHistorySection from "./package-build-history-section";
import PackageEditFormSection, {
  type PackageEditFormState,
} from "./package-edit-form-section";
import PackageDetailHeader from "./package-detail-header";
import PackageRepoFilesSection from "./package-repo-files-section";
import PackageStateSidebar from "./package-state-sidebar";
import SyncHistoryTable from "./sync-history-table";
import type {
  BuildEnvVar,
  PackageResponse,
  SpecSource,
  UpdatePackageRequest,
} from "../../../lib/types";

interface Props {
  packageName: string;
}

const BUILD_HISTORY_PAGE_SIZE = 12;
const REPO_FILES_PAGE_SIZE = 20;

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
  if (!trimmed) return 0;
  const parsed = Number(trimmed);
  if (!Number.isFinite(parsed) || parsed <= 0) return 0;
  const millicores = Math.floor(parsed * 1000);
  if (!maxCpuCores || maxCpuCores <= 0) return millicores;
  return Math.min(millicores, Math.floor(maxCpuCores * 1000));
}

function parseUpdateMemoryLimit(value: string, maxMemoryMb: number | null): number {
  const trimmed = value.trim();
  if (!trimmed) return 0;
  const parsed = Number(trimmed);
  if (!Number.isFinite(parsed) || parsed <= 0) return 0;
  const memoryLimitMb = Math.floor(parsed);
  if (!maxMemoryMb || maxMemoryMb <= 0) return memoryLimitMb;
  return Math.min(memoryLimitMb, Math.floor(maxMemoryMb));
}

function parseOptionalMegabytes(value: string): number | undefined {
  const trimmed = value.trim();
  if (!trimmed) return undefined;
  const parsed = Number(trimmed);
  if (!Number.isFinite(parsed) || parsed <= 0) return undefined;
  return Math.floor(parsed);
}

function formatCpuLimitCores(value?: number | null): string {
  if (!value || value <= 0) return "";
  const cores = value / 1000;
  return Number.isInteger(cores)
    ? String(cores)
    : cores.toFixed(2).replace(/\.?0+$/, "");
}

function buildFormFromPackage(packageRes: PackageResponse): PackageEditFormState {
  const definition = packageRes.package;
  return {
    repoUrl: definition.source.repo_url,
    specPath: definition.source.spec_file,
    poll: definition.source.poll ?? true,
    mockChroots: definition.mock_chroots ?? [],
    pollIntervalSeconds: String(definition.poll_interval_seconds ?? 900),
    buildTimeoutSeconds: String(definition.build_timeout_seconds ?? 7200),
    packageHistoryCount: String(definition.package_history_count ?? 3),
    cpuLimitCores: formatCpuLimitCores(definition.cpu_limit_millicores),
    cpuLimitEnabled: Number(definition.cpu_limit_millicores ?? 0) > 0,
    memoryLimitEnabled: Number(definition.memory_limit_mb ?? 0) > 0,
    memoryLimitMb: definition.memory_limit_mb ? String(definition.memory_limit_mb) : "",
    ccache_enabled: definition.ccache_enabled ?? false,
    ccacheMaxSizeMb: definition.ccache_max_size_mb
      ? String(definition.ccache_max_size_mb)
      : "",
    buildEnv: encodeBuildEnv(definition.build_env ?? []),
    enabled: definition.enabled ?? true,
    publish_srpm: definition.publish_srpm ?? true,
    publish_debuginfo: definition.publish_debuginfo ?? true,
    network_access: definition.network_access ?? false,
  };
}

const EMPTY_FORM: PackageEditFormState = {
  repoUrl: "",
  specPath: "",
  poll: true,
  mockChroots: [],
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
};

export default function PackageDetail({ packageName }: Props) {
  const queryClient = useQueryClient();
  const navigate = useNavigate();
  const { confirm, notify } = useDialogs();
  const serverHardware = useServerHardware();

  const packageQuery = useQuery(packagesQueries.detail(packageName));
  const chrootsQuery = useQuery(packagesQueries.mockChroots());

  const [buildsOffset, setBuildsOffset] = useState(0);
  const [repoFilesOffset, setRepoFilesOffset] = useState(0);

  const buildsQuery = useQuery(
    packagesQueries.builds(packageName, {
      limit: BUILD_HISTORY_PAGE_SIZE,
      offset: buildsOffset,
    }),
  );

  const repoFilesQuery = useQuery(
    packagesQueries.repoFiles(packageName, {
      limit: REPO_FILES_PAGE_SIZE,
      offset: repoFilesOffset,
    }),
  );

  const [error, setError] = useState<string | null>(null);
  const [form, setForm] = useState<PackageEditFormState>(EMPTY_FORM);
  const [formInitialized, setFormInitialized] = useState(false);
  const [showSpecPicker, setShowSpecPicker] = useState(false);
  const [showChrootPicker, setShowChrootPicker] = useState(false);
  const [browseFiles, setBrowseFiles] = useState<string[]>([]);
  const [browseError, setBrowseError] = useState<string | null>(null);

  useEffect(() => {
    if (!formInitialized && packageQuery.data) {
      setForm(buildFormFromPackage(packageQuery.data));
      setFormInitialized(true);
    }
  }, [formInitialized, packageQuery.data]);

  useEffect(() => {
    setBuildsOffset(0);
    setRepoFilesOffset(0);
    setFormInitialized(false);
    setForm(EMPTY_FORM);
  }, [packageName]);

  const selectableFiles = useMemo(
    () => browseFiles.filter((file) => file.endsWith(".spec")),
    [browseFiles],
  );
  const maxCpuCores = serverHardware?.cpu_cores ?? null;
  const maxMemoryMb = serverHardware?.total_memory_mb ?? null;

  const invalidatePackage = () =>
    queryClient.invalidateQueries({ queryKey: ["packages"] });

  const saveMutation = useMutation({
    mutationFn: () => {
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
      return api.updatePackage(packageName, request);
    },
    onSuccess: invalidatePackage,
    onError: (err) =>
      setError(err instanceof Error ? err.message : "Failed to save package"),
  });

  const deleteMutation = useMutation({
    mutationFn: () => api.deletePackage(packageName),
    onSuccess: () => {
      navigate({ to: "/packages" });
    },
    onError: (err) =>
      setError(err instanceof Error ? err.message : "Failed to delete package"),
  });

  const triggerMutation = useMutation({
    mutationFn: ({ action }: { action: "rebuild" | "refresh" }) =>
      action === "rebuild"
        ? api.rebuildPackage(packageName)
        : api.refreshPackage(packageName),
    onSuccess: async (response, variables) => {
      await notify({
        title: variables.action === "rebuild" ? "Rebuild queued" : "Refresh queued",
        message: summarizePackageAction(response),
      });
      await invalidatePackage();
    },
    onError: (err, variables) =>
      setError(err instanceof Error ? err.message : `Failed to ${variables.action}`),
  });

  const triggerTargetMutation = useMutation({
    mutationFn: ({
      mockChroot,
      action,
    }: {
      mockChroot: string;
      action: "rebuild" | "refresh";
    }) =>
      action === "rebuild"
        ? api.rebuildPackageTarget(packageName, mockChroot)
        : api.refreshPackageTarget(packageName, mockChroot),
    onSuccess: async (response, variables) => {
      await notify({
        title: variables.action === "rebuild" ? "Rebuild queued" : "Refresh queued",
        message: summarizePackageTargetAction(
          response,
          variables.action === "rebuild" ? "Rebuild" : "Refresh",
        ),
      });
      await invalidatePackage();
    },
    onError: (err, variables) =>
      setError(
        err instanceof Error ? err.message : `Failed to ${variables.action} target`,
      ),
  });

  const deleteJobMutation = useMutation({
    mutationFn: (jobId: string) => api.deleteJob(jobId),
    onSuccess: invalidatePackage,
    onError: (err) =>
      setError(err instanceof Error ? err.message : "Failed to delete build"),
  });

  const browseMutation = useMutation({
    mutationFn: (repoUrl: string) => api.browseRepository({ repo_url: repoUrl }),
    onSuccess: (response) => {
      setBrowseFiles(response.files);
      setBrowseError(null);
      if (!form.specPath && response.spec_files.length > 0) {
        setForm((current) => ({ ...current, specPath: response.spec_files[0] }));
      }
    },
    onError: (err) =>
      setBrowseError(err instanceof Error ? err.message : "Failed to browse repository"),
  });

  async function handleDelete() {
    const ok = await confirm({
      title: "Delete package?",
      message: `Package "${packageName}" and its stored spec sources will be removed.`,
      confirmLabel: "Delete",
      destructive: true,
    });
    if (!ok) return;
    deleteMutation.mutate();
  }

  function trigger(action: "rebuild" | "refresh") {
    if (
      action === "refresh" &&
      triggerMutation.isPending &&
      triggerMutation.variables?.action === "refresh"
    ) {
      return;
    }
    triggerMutation.mutate({ action });
  }

  async function handleDeleteJob(jobId: string) {
    const ok = await confirm({
      title: "Delete build?",
      message: `Build ${jobId} and its published repo files will be removed.`,
      confirmLabel: "Delete",
      destructive: true,
    });
    if (!ok) return;
    deleteJobMutation.mutate(jobId);
  }

  function toggleChroot(chroot: string, checked: boolean) {
    setForm((current) => ({
      ...current,
      mockChroots: checked
        ? Array.from(new Set([...current.mockChroots, chroot]))
        : current.mockChroots.filter((value) => value !== chroot),
    }));
  }

  function handleBrowse() {
    if (!form.repoUrl.trim()) {
      setBrowseError("Repository URL is required before browsing.");
      return;
    }
    browseMutation.mutate(form.repoUrl.trim());
  }

  function handleSave(event: SyntheticEvent) {
    event.preventDefault();
    saveMutation.mutate();
  }

  if (packageQuery.isPending) {
    return <LoadingBlock label="Loading package…" lines={4} />;
  }

  if (packageQuery.error || !packageQuery.data) {
    return (
      <ErrorMessage
        message={
          packageQuery.error instanceof Error
            ? packageQuery.error.message
            : "Failed to load package"
        }
      />
    );
  }

  const refreshing =
    triggerMutation.isPending && triggerMutation.variables?.action === "refresh";

  return (
    <div className="min-w-0 space-y-8">
      {error ? <ErrorMessage message={error} /> : null}
      <PackageDetailHeader
        packageName={packageQuery.data.package.name}
        description={packageQuery.data.package.description || "No description"}
        deleting={deleteMutation.isPending}
        refreshing={refreshing}
        onRefresh={() => trigger("refresh")}
        onRebuild={() => trigger("rebuild")}
        onDelete={() => void handleDelete()}
      />

      <section className="grid min-w-0 gap-6 xl:grid-cols-[minmax(0,1.1fr)_minmax(320px,0.9fr)]">
        <div className="min-w-0">
          <PackageEditFormSection
            form={form}
            maxCpuCores={maxCpuCores}
            maxMemoryMb={maxMemoryMb}
            saving={saveMutation.isPending}
            availableChroots={chrootsQuery.data?.chroots ?? []}
            showSpecPicker={showSpecPicker}
            showChrootPicker={showChrootPicker}
            browsing={browseMutation.isPending}
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
            onBrowseRepository={handleBrowse}
          />
        </div>

        <div className="min-w-0">
          <PackageStateSidebar pkg={packageQuery.data} />
        </div>
      </section>

      <PackageBuildHistorySection
        buildsLoaded={!buildsQuery.isPending}
        buildsTotal={buildsQuery.data?.page.total ?? null}
        buildsLoading={buildsQuery.isFetching}
        builds={buildsQuery.data?.builds ?? []}
        buildsOffset={buildsOffset}
        buildsHasMore={buildsQuery.data?.page.has_more ?? false}
        onLoadPrevious={() =>
          setBuildsOffset(Math.max(0, buildsOffset - BUILD_HISTORY_PAGE_SIZE))
        }
        onLoadNext={() => setBuildsOffset(buildsOffset + BUILD_HISTORY_PAGE_SIZE)}
        onRefreshTarget={(mockChroot) =>
          triggerTargetMutation.mutate({ mockChroot, action: "refresh" })
        }
        onRebuildTarget={(mockChroot) =>
          triggerTargetMutation.mutate({ mockChroot, action: "rebuild" })
        }
        onDeleteJob={(jobId) => void handleDeleteJob(jobId)}
        deletingJobId={
          deleteJobMutation.isPending && deleteJobMutation.variables
            ? deleteJobMutation.variables
            : null
        }
      />

      <section className="border-4 border-[var(--theme-border-strong)] bg-black p-6 shadow-card-md">
        <div className="mb-5">
          <h2 className="font-mono text-xl font-bold uppercase text-white">Sync History</h2>
          <p className="mt-2 text-sm text-zinc-400">
            Source sync outcomes for this package across poll and manual triggers.
          </p>
        </div>
        <SyncHistoryTable packageName={packageName} />
      </section>

      <PackageRepoFilesSection
        repoFilesLoaded={!repoFilesQuery.isPending}
        repoFilesTotal={repoFilesQuery.data?.page.total ?? null}
        repoFilesLoading={repoFilesQuery.isFetching}
        repoFiles={repoFilesQuery.data?.repo_files ?? []}
        repoFilesOffset={repoFilesOffset}
        repoFilesHasMore={repoFilesQuery.data?.page.has_more ?? false}
        onLoadPrevious={() =>
          setRepoFilesOffset(Math.max(0, repoFilesOffset - REPO_FILES_PAGE_SIZE))
        }
        onLoadNext={() => setRepoFilesOffset(repoFilesOffset + REPO_FILES_PAGE_SIZE)}
      />
    </div>
  );
}
