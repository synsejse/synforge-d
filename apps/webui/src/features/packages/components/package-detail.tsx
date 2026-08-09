import { useEffect, useMemo, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import api from "../../../lib/api";
import { packagesQueries } from "../../../lib/queries";
import { summarizeSyncEnqueue } from "../../../lib/package-actions";
import ErrorMessage from "../../../components/common/error-message";
import { useDialogs } from "../../../components/common/dialogs-context";
import { useToast } from "../../../components/common/toast-context";
import { useServerHardware } from "../../../components/common/server-hardware-provider";
import LoadingBlock from "../../../components/ui/loading-block";
import Breadcrumbs from "../../../components/ui/breadcrumbs";
import PackageEditFormSection from "./package-edit-form-section";
import type { PackageEditFormState } from "./package-edit-form-state";
import {
  EMPTY_FORM,
  buildFormFromPackage,
  buildUpdateRequest,
} from "./package-detail-form";
import PackageDetailHeader from "./package-detail-header";
import PackageDetailTabs, {
  type PackageDetailTab,
} from "./package-detail-tabs";
import PackageStatusStrip from "./package-status-strip";

interface Props {
  packageName: string;
}

const BUILD_HISTORY_PAGE_SIZE = 12;
const REPO_FILES_PAGE_SIZE = 20;

export default function PackageDetail({ packageName }: Props) {
  const queryClient = useQueryClient();
  const navigate = useNavigate();
  const { confirm } = useDialogs();
  const toast = useToast();
  const serverHardware = useServerHardware();

  const packageQuery = useQuery(packagesQueries.detail(packageName));
  const chrootsQuery = useQuery(packagesQueries.mockChroots());

  const [buildsOffset, setBuildsOffset] = useState(0);
  const [repoFilesOffset, setRepoFilesOffset] = useState(0);
  const [activeTab, setActiveTab] = useState<PackageDetailTab>("builds");
  const [includeDeletedBuilds, setIncludeDeletedBuilds] = useState(false);

  const buildsQuery = useQuery(
    packagesQueries.builds(packageName, {
      limit: BUILD_HISTORY_PAGE_SIZE,
      offset: buildsOffset,
      includeDeleted: includeDeletedBuilds,
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
  const [pristine, setPristine] = useState<PackageEditFormState | null>(null);
  const [formInitialized, setFormInitialized] = useState(false);
  const [showSpecPicker, setShowSpecPicker] = useState(false);
  const [showChrootPicker, setShowChrootPicker] = useState(false);
  const [browseFiles, setBrowseFiles] = useState<string[]>([]);
  const [browseError, setBrowseError] = useState<string | null>(null);

  useEffect(() => {
    if (!formInitialized && packageQuery.data) {
      const next = buildFormFromPackage(packageQuery.data);
      setForm(next);
      setPristine(next);
      setFormInitialized(true);
    }
  }, [formInitialized, packageQuery.data]);

  useEffect(() => {
    setBuildsOffset(0);
    setRepoFilesOffset(0);
    setFormInitialized(false);
    setForm(EMPTY_FORM);
    setPristine(null);
  }, [packageName]);

  // After a successful save the API result becomes the new pristine.
  useEffect(() => {
    if (formInitialized && packageQuery.data) {
      setPristine(buildFormFromPackage(packageQuery.data));
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [packageQuery.dataUpdatedAt]);

  const selectableFiles = useMemo(
    () => browseFiles.filter((file) => file.endsWith(".spec")),
    [browseFiles],
  );
  const maxCpuCores = serverHardware?.cpu_cores ?? null;
  const maxMemoryMb = serverHardware?.total_memory_mb ?? null;

  const invalidatePackage = () =>
    queryClient.invalidateQueries({ queryKey: ["packages"] });

  const saveMutation = useMutation({
    mutationFn: () =>
      api.updatePackage(
        packageName,
        buildUpdateRequest(form, maxCpuCores, maxMemoryMb),
      ),
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
    onSuccess: (response, variables) => {
      toast.toast({
        title:
          variables.action === "rebuild" ? "Rebuild queued" : "Refresh queued",
        message: summarizeSyncEnqueue(response),
        variant: "success",
        action: {
          label: "Open sync",
          href: `/syncs/view?id=${encodeURIComponent(response.operation.id)}`,
        },
      });
      void invalidatePackage();
    },
    onError: (err, variables) =>
      setError(
        err instanceof Error ? err.message : `Failed to ${variables.action}`,
      ),
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
    onSuccess: (response, variables) => {
      toast.toast({
        title:
          variables.action === "rebuild" ? "Rebuild queued" : "Refresh queued",
        message: summarizeSyncEnqueue(response),
        variant: "success",
        action: {
          label: "Open sync",
          href: `/syncs/view?id=${encodeURIComponent(response.operation.id)}`,
        },
      });
      void invalidatePackage();
    },
    onError: (err, variables) =>
      setError(
        err instanceof Error
          ? err.message
          : `Failed to ${variables.action} target`,
      ),
  });

  const deleteJobMutation = useMutation({
    mutationFn: (jobId: string) => api.deleteJob(jobId),
    onSuccess: invalidatePackage,
    onError: (err) =>
      setError(err instanceof Error ? err.message : "Failed to delete build"),
  });

  const browseMutation = useMutation({
    mutationFn: (repoUrl: string) =>
      api.browseRepository({ repo_url: repoUrl }),
    onSuccess: (response) => {
      setBrowseFiles(response.files);
      setBrowseError(null);
      if (!form.specPath && response.spec_files.length > 0) {
        setForm((current) => ({
          ...current,
          specPath: response.spec_files[0],
        }));
      }
    },
    onError: (err) =>
      setBrowseError(
        err instanceof Error ? err.message : "Failed to browse repository",
      ),
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

  function handleSave() {
    saveMutation.mutate();
  }

  if (packageQuery.isPending) {
    return (
      <div className="min-w-0 space-y-6">
        <Breadcrumbs
          items={[
            { label: "Packages", to: "/packages" },
            { label: packageName },
          ]}
        />
        <LoadingBlock label="Loading package…" lines={4} />
      </div>
    );
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
    triggerMutation.isPending &&
    triggerMutation.variables?.action === "refresh";

  const buildsTotal = buildsQuery.data?.page.total ?? null;
  const repoFilesTotal = repoFilesQuery.data?.page.total ?? null;

  return (
    <div className="min-w-0 space-y-6">
      <Breadcrumbs
        items={[
          { label: "Packages", to: "/packages" },
          { label: packageQuery.data.package.name },
        ]}
      />
      {error ? <ErrorMessage message={error} /> : null}
      <PackageDetailHeader
        packageName={packageQuery.data.package.name}
        description={packageQuery.data.package.description || ""}
        deleting={deleteMutation.isPending}
        refreshing={refreshing}
        onRefresh={() => trigger("refresh")}
        onRebuild={() => trigger("rebuild")}
        onDelete={() => void handleDelete()}
      />

      <PackageStatusStrip pkg={packageQuery.data} />

      <PackageEditFormSection
        form={form}
        pristine={pristine}
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
        onDiscard={() => pristine && setForm(pristine)}
      />

      <PackageDetailTabs
        packageName={packageName}
        activeTab={activeTab}
        onTabChange={setActiveTab}
        buildsLoaded={!buildsQuery.isPending}
        buildsTotal={buildsTotal}
        buildsLoading={buildsQuery.isFetching}
        builds={buildsQuery.data?.builds ?? []}
        buildsOffset={buildsOffset}
        buildsPageSize={BUILD_HISTORY_PAGE_SIZE}
        buildsHasMore={buildsQuery.data?.page.has_more ?? false}
        includeDeleted={includeDeletedBuilds}
        ccacheEnabled={packageQuery.data.package.ccache_enabled ?? false}
        ccacheStatsByTarget={buildsQuery.data?.ccache_stats_by_target ?? []}
        deletingJobId={
          deleteJobMutation.isPending && deleteJobMutation.variables
            ? deleteJobMutation.variables
            : null
        }
        onIncludeDeletedChange={(next) => {
          setIncludeDeletedBuilds(next);
          setBuildsOffset(0);
        }}
        onBuildsOffsetChange={setBuildsOffset}
        onRefreshTarget={(mockChroot) =>
          triggerTargetMutation.mutate({ mockChroot, action: "refresh" })
        }
        onRebuildTarget={(mockChroot) =>
          triggerTargetMutation.mutate({ mockChroot, action: "rebuild" })
        }
        onDeleteJob={(jobId) => void handleDeleteJob(jobId)}
        repoFilesLoaded={!repoFilesQuery.isPending}
        repoFilesTotal={repoFilesTotal}
        repoFilesLoading={repoFilesQuery.isFetching}
        repoFiles={repoFilesQuery.data?.repo_files ?? []}
        repoFilesOffset={repoFilesOffset}
        repoFilesPageSize={REPO_FILES_PAGE_SIZE}
        repoFilesHasMore={repoFilesQuery.data?.page.has_more ?? false}
        onRepoFilesOffsetChange={setRepoFilesOffset}
      />
    </div>
  );
}
