import { useEffect, useMemo, useState, type FormEvent } from "react";
import api from "../../lib/api";
import {
  summarizePackageAction,
  summarizePackageTargetAction,
} from "../../lib/package-actions";
import ErrorMessage from "../common/ErrorMessage";
import LoadingBlock from "../ui/LoadingBlock";
import PackageBuildHistorySection from "./PackageBuildHistorySection";
import PackageEditFormSection, {
  type PackageEditFormState,
} from "./PackageEditFormSection";
import PackageDetailHeader from "./PackageDetailHeader";
import PackageRepoFilesSection from "./PackageRepoFilesSection";
import PackageStateSidebar from "./PackageStateSidebar";
import type {
  BuildEnvVar,
  PackageBuildInventoryEntry,
  PackageResponse,
  PublishedRepoFile,
  SpecSource,
  UpdatePackageRequest,
} from "../../lib/types";

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
  const [buildsOpen, setBuildsOpen] = useState(false);
  const [repoFilesOpen, setRepoFilesOpen] = useState(false);
  const [buildsLoaded, setBuildsLoaded] = useState(false);
  const [repoFilesLoaded, setRepoFilesLoaded] = useState(false);
  const [buildsLoading, setBuildsLoading] = useState(false);
  const [repoFilesLoading, setRepoFilesLoading] = useState(false);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [deleting, setDeleting] = useState(false);
  const [deletingJobId, setDeletingJobId] = useState<string | null>(null);
  const [availableChroots, setAvailableChroots] = useState<string[]>([]);
  const [showSpecPicker, setShowSpecPicker] = useState(false);
  const [showChrootPicker, setShowChrootPicker] = useState(false);
  const [browseFiles, setBrowseFiles] = useState<string[]>([]);
  const [browseError, setBrowseError] = useState<string | null>(null);
  const [browsing, setBrowsing] = useState(false);
  const [form, setForm] = useState<PackageEditFormState>({
    repoUrl: "",
    specPath: "",
    poll: true,
    mockChroots: [] as string[],
    pollIntervalSeconds: "900",
    buildTimeoutSeconds: "7200",
    packageHistoryCount: "3",
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

  function applyPackageState(packageRes: PackageResponse) {
    setPkg(packageRes);
    setForm({
      repoUrl: packageRes.package.source.repo_url,
      specPath: packageRes.package.source.spec_file,
      poll: packageRes.package.source.poll,
      mockChroots: packageRes.package.mock_chroots,
      pollIntervalSeconds: String(packageRes.package.poll_interval_seconds),
      buildTimeoutSeconds: String(packageRes.package.build_timeout_seconds),
      packageHistoryCount: String(packageRes.package.package_history_count),
      buildEnv: encodeBuildEnv(packageRes.package.build_env),
      enabled: packageRes.package.enabled,
      publish_srpm: packageRes.package.publish_srpm,
      publish_debuginfo: packageRes.package.publish_debuginfo,
      network_access: packageRes.package.network_access,
    });
  }

  async function loadPrimary() {
    try {
      setLoading(true);
      const packageRes = await api.getPackage(packageName);
      api
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
      const buildsRes = await api.getPackageBuilds(
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
      const repoFilesRes = await api.getRepoInventory(
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
    if (buildsOpen) {
      await loadBuildHistory(true);
    }
    if (repoFilesOpen) {
      await loadPackageRepoFiles(true);
    }
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
    setBuildsOpen(false);
    setRepoFilesOpen(false);
    setBuildsLoaded(false);
    setRepoFilesLoaded(false);
    loadPrimary();
  }, [packageName]);

  useEffect(() => {
    if (buildsOpen && !buildsLoaded) {
      void loadBuildHistory();
    }
  }, [buildsOpen, buildsLoaded, packageName]);

  useEffect(() => {
    if (repoFilesOpen && !repoFilesLoaded) {
      void loadPackageRepoFiles();
    }
  }, [repoFilesOpen, repoFilesLoaded, packageName]);

  async function handleSave(event: FormEvent) {
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
        build_env: parseBuildEnv(form.buildEnv),
      };
      await api.updatePackage(packageName, request);
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
      await api.deletePackage(packageName);
      window.location.href = "/packages/";
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to delete package");
      setDeleting(false);
    }
  }

  async function trigger(action: "rebuild" | "refresh") {
    try {
      const response =
        action === "rebuild"
          ? await api.rebuildPackage(packageName)
          : await api.refreshPackage(packageName);
      alert(summarizePackageAction(response));
      await refreshVisibleData();
    } catch (e) {
      setError(e instanceof Error ? e.message : `Failed to ${action}`);
    }
  }

  async function triggerBuildTarget(
    mockChroot: string,
    action: "rebuild" | "refresh",
  ) {
    try {
      const response =
        action === "rebuild"
          ? await api.rebuildPackageTarget(packageName, mockChroot)
          : await api.refreshPackageTarget(packageName, mockChroot);
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
      await api.deleteJob(jobId);
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
      const response = await api.browseRepository({
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
    <div className="space-y-8">
      <PackageDetailHeader
        packageName={pkg.package.name}
        description={pkg.package.description || "No description"}
        deleting={deleting}
        onRefresh={() => void trigger("refresh")}
        onRebuild={() => void trigger("rebuild")}
        onDelete={() => void handleDelete()}
      />

      <section className="grid gap-6 xl:grid-cols-[minmax(0,1.1fr)_minmax(320px,0.9fr)]">
        <PackageEditFormSection
          form={form}
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

        <PackageStateSidebar pkg={pkg} />
      </section>

      <PackageBuildHistorySection
        buildsLoaded={buildsLoaded}
        buildsTotal={buildsTotal}
        buildsOpen={buildsOpen}
        onToggleOpen={() => setBuildsOpen((current) => !current)}
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

      <PackageRepoFilesSection
        repoFilesLoaded={repoFilesLoaded}
        repoFilesTotal={repoFilesTotal}
        repoFilesOpen={repoFilesOpen}
        onToggleOpen={() => setRepoFilesOpen((current) => !current)}
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
