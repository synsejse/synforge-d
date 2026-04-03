import { useEffect, useMemo, useState, type FormEvent } from "react";
import api from "../lib/api";
import FaIcon from "./FaIcon";
import ActionButton from "./ActionButton";
import EmptyState from "./EmptyState";
import PageHeader from "./PageHeader";
import StatusPill from "./StatusPill";
import type {
  BuildEnvVar,
  CreatePackageRequest,
  PackageResponse,
  SpecSource,
} from "../lib/types";
import {
  faFolderOpen,
  faPlus,
  faRotate,
  faHammer,
  faTrash,
  faMagnifyingGlass,
} from "@fortawesome/free-solid-svg-icons";

export default function PackageList() {
  const [packages, setPackages] = useState<PackageResponse[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showAddModal, setShowAddModal] = useState(false);

  async function load() {
    try {
      setLoading(true);
      const res = await api.listPackages();
      setPackages(res.packages);
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to load packages");
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    load();
  }, []);

  async function handleDelete(name: string) {
    if (!confirm(`Delete package "${name}"?`)) {
      return;
    }
    try {
      await api.deletePackage(name);
      await load();
    } catch (e) {
      alert(e instanceof Error ? e.message : "Failed to delete package");
    }
  }

  async function trigger(name: string, action: "refresh" | "rebuild") {
    try {
      if (action === "refresh") {
        await api.refreshPackage(name);
      } else {
        await api.rebuildPackage(name);
      }
      await load();
    } catch (e) {
      alert(e instanceof Error ? e.message : `Failed to ${action} package`);
    }
  }

  if (loading) {
    return <div className="text-zinc-400">Loading packages…</div>;
  }

  if (error) {
    return (
      <div className="border border-zinc-800 bg-black p-4 text-zinc-200">
        Error: {error}
      </div>
    );
  }

  return (
    <div className="space-y-8">
      <PageHeader
        eyebrow="Package Registry"
        title="Packages"
        description="Sources, targets, and builds."
        actions={[
          {
            onClick: () => setShowAddModal(true),
            label: "Add Package",
            icon: faPlus,
            variant: "primary",
          },
        ]}
      />

      {packages.length === 0 ? (
        <EmptyState>
          No packages configured yet. Add a spec source to start building.
        </EmptyState>
      ) : (
        <div className="overflow-x-auto border border-zinc-800 bg-black">
          <table className="min-w-[1220px] w-full">
            <thead className="bg-zinc-950 text-left text-xs uppercase tracking-[0.2em] text-zinc-500">
              <tr>
                <th className="px-4 py-3">Package</th>
                <th className="px-4 py-3">Version</th>
                <th className="px-4 py-3">Target</th>
                <th className="px-4 py-3">Repository</th>
                <th className="px-4 py-3">Spec file</th>
                <th className="px-4 py-3">Last revision</th>
                <th className="px-4 py-3">Status</th>
                <th className="px-4 py-3">Actions</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-white/8">
              {packages.map((entry) => {
                const active = Boolean(entry.state.active_job_id);
                const status = active
                  ? "running"
                  : entry.package.enabled
                    ? "enabled"
                    : "disabled";
                return (
                  <tr key={entry.package.name} className="hover:bg-zinc-950">
                    <td className="px-4 py-3">
                      <div className="min-w-[180px] space-y-1">
                        <a
                          href={`/packages/view/?name=${encodeURIComponent(entry.package.name)}`}
                          className="font-medium text-white transition hover:text-zinc-300"
                        >
                          {entry.package.name}
                        </a>
                        <div className="max-w-[260px] text-xs text-zinc-500">
                          {entry.package.description || "No description"}
                        </div>
                      </div>
                    </td>
                    <td className="px-4 py-3 text-sm text-zinc-300">
                      {entry.package.version}-{entry.package.release}
                    </td>
                    <td className="px-4 py-3 text-sm text-zinc-400">
                      {formatMockChroots(entry.package.mock_chroots)}
                    </td>
                    <td className="px-4 py-3">
                      <div className="max-w-[280px] break-all font-mono text-sm text-zinc-300">
                        {entry.package.source.repo_url}
                      </div>
                    </td>
                    <td className="px-4 py-3">
                      <div className="max-w-[220px] break-all font-mono text-sm text-zinc-300">
                        {entry.package.source.spec_path}
                      </div>
                    </td>
                    <td className="px-4 py-3">
                      <div className="max-w-[220px] break-all font-mono text-sm text-zinc-400">
                        {entry.state.last_revision || "None yet"}
                      </div>
                    </td>
                    <td className="px-4 py-3">
                      <StatusPill status={status} />
                    </td>
                    <td className="px-4 py-3">
                      <div className="flex flex-wrap gap-2">
                        <ActionButton
                          href={`/packages/view/?name=${encodeURIComponent(entry.package.name)}`}
                          icon={faFolderOpen}
                        >
                          Open
                        </ActionButton>
                        <ActionButton
                          onClick={() => trigger(entry.package.name, "refresh")}
                          icon={faRotate}
                        >
                          Refresh
                        </ActionButton>
                        <ActionButton
                          onClick={() => trigger(entry.package.name, "rebuild")}
                          icon={faHammer}
                        >
                          Rebuild
                        </ActionButton>
                        <ActionButton
                          onClick={() => handleDelete(entry.package.name)}
                          icon={faTrash}
                          className="text-zinc-300"
                        >
                          Delete
                        </ActionButton>
                      </div>
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      )}

      {showAddModal && (
        <AddPackageModal
          onClose={() => setShowAddModal(false)}
          onSuccess={() => {
            setShowAddModal(false);
            load();
          }}
        />
      )}
    </div>
  );
}

function formatMockChroots(chroots: string[]) {
  return chroots.join(", ");
}

function encodeBuildEnv(entries: BuildEnvVar[]): string {
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

function AddPackageModal({
  onClose,
  onSuccess,
}: {
  onClose: () => void;
  onSuccess: () => void;
}) {
  const [name, setName] = useState("");
  const [repoUrl, setRepoUrl] = useState("");
  const [specPath, setSpecPath] = useState("");
  const [poll, setPoll] = useState(true);
  const [networkAccess, setNetworkAccess] = useState(false);
  const [mockChroots, setMockChroots] = useState<string[]>([
    "fedora-44-x86_64",
  ]);
  const [pollIntervalSeconds, setPollIntervalSeconds] = useState("900");
  const [buildTimeoutSeconds, setBuildTimeoutSeconds] = useState("7200");
  const [packageHistoryCount, setPackageHistoryCount] = useState("3");
  const [buildEnv, setBuildEnv] = useState(encodeBuildEnv([]));
  const [browsing, setBrowsing] = useState(false);
  const [browseError, setBrowseError] = useState<string | null>(null);
  const [browseFiles, setBrowseFiles] = useState<string[]>([]);
  const [availableChroots, setAvailableChroots] = useState<string[]>([]);
  const [chrootsLoading, setChrootsLoading] = useState(true);
  const [showSpecPicker, setShowSpecPicker] = useState(false);
  const [showChrootPicker, setShowChrootPicker] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const selectableFiles = useMemo(
    () => browseFiles.filter((file) => file.endsWith(".spec")),
    [browseFiles],
  );

  useEffect(() => {
    async function loadChroots() {
      try {
        const response = await api.listMockChroots();
        setAvailableChroots(response.chroots);
        setMockChroots((current) => {
          if (current.some((value) => response.chroots.includes(value))) {
            return current.filter((value) => response.chroots.includes(value));
          }
          if (response.chroots.includes("fedora-44-x86_64")) {
            return ["fedora-44-x86_64"];
          }
          return response.chroots.length > 0 ? [response.chroots[0]] : [];
        });
      } catch (e) {
        setError(
          e instanceof Error ? e.message : "Failed to load mock chroots",
        );
      } finally {
        setChrootsLoading(false);
      }
    }

    loadChroots();
  }, []);

  async function handleBrowse() {
    if (!repoUrl.trim()) {
      setBrowseError("Repository URL is required before browsing.");
      return;
    }
    setBrowsing(true);
    setBrowseError(null);
    try {
      const response = await api.browseRepository({ repo_url: repoUrl.trim() });
      setBrowseFiles(response.files);
      if (!specPath && response.spec_files.length > 0) {
        setSpecPath(response.spec_files[0]);
      }
    } catch (e) {
      setBrowseError(
        e instanceof Error ? e.message : "Failed to browse repository",
      );
    } finally {
      setBrowsing(false);
    }
  }

  async function handleSubmit(event: FormEvent) {
    event.preventDefault();
    setSubmitting(true);
    setError(null);

    const source: SpecSource = {
      repo_url: repoUrl.trim(),
      spec_path: specPath.trim(),
      poll,
    };

    const request: CreatePackageRequest = {
      name: name.trim(),
      source,
      network_access: networkAccess,
      mock_chroots: mockChroots,
      poll_interval_seconds: Number(pollIntervalSeconds),
      build_timeout_seconds: Number(buildTimeoutSeconds),
      package_history_count: Number(packageHistoryCount),
      build_env: parseBuildEnv(buildEnv),
    };

    try {
      await api.createPackage(request);
      onSuccess();
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to create package");
    } finally {
      setSubmitting(false);
    }
  }

  function toggleChroot(chroot: string, checked: boolean) {
    setMockChroots((current) => {
      const next = checked
        ? Array.from(new Set([...current, chroot]))
        : current.filter((value) => value !== chroot);
      return next;
    });
  }

  return (
    <div className="fixed inset-0 z-50 overflow-y-auto bg-black/70 px-4 py-6">
      <div className="mx-auto w-full max-w-3xl border border-zinc-800 bg-black">
        <div className="border-b border-zinc-800 px-6 py-5">
          <p className="text-xs uppercase tracking-[0.28em] text-zinc-500">
            Package
          </p>
          <h2 className="mt-2 text-2xl font-semibold text-white">
            Add package
          </h2>
        </div>

        <form
          onSubmit={handleSubmit}
          className="max-h-[calc(100vh-8rem)] space-y-5 overflow-y-auto px-6 py-6"
        >
          <label className="block">
            <span className="mb-2 block text-sm font-medium text-zinc-300">
              Package name
            </span>
            <input
              type="text"
              value={name}
              onChange={(event) => setName(event.target.value)}
              placeholder="mesa"
              required
              className="w-full border border-zinc-800 bg-black px-4 py-3 text-sm text-white outline-none transition focus:border-zinc-600"
            />
          </label>

          <label className="block">
            <span className="mb-2 block text-sm font-medium text-zinc-300">
              Git repository URL
            </span>
            <input
              type="url"
              value={repoUrl}
              onChange={(event) => setRepoUrl(event.target.value)}
              placeholder="https://github.com/example/repo.git"
              required
              className="w-full border border-zinc-800 bg-black px-4 py-3 text-sm text-white outline-none transition focus:border-zinc-600"
            />
          </label>

          <label className="flex items-center justify-between border border-zinc-800 bg-black px-4 py-3">
            <span>
              <span className="block text-sm font-medium text-white">
                Enable polling
              </span>
              <span className="mt-1 block text-xs text-zinc-400">
                Automatically watch the source for updates.
              </span>
            </span>
            <input
              type="checkbox"
              checked={poll}
              onChange={(event) => setPoll(event.target.checked)}
              className="h-4 w-4 rounded border-zinc-700 bg-zinc-900"
            />
          </label>

          <label className="flex items-center justify-between border border-zinc-800 bg-black px-4 py-3">
            <span>
              <span className="block text-sm font-medium text-white">
                Network access
              </span>
              <span className="mt-1 block text-xs text-zinc-400">
                Allow mock builds to access the network for packages that cannot
                build fully offline.
              </span>
            </span>
            <input
              type="checkbox"
              checked={networkAccess}
              onChange={(event) => setNetworkAccess(event.target.checked)}
              className="h-4 w-4 rounded border-zinc-700 bg-zinc-900"
            />
          </label>

          <div className="grid gap-4 lg:grid-cols-2">
            <div className="border border-zinc-800 bg-black p-4 lg:col-span-2">
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
                {mockChroots.length > 0
                  ? formatMockChroots(mockChroots)
                  : "No chroots selected"}
              </div>
            </div>

            <label className="block">
              <span className="mb-2 block text-sm font-medium text-zinc-300">
                Poll interval (seconds)
              </span>
              <input
                type="number"
                min="1"
                step="1"
                value={pollIntervalSeconds}
                onChange={(event) => setPollIntervalSeconds(event.target.value)}
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
                value={buildTimeoutSeconds}
                onChange={(event) => setBuildTimeoutSeconds(event.target.value)}
                className="w-full border border-zinc-800 bg-black px-4 py-3 text-sm text-white outline-none transition focus:border-zinc-600"
                required
              />
            </label>

            <label className="block lg:col-span-2">
              <span className="mb-2 block text-sm font-medium text-zinc-300">
                History count
              </span>
              <input
                type="number"
                min="1"
                step="1"
                value={packageHistoryCount}
                onChange={(event) => setPackageHistoryCount(event.target.value)}
                className="w-full border border-zinc-800 bg-black px-4 py-3 text-sm text-white outline-none transition focus:border-zinc-600"
                required
              />
            </label>
          </div>

          <label className="block">
            <span className="mb-2 block text-sm font-medium text-zinc-300">
              Build environment
            </span>
            <textarea
              value={buildEnv}
              onChange={(event) => setBuildEnv(event.target.value)}
              rows={6}
              placeholder={
                "KEY=value\nMESON_ARGS=-Dgallium-drivers=swrast\nRUSTFLAGS=-C debuginfo=1"
              }
              className="w-full border border-zinc-800 bg-black px-4 py-3 font-mono text-sm text-white outline-none transition focus:border-zinc-600"
            />
            <span className="mt-2 block text-xs text-zinc-500">
              One `KEY=value` entry per line. Applied to SRPM creation and mock
              rebuild steps.
            </span>
          </label>

          <div className="border border-zinc-800 bg-black p-4">
            <div className="flex flex-col gap-3 md:flex-row md:items-center md:justify-between">
              <div>
                <div className="text-sm font-medium text-white">Spec file</div>
                <div className="mt-1 text-xs text-zinc-400">
                  Browse the repository and select the `.spec` file to build.
                </div>
              </div>
              <button
                type="button"
                onClick={() => setShowSpecPicker(true)}
                disabled={browsing}
                className="border border-zinc-800 bg-black px-4 py-2 text-sm text-zinc-300 transition hover:border-zinc-600 hover:bg-zinc-950 disabled:opacity-60"
              >
                <FaIcon icon={faMagnifyingGlass} className="mr-2" />
                Browse repository
              </button>
            </div>

            <input
              type="text"
              value={specPath}
              onChange={(event) => setSpecPath(event.target.value)}
              placeholder="path/to/package.spec"
              required
              className="mt-4 w-full border border-zinc-800 bg-black px-4 py-3 text-sm text-white outline-none transition focus:border-zinc-600"
            />
          </div>

          {error && (
            <div className="border border-zinc-800 bg-black px-4 py-3 text-sm text-zinc-200">
              {error}
            </div>
          )}

          <div className="flex justify-end gap-3 pt-2">
            <button
              type="button"
              onClick={onClose}
              className="border border-zinc-800 bg-black px-4 py-2 text-sm text-zinc-300 transition hover:border-zinc-600 hover:bg-zinc-950"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={submitting}
              className="border border-zinc-200 bg-zinc-100 px-4 py-2 text-sm font-semibold text-black transition hover:bg-white disabled:opacity-70"
            >
              <FaIcon icon={faPlus} className="mr-2" />
              {submitting ? "Adding…" : "Add Package"}
            </button>
          </div>
        </form>
      </div>

      {showSpecPicker && (
        <SelectionDialog
          title="Choose spec file"
          subtitle="Browse the repository and select the .spec file to build."
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
                      setSpecPath(file);
                      setShowSpecPicker(false);
                    }}
                    className={`block w-full border-b border-zinc-800 px-4 py-2 text-left font-mono text-sm transition last:border-b-0 ${
                      specPath === file
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
            {chrootsLoading ? (
              <div className="px-4 py-3 text-sm text-zinc-400">
                Loading available chroots…
              </div>
            ) : availableChroots.length === 0 ? (
              <div className="px-4 py-3 text-sm text-zinc-400">
                No mock chroots available.
              </div>
            ) : (
              <div className="divide-y divide-white/8">
                {availableChroots.map((chroot) => (
                  <label
                    key={chroot}
                    className="flex items-center justify-between gap-4 px-4 py-3 text-sm text-zinc-200"
                  >
                    <span className="font-mono">{chroot}</span>
                    <input
                      type="checkbox"
                      checked={mockChroots.includes(chroot)}
                      onChange={(event) =>
                        toggleChroot(chroot, event.target.checked)
                      }
                      className="h-4 w-4 rounded border-zinc-700 bg-zinc-900"
                    />
                  </label>
                ))}
              </div>
            )}
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
