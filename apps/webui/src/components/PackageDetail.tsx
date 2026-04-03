import { useEffect, useMemo, useState, type FormEvent } from "react";
import api from "../lib/api";
import DetailStat from "./DetailStat";
import EmptyState from "./EmptyState";
import FaIcon from "./FaIcon";
import LoadingBlock from "./LoadingBlock";
import StatusPill from "./StatusPill";
import { formatDateTime } from "../lib/datetime";
import type {
  BuildEnvVar,
  PackageBuildInventoryEntry,
  PackageResponse,
  PublishedRepoFile,
  SpecSource,
  UpdatePackageRequest,
} from "../lib/types";
import {
  faArrowLeft,
  faBoxesStacked,
  faFolderOpen,
  faHammer,
  faMagnifyingGlass,
  faRotate,
  faSave,
  faTrash,
} from "@fortawesome/free-solid-svg-icons";

interface Props {
  packageName: string;
}

function formatMockChroots(chroots: string[]) {
  return chroots.join(", ");
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
  const [pkg, setPkg] = useState<PackageResponse | null>(null);
  const [builds, setBuilds] = useState<PackageBuildInventoryEntry[]>([]);
  const [repoFiles, setRepoFiles] = useState<PublishedRepoFile[]>([]);
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
  const [form, setForm] = useState({
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
      specPath: packageRes.package.source.spec_path,
      poll: packageRes.package.source.poll,
      mockChroots: packageRes.package.mock_chroots,
      pollIntervalSeconds: String(packageRes.package.poll_interval_seconds),
      buildTimeoutSeconds: String(packageRes.package.build_timeout_seconds),
      packageHistoryCount: String(packageRes.package.package_history_count),
      buildEnv: encodeBuildEnv(packageRes.package.build_env),
      enabled: packageRes.package.enabled,
      publish_srpm: packageRes.package.publish_srpm,
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

  async function loadBuildHistory(force = false) {
    if (buildsLoading || (buildsLoaded && !force)) {
      return;
    }
    try {
      setBuildsLoading(true);
      const buildsRes = await api.getPackageBuilds(packageName);
      setBuilds(buildsRes.builds);
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

  async function loadPackageRepoFiles(force = false) {
    if (repoFilesLoading || (repoFilesLoaded && !force)) {
      return;
    }
    try {
      setRepoFilesLoading(true);
      const repoFilesRes = await api.getPackageRepoFiles(packageName);
      setRepoFiles(repoFilesRes.repo_files);
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
    setRepoFiles([]);
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
        spec_path: form.specPath,
        poll: form.poll,
      };
      const request: UpdatePackageRequest = {
        source,
        enabled: form.enabled,
        publish_srpm: form.publish_srpm,
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
      if (action === "rebuild") {
        await api.rebuildPackage(packageName);
      } else {
        await api.refreshPackage(packageName);
      }
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
    return (
      <div className="border border-zinc-800 bg-black p-4 text-zinc-200">
        Error: {error || "Failed to load package"}
      </div>
    );
  }

  const latestBuilds = builds.slice(0, 12);

  return (
    <div className="space-y-8">
      <section className="border border-zinc-800 bg-black p-6">
        <div className="flex flex-col gap-5 lg:flex-row lg:items-end lg:justify-between">
          <div className="space-y-3">
            <a
              href="/packages/"
              className="text-sm text-zinc-400 transition hover:text-zinc-100"
            >
              <FaIcon icon={faArrowLeft} className="mr-2" />
              Back to packages
            </a>
            <div>
              <p className="text-xs uppercase tracking-[0.28em] text-zinc-500">
                Package Control
              </p>
              <h1 className="mt-2 text-4xl font-semibold tracking-tight text-white">
                {pkg.package.name}
              </h1>
            </div>
            <p className="max-w-3xl text-sm leading-6 text-zinc-300">
              {pkg.package.description || "No description"}
            </p>
          </div>
          <div className="flex flex-wrap gap-3">
            <button
              onClick={() => trigger("refresh")}
              className="border border-zinc-800 bg-black px-4 py-2 text-sm font-medium text-zinc-100 transition hover:border-zinc-600 hover:bg-zinc-950"
            >
              <FaIcon icon={faRotate} className="mr-2" />
              Refresh
            </button>
            <button
              onClick={() => trigger("rebuild")}
              className="border border-zinc-200 bg-zinc-100 px-4 py-2 text-sm font-semibold text-black transition hover:bg-white"
            >
              <FaIcon icon={faHammer} className="mr-2" />
              Rebuild
            </button>
            <button
              onClick={handleDelete}
              disabled={deleting}
              className="border border-zinc-800 bg-black px-4 py-2 text-sm font-medium text-zinc-300 transition hover:border-zinc-600 hover:bg-zinc-950 disabled:opacity-60"
            >
              <FaIcon icon={faTrash} className="mr-2" />
              {deleting ? "Deleting…" : "Delete Package"}
            </button>
          </div>
        </div>
      </section>

      <section className="grid gap-6 xl:grid-cols-[minmax(0,1.1fr)_minmax(320px,0.9fr)]">
        <form
          onSubmit={handleSave}
          className="border border-zinc-800 bg-black p-6"
        >
          <div className="mb-6">
            <h2 className="text-xl font-semibold text-white">Edit Package</h2>
            <p className="mt-2 text-sm text-zinc-400">
              Update the tracked repository, selected spec path, polling
              behavior, and package state from one place.
            </p>
          </div>

          <div className="space-y-5">
            <label className="block">
              <span className="mb-2 block text-sm font-medium text-zinc-300">
                Git repository URL
              </span>
              <input
                type="url"
                value={form.repoUrl}
                onChange={(event) =>
                  setForm((current) => ({
                    ...current,
                    repoUrl: event.target.value,
                  }))
                }
                className="w-full border border-zinc-800 bg-black px-4 py-3 text-sm text-white outline-none transition focus:border-zinc-600"
                required
              />
            </label>

            <div className="border border-zinc-800 bg-black p-4">
              <div className="flex flex-col gap-3 md:flex-row md:items-center md:justify-between">
                <div>
                  <span className="block text-sm font-medium text-zinc-300">
                    Repository spec path
                  </span>
                  <span className="mt-1 block text-xs text-zinc-500">
                    Choose the .spec file from the tracked repository.
                  </span>
                </div>
                <button
                  type="button"
                  onClick={() => setShowSpecPicker(true)}
                  className="border border-zinc-800 bg-black px-4 py-2 text-sm text-zinc-300 transition hover:border-zinc-600 hover:bg-zinc-950"
                >
                  <FaIcon icon={faMagnifyingGlass} className="mr-2" />
                  Browse repository
                </button>
              </div>
              <input
                type="text"
                value={form.specPath}
                onChange={(event) =>
                  setForm((current) => ({
                    ...current,
                    specPath: event.target.value,
                  }))
                }
                placeholder="path/to/package.spec"
                className="mt-4 w-full border border-zinc-800 bg-black px-4 py-3 text-sm text-white outline-none transition focus:border-zinc-600"
                required
              />
            </div>

            <div className="grid gap-4 md:grid-cols-2">
              <label className="block">
                <span className="mb-2 block text-sm font-medium text-zinc-300">
                  Poll interval (seconds)
                </span>
                <input
                  type="number"
                  min="1"
                  step="1"
                  value={form.pollIntervalSeconds}
                  onChange={(event) =>
                    setForm((current) => ({
                      ...current,
                      pollIntervalSeconds: event.target.value,
                    }))
                  }
                  className="w-full border border-zinc-800 bg-black px-4 py-3 text-sm text-white outline-none transition focus:border-zinc-600"
                  required
                />
              </label>

              <label className="block">
                <span className="mb-2 block text-sm font-medium text-zinc-300">
                  Build timeout (seconds)
                </span>
                <input
                  type="number"
                  min="1"
                  step="1"
                  value={form.buildTimeoutSeconds}
                  onChange={(event) =>
                    setForm((current) => ({
                      ...current,
                      buildTimeoutSeconds: event.target.value,
                    }))
                  }
                  className="w-full border border-zinc-800 bg-black px-4 py-3 text-sm text-white outline-none transition focus:border-zinc-600"
                  required
                />
              </label>

              <label className="block md:col-span-2">
                <span className="mb-2 block text-sm font-medium text-zinc-300">
                  History count
                </span>
                <input
                  type="number"
                  min="1"
                  step="1"
                  value={form.packageHistoryCount}
                  onChange={(event) =>
                    setForm((current) => ({
                      ...current,
                      packageHistoryCount: event.target.value,
                    }))
                  }
                  className="w-full border border-zinc-800 bg-black px-4 py-3 text-sm text-white outline-none transition focus:border-zinc-600"
                  required
                />
              </label>

              <div className="border border-zinc-800 bg-black p-4 md:col-span-2">
                <div className="flex flex-col gap-3 md:flex-row md:items-center md:justify-between">
                  <div>
                    <span className="block text-sm font-medium text-zinc-300">
                      Mock chroots
                    </span>
                    <span className="mt-1 block text-xs text-zinc-500">
                      Each selected chroot becomes a separate build job.
                    </span>
                  </div>
                  <button
                    type="button"
                    onClick={() => setShowChrootPicker(true)}
                    className="border border-zinc-800 bg-black px-4 py-2 text-sm text-zinc-300 transition hover:border-zinc-600 hover:bg-zinc-950"
                  >
                    Choose chroots
                  </button>
                </div>
                <div className="mt-4 border border-zinc-800 bg-zinc-950 px-4 py-3 text-sm font-mono text-zinc-200">
                  {form.mockChroots.length > 0
                    ? formatMockChroots(form.mockChroots)
                    : "No chroots selected"}
                </div>
              </div>

              <label className="flex items-center justify-between border border-zinc-800 bg-black px-4 py-3">
                <span>
                  <span className="block text-sm font-medium text-white">
                    Enabled
                  </span>
                  <span className="mt-1 block text-xs text-zinc-400">
                    Allow new builds for this package.
                  </span>
                </span>
                <input
                  type="checkbox"
                  checked={form.enabled}
                  onChange={(event) =>
                    setForm((current) => ({
                      ...current,
                      enabled: event.target.checked,
                    }))
                  }
                  className="h-4 w-4 rounded border-zinc-700 bg-zinc-900"
                />
              </label>

              <label className="flex items-center justify-between border border-zinc-800 bg-black px-4 py-3">
                <span>
                  <span className="block text-sm font-medium text-white">
                    Source Polling
                  </span>
                  <span className="mt-1 block text-xs text-zinc-400">
                    Watch the tracked git repository for new commits.
                  </span>
                </span>
                <input
                  type="checkbox"
                  checked={form.poll}
                  onChange={(event) =>
                    setForm((current) => ({
                      ...current,
                      poll: event.target.checked,
                    }))
                  }
                  className="h-4 w-4 rounded border-zinc-700 bg-zinc-900"
                />
              </label>

              <label className="flex items-center justify-between border border-zinc-800 bg-black px-4 py-3 md:col-span-2">
                <span>
                  <span className="block text-sm font-medium text-white">
                    Publish SRPM
                  </span>
                  <span className="mt-1 block text-xs text-zinc-400">
                    Keep source RPM publication enabled for this package.
                  </span>
                </span>
                <input
                  type="checkbox"
                  checked={form.publish_srpm}
                  onChange={(event) =>
                    setForm((current) => ({
                      ...current,
                      publish_srpm: event.target.checked,
                    }))
                  }
                  className="h-4 w-4 rounded border-zinc-700 bg-zinc-900"
                />
              </label>

              <label className="flex items-center justify-between border border-zinc-800 bg-black px-4 py-3 md:col-span-2">
                <span>
                  <span className="block text-sm font-medium text-white">
                    Network access
                  </span>
                  <span className="mt-1 block text-xs text-zinc-400">
                    Allow mock builds for this package to access the network.
                  </span>
                </span>
                <input
                  type="checkbox"
                  checked={form.network_access}
                  onChange={(event) =>
                    setForm((current) => ({
                      ...current,
                      network_access: event.target.checked,
                    }))
                  }
                  className="h-4 w-4 rounded border-zinc-700 bg-zinc-900"
                />
              </label>
            </div>

            <label className="block">
              <span className="mb-2 block text-sm font-medium text-zinc-300">
                Build environment
              </span>
              <textarea
                value={form.buildEnv}
                onChange={(event) =>
                  setForm((current) => ({
                    ...current,
                    buildEnv: event.target.value,
                  }))
                }
                rows={6}
                placeholder={
                  "KEY=value\nMESON_ARGS=-Dgallium-drivers=swrast\nRUSTFLAGS=-C debuginfo=1"
                }
                className="w-full border border-zinc-800 bg-black px-4 py-3 font-mono text-sm text-white outline-none transition focus:border-zinc-600"
              />
              <span className="mt-2 block text-xs text-zinc-500">
                One `KEY=value` entry per line. Applied to SRPM creation and
                mock rebuild steps.
              </span>
            </label>

            <div className="flex justify-end">
              <button
                type="submit"
                disabled={saving}
                className="border border-zinc-200 bg-zinc-100 px-5 py-2.5 text-sm font-semibold text-black transition hover:bg-white disabled:opacity-70"
              >
                <FaIcon icon={faSave} className="mr-2" />
                {saving ? "Saving…" : "Save Changes"}
              </button>
            </div>
          </div>
        </form>

        <aside className="space-y-4 border border-zinc-800 bg-black p-6">
          <div>
            <h2 className="text-xl font-semibold text-white">State</h2>
            <p className="mt-2 text-sm text-zinc-400">
              Current package and build status.
            </p>
          </div>
          <div>
            <div className="mb-2 text-xs uppercase tracking-[0.18em] text-zinc-500">
              Status
            </div>
            <StatusPill status={pkg.package.enabled ? "enabled" : "disabled"} />
          </div>
          <DetailStat
            label="Version"
            value={`${pkg.package.version}-${pkg.package.release}`}
          />
          <DetailStat
            label="Mock Chroots"
            value={formatMockChroots(pkg.package.mock_chroots)}
          />
          <DetailStat
            label="Repository"
            value={pkg.package.source.repo_url}
            mono
          />
          <DetailStat
            label="Poll Interval"
            value={`${pkg.package.poll_interval_seconds}s`}
          />
          <DetailStat
            label="Build Timeout"
            value={`${pkg.package.build_timeout_seconds}s`}
          />
          <DetailStat
            label="History Count"
            value={pkg.package.package_history_count}
          />
          <DetailStat
            label="Network Access"
            value={pkg.package.network_access ? "Enabled" : "Disabled"}
          />
          <DetailStat
            label="Build Env Vars"
            value={String(pkg.package.build_env.length)}
          />
          <DetailStat
            label="Last Successful Revision"
            value={pkg.state.last_revision || "None yet"}
          />
          <DetailStat
            label="Active Job"
            value={pkg.state.active_job_id || "None"}
          />
          <DetailStat label="Spec Path" value={pkg.package.spec_path} mono />
        </aside>
      </section>

      <section className="border border-zinc-800 bg-black p-6">
        <div className="mb-5 flex flex-col gap-2 md:flex-row md:items-end md:justify-between">
          <div>
            <h2 className="text-xl font-semibold text-white">Build History</h2>
            <p className="mt-2 text-sm text-zinc-400">
              Build activity for this package, including revisions, outcomes,
              and managed repo ownership.
            </p>
          </div>
          <div className="flex items-center gap-3">
            {buildsLoaded ? (
              <div className="border border-zinc-800 bg-black px-4 py-2 text-xs uppercase tracking-[0.2em] text-zinc-400">
                {builds.length} total builds
              </div>
            ) : null}
            <button
              type="button"
              onClick={() => setBuildsOpen((current) => !current)}
              className="border border-zinc-800 bg-black px-4 py-2 text-sm text-zinc-300 transition hover:border-zinc-600 hover:bg-zinc-950"
            >
              {buildsOpen ? "Hide History" : "Open History"}
            </button>
          </div>
        </div>

        {!buildsOpen ? (
          <EmptyState>
            Open build history to load recent package builds.
          </EmptyState>
        ) : buildsLoading && !buildsLoaded ? (
          <LoadingBlock label="Loading build history…" lines={4} />
        ) : latestBuilds.length === 0 ? (
          <EmptyState>No build history yet.</EmptyState>
        ) : (
          <div className="overflow-hidden border border-zinc-800">
            <table className="w-full">
              <thead className="bg-zinc-950 text-left text-xs uppercase tracking-[0.2em] text-zinc-500">
                <tr>
                  <th className="px-4 py-3">Target</th>
                  <th className="px-4 py-3">Revision</th>
                  <th className="px-4 py-3">Status</th>
                  <th className="px-4 py-3">Trigger</th>
                  <th className="px-4 py-3">Created</th>
                  <th className="px-4 py-3">Repo Files</th>
                  <th className="px-4 py-3">Actions</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-white/10 bg-black">
                {latestBuilds.map((entry) => {
                  const publishedFiles = entry.repo_files;
                  const live =
                    entry.build.job.status === "pending" ||
                    entry.build.job.status === "running";
                  return (
                    <tr key={entry.build.job.id} className="hover:bg-zinc-950">
                      <td className="px-4 py-3 text-sm font-mono text-zinc-300">
                        {entry.build.job.mock_chroot}
                      </td>
                      <td className="px-4 py-3">
                        <div className="font-mono text-sm text-zinc-200">
                          {entry.build.job.revision}
                        </div>
                        <div className="mt-1 text-xs text-zinc-500">
                          {entry.build.job.id}
                        </div>
                      </td>
                      <td className="px-4 py-3">
                        <StatusPill status={entry.build.job.status} />
                      </td>
                      <td className="px-4 py-3 text-sm text-zinc-400">
                        {entry.build.job.trigger}
                      </td>
                      <td className="px-4 py-3 text-sm text-zinc-400">
                        {formatDateTime(entry.build.job.created_at)}
                      </td>
                      <td className="px-4 py-3 text-sm text-zinc-300">
                        {publishedFiles.length}
                      </td>
                      <td className="px-4 py-3">
                        <div className="flex flex-wrap gap-3">
                          <a
                            href={`/jobs/view/?id=${encodeURIComponent(entry.build.job.id)}`}
                            className="text-sm font-medium text-zinc-300 transition hover:text-white"
                          >
                            <FaIcon icon={faFolderOpen} className="mr-2" />
                            Open Job
                          </a>
                          <button
                            onClick={() => handleDeleteJob(entry.build.job.id)}
                            disabled={
                              live || deletingJobId === entry.build.job.id
                            }
                            className="text-sm font-medium text-zinc-500 transition hover:text-white disabled:cursor-not-allowed disabled:opacity-50"
                          >
                            <FaIcon icon={faTrash} className="mr-2" />
                            {deletingJobId === entry.build.job.id
                              ? "Deleting…"
                              : "Delete Build"}
                          </button>
                        </div>
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        )}
      </section>

      <section className="border border-zinc-800 bg-black p-6">
        <div className="mb-5 flex flex-col gap-2 md:flex-row md:items-end md:justify-between">
          <div>
            <h2 className="text-xl font-semibold text-white">
              Repository Files
            </h2>
            <p className="mt-2 text-sm text-zinc-400">
              Build-owned files currently present in the repo namespace for this
              package.
            </p>
          </div>
          <div className="flex items-center gap-3">
            {repoFilesLoaded ? (
              <div className="border border-zinc-800 bg-black px-4 py-2 text-xs uppercase tracking-[0.2em] text-zinc-400">
                {repoFiles.length} tracked files
              </div>
            ) : null}
            <button
              type="button"
              onClick={() => setRepoFilesOpen((current) => !current)}
              className="border border-zinc-800 bg-black px-4 py-2 text-sm text-zinc-300 transition hover:border-zinc-600 hover:bg-zinc-950"
            >
              {repoFilesOpen ? "Hide Files" : "Open Files"}
            </button>
          </div>
        </div>

        {!repoFilesOpen ? (
          <EmptyState>
            Open repository files to load the repo-owned outputs for this
            package.
          </EmptyState>
        ) : repoFilesLoading && !repoFilesLoaded ? (
          <LoadingBlock label="Loading repository files…" lines={4} />
        ) : repoFiles.length === 0 ? (
          <EmptyState>
            No repo files are currently tracked for this package.
          </EmptyState>
        ) : (
          <div className="overflow-x-auto border border-zinc-800">
            <table className="min-w-[980px] w-full">
              <thead className="bg-zinc-950 text-left text-xs uppercase tracking-[0.2em] text-zinc-500">
                <tr>
                  <th className="px-4 py-3">Repo Path</th>
                  <th className="px-4 py-3">Build</th>
                  <th className="px-4 py-3">Kind</th>
                  <th className="px-4 py-3">Size</th>
                  <th className="px-4 py-3">Published</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-white/10 bg-black">
                {repoFiles.map((file) => (
                  <tr
                    key={`${file.job_id}:${file.repo_path}`}
                    className="hover:bg-zinc-950"
                  >
                    <td className="px-4 py-3 font-mono text-sm text-zinc-200">
                      {file.repo_path}
                    </td>
                    <td className="px-4 py-3">
                      <a
                        href={`/jobs/view/?id=${encodeURIComponent(file.job_id)}`}
                        className="font-mono text-sm text-zinc-300 transition hover:text-white"
                      >
                        <FaIcon icon={faBoxesStacked} className="mr-2" />
                        {file.job_id}
                      </a>
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
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </section>

      {showSpecPicker && (
        <SelectionDialog
          title="Choose spec file"
          subtitle="Browse the tracked repository and select the .spec file to build."
          onClose={() => setShowSpecPicker(false)}
        >
          <div className="space-y-4">
            <button
              type="button"
              onClick={handleBrowse}
              disabled={browsing}
              className="border border-zinc-800 bg-black px-4 py-2 text-sm text-zinc-300 transition hover:border-zinc-600 hover:bg-zinc-950 disabled:opacity-60"
            >
              <FaIcon icon={faMagnifyingGlass} className="mr-2" />
              {browsing ? "Browsing…" : "Load repository files"}
            </button>
            {browseError ? (
              <div className="border border-zinc-800 bg-black px-4 py-3 text-sm text-zinc-200">
                {browseError}
              </div>
            ) : null}
            <div className="max-h-[50vh] overflow-auto border border-zinc-800 bg-black">
              {selectableFiles.length > 0 ? (
                selectableFiles.map((file) => (
                  <button
                    key={file}
                    type="button"
                    onClick={() => {
                      setForm((current) => ({ ...current, specPath: file }));
                      setShowSpecPicker(false);
                    }}
                    className={`block w-full border-b border-zinc-800 px-4 py-2 text-left font-mono text-sm transition last:border-b-0 ${
                      form.specPath === file
                        ? "bg-zinc-950 text-white"
                        : "bg-black text-zinc-300 hover:bg-zinc-950"
                    }`}
                  >
                    {file}
                  </button>
                ))
              ) : (
                <div className="px-4 py-3 text-sm text-zinc-400">
                  No spec files loaded yet.
                </div>
              )}
            </div>
          </div>
        </SelectionDialog>
      )}

      {showChrootPicker && (
        <SelectionDialog
          title="Choose mock chroots"
          subtitle="Select one or more build targets."
          onClose={() => setShowChrootPicker(false)}
        >
          <div className="max-h-[50vh] overflow-y-auto border border-zinc-800 bg-black">
            <div className="divide-y divide-white/8">
              {availableChroots.map((chroot) => (
                <label
                  key={chroot}
                  className="flex items-center justify-between gap-4 px-4 py-3 text-sm text-zinc-200"
                >
                  <span className="font-mono">{chroot}</span>
                  <input
                    type="checkbox"
                    checked={form.mockChroots.includes(chroot)}
                    onChange={(event) =>
                      toggleChroot(chroot, event.target.checked)
                    }
                    className="h-4 w-4 rounded border-zinc-700 bg-zinc-900"
                  />
                </label>
              ))}
            </div>
          </div>
        </SelectionDialog>
      )}
    </div>
  );
}

function SelectionDialog({
  title,
  subtitle,
  onClose,
  children,
}: {
  title: string;
  subtitle: string;
  onClose: () => void;
  children: React.ReactNode;
}) {
  return (
    <div className="fixed inset-0 z-[60] overflow-y-auto bg-black/80 px-4 py-8">
      <div className="mx-auto w-full max-w-3xl border border-zinc-800 bg-black">
        <div className="flex items-start justify-between gap-4 border-b border-zinc-800 px-6 py-5">
          <div>
            <h3 className="text-xl font-semibold text-white">{title}</h3>
            <p className="mt-2 text-sm text-zinc-400">{subtitle}</p>
          </div>
          <button
            type="button"
            onClick={onClose}
            className="border border-zinc-800 bg-black px-3 py-2 text-sm text-zinc-300 transition hover:border-zinc-600 hover:bg-zinc-950"
          >
            Close
          </button>
        </div>
        <div className="px-6 py-6">{children}</div>
      </div>
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
