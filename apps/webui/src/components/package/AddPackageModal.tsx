import {
  useEffect,
  useId,
  useMemo,
  useRef,
  useState,
  type SyntheticEvent,
} from "react";
import {
  faMagnifyingGlass,
  faPlus,
} from "@fortawesome/free-solid-svg-icons";
import api from "../../lib/api";
import { formatMockChroots } from "../../lib/utils";
import type {
  BrowseRepositoryProgressView,
  BuildEnvVar,
  CreatePackageRequest,
  ServerHardwareResponse,
  SpecSource,
} from "../../lib/types";
import FaIcon from "../ui/FaIcon";
import SelectionDialog from "../common/SelectionDialog";

interface AddPackageModalProps {
  onClose: () => void;
  onSuccess: () => void;
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

function parseOptionalCpuLimit(
  value: string,
  maxCpuCores: number | null,
): number | undefined {
  const trimmed = value.trim();
  if (!trimmed) {
    return undefined;
  }
  const parsed = Number(trimmed);
  if (!Number.isFinite(parsed) || parsed <= 0) {
    return undefined;
  }
  const millicores = Math.floor(parsed * 1000);
  if (!maxCpuCores || maxCpuCores <= 0) {
    return millicores;
  }
  return Math.min(millicores, Math.floor(maxCpuCores * 1000));
}

function stateLabel(state: BrowseRepositoryProgressView["state"]): string {
  switch (state) {
    case "completed":
      return "Completed";
    case "failed":
      return "Failed";
    default:
      return "Cloning";
  }
}

function MockTargetCheckIndicator({ label }: { label: string }) {
  return (
    <div className="space-y-2">
      <div className="flex items-center justify-between gap-3">
        <span className="font-mono text-xs uppercase tracking-[0.15em] text-zinc-300">
          {label}
        </span>
        <span className="flex items-center gap-1" aria-hidden="true">
          <span className="h-1.5 w-1.5 rounded-full bg-[var(--theme-accent-lime)] animate-pulse" />
          <span className="h-1.5 w-1.5 rounded-full bg-[var(--theme-accent-lime)] animate-pulse [animation-delay:150ms]" />
          <span className="h-1.5 w-1.5 rounded-full bg-[var(--theme-accent-lime)] animate-pulse [animation-delay:300ms]" />
        </span>
      </div>
      <div className="h-2 w-full overflow-hidden border border-zinc-700 bg-zinc-900">
        <div className="h-full w-full animate-pulse bg-[var(--theme-accent-lime)]/65" />
      </div>
    </div>
  );
}

export default function AddPackageModal({
  onClose,
  onSuccess,
}: AddPackageModalProps) {
  const [name, setName] = useState("");
  const [repoUrl, setRepoUrl] = useState("");
  const [specPath, setSpecPath] = useState("");
  const [enabled, setEnabled] = useState(true);
  const [poll, setPoll] = useState(true);
  const [publishSrpm, setPublishSrpm] = useState(true);
  const [publishDebuginfo, setPublishDebuginfo] = useState(true);
  const [networkAccess, setNetworkAccess] = useState(false);
  const [mockChroots, setMockChroots] = useState<string[]>(["fedora-44-x86_64"]);
  const [pollIntervalSeconds, setPollIntervalSeconds] = useState("900");
  const [buildTimeoutSeconds, setBuildTimeoutSeconds] = useState("7200");
  const [packageHistoryCount, setPackageHistoryCount] = useState("3");
  const [cpuLimitCores, setCpuLimitCores] = useState("");
  const [cpuLimitEnabled, setCpuLimitEnabled] = useState(false);
  const [memoryLimitEnabled, setMemoryLimitEnabled] = useState(false);
  const [memoryLimitMb, setMemoryLimitMb] = useState("1024");
  const [ccacheEnabled, setCcacheEnabled] = useState(false);
  const [buildEnv, setBuildEnv] = useState(encodeBuildEnv([]));
  const [browsing, setBrowsing] = useState(false);
  const [browseError, setBrowseError] = useState<string | null>(null);
  const [browseFiles, setBrowseFiles] = useState<string[]>([]);
  const [serverHardware, setServerHardware] =
    useState<ServerHardwareResponse | null>(null);
  const [browseProgress, setBrowseProgress] =
    useState<BrowseRepositoryProgressView | null>(null);
  const [browseProgressIssue, setBrowseProgressIssue] = useState<string | null>(
    null,
  );
  const [availableChroots, setAvailableChroots] = useState<string[]>([]);
  const [chrootsLoading, setChrootsLoading] = useState(true);
  const [showSpecPicker, setShowSpecPicker] = useState(false);
  const [showChrootPicker, setShowChrootPicker] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const titleId = useId();
  const dialogRef = useRef<HTMLDivElement | null>(null);

  const selectableFiles = useMemo(
    () => browseFiles.filter((file) => file.endsWith(".spec")),
    [browseFiles],
  );
  const activeBrowseProgress = useMemo(() => {
    const trimmedRepoUrl = repoUrl.trim();
    if (!browseProgress || !trimmedRepoUrl) {
      return null;
    }
    if (browseProgress.repo_url !== trimmedRepoUrl) {
      return null;
    }
    return browseProgress;
  }, [browseProgress, repoUrl]);
  const browseProgressPercent = activeBrowseProgress?.progress_percent ?? (browsing ? 2 : 0);
  const browseProgressState = activeBrowseProgress?.state ?? "running";
  const browseProgressMessage =
    activeBrowseProgress?.message ?? "Preparing repository clone…";
  const maxCpuCores = serverHardware?.cpu_cores ?? null;
  const CPU_MIN_CORES = 1;
  const CPU_STEP_CORES = 1;
  const cpuSliderMax = Math.max(CPU_MIN_CORES, maxCpuCores ?? 64);
  const CPU_DEFAULT_CORES = Math.min(4, cpuSliderMax);
  const cpuSliderValue = Math.min(
    cpuSliderMax,
    Math.max(CPU_MIN_CORES, Math.floor(Number(cpuLimitCores) || CPU_DEFAULT_CORES)),
  );
  const MEMORY_MIN_MB = 256;
  const MEMORY_STEP_MB = 256;
  const memorySliderMax = Math.max(
    MEMORY_MIN_MB,
    serverHardware?.total_memory_mb
      ? Math.floor(serverHardware.total_memory_mb / MEMORY_STEP_MB) * MEMORY_STEP_MB
      : 32768,
  );
  const MEMORY_DEFAULT_MB = Math.min(1024, memorySliderMax);
  const memorySliderValue = Math.min(
    memorySliderMax,
    Math.max(MEMORY_MIN_MB, Number(memoryLimitMb) || MEMORY_DEFAULT_MB),
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
        setError(e instanceof Error ? e.message : "Failed to load mock chroots");
      } finally {
        setChrootsLoading(false);
      }
    }

    loadChroots();
  }, []);

  useEffect(() => {
    api
      .getServerHardware()
      .then((response) => setServerHardware(response))
      .catch(() => undefined);
  }, []);

  useEffect(() => {
    const firstFocusable = dialogRef.current?.querySelector<HTMLElement>(
      'input, select, textarea, button, [href], [tabindex]:not([tabindex="-1"])',
    );
    firstFocusable?.focus();
  }, []);

  useEffect(() => {
    if (!browsing) {
      return;
    }
    const currentRepoUrl = repoUrl.trim();
    if (!currentRepoUrl) {
      return;
    }

    let cancelled = false;

    const poll = async () => {
      try {
        const response = await api.getBrowseRepositoryProgress();
        if (cancelled) {
          return;
        }
        if (response.operation && response.operation.repo_url === currentRepoUrl) {
          setBrowseProgress(response.operation);
          setBrowseProgressIssue(null);
        }
      } catch (e) {
        if (!cancelled) {
          setBrowseProgressIssue(
            "Clone is running, but live progress updates are temporarily unavailable.",
          );
          console.warn("Failed to poll repository browse progress", e);
        }
      }
    };

    void poll();
    const intervalId = window.setInterval(() => {
      void poll();
    }, 700);

    return () => {
      cancelled = true;
      window.clearInterval(intervalId);
    };
  }, [browsing, repoUrl]);

  async function handleBrowse() {
    const trimmedRepoUrl = repoUrl.trim();
    if (!trimmedRepoUrl) {
      setBrowseError("Repository URL is required before browsing.");
      return;
    }
    setBrowsing(true);
    setBrowseError(null);
    setBrowseProgressIssue(null);
    setBrowseProgress(null);
    try {
      const response = await api.browseRepository({ repo_url: trimmedRepoUrl });
      setBrowseFiles(response.files);
      if (!specPath && response.spec_files.length > 0) {
        setSpecPath(response.spec_files[0]);
      }
    } catch (e) {
      setBrowseError(
        e instanceof Error ? e.message : "Failed to browse repository",
      );
    } finally {
      try {
        const progressResponse = await api.getBrowseRepositoryProgress();
        if (
          progressResponse.operation &&
          progressResponse.operation.repo_url === trimmedRepoUrl
        ) {
          setBrowseProgress(progressResponse.operation);
        }
      } catch (e) {
        console.warn("Failed to load final browse progress", e);
      }
      setBrowsing(false);
    }
  }

  async function handleSubmit(event: SyntheticEvent) {
    event.preventDefault();
    setSubmitting(true);
    setError(null);

    const source: SpecSource = {
      repo_url: repoUrl.trim(),
      spec_file: specPath.trim(),
      poll,
    };

    const request: CreatePackageRequest = {
      name: name.trim(),
      source,
      enabled,
      publish_srpm: publishSrpm,
      publish_debuginfo: publishDebuginfo,
      network_access: networkAccess,
      mock_chroots: mockChroots,
      poll_interval_seconds: Number(pollIntervalSeconds),
      build_timeout_seconds: Number(buildTimeoutSeconds),
      package_history_count: Number(packageHistoryCount),
      cpu_limit_millicores: cpuLimitEnabled
        ? parseOptionalCpuLimit(cpuLimitCores, maxCpuCores)
        : undefined,
      memory_limit_mb: memoryLimitEnabled ? memorySliderValue : undefined,
      ccache_enabled: ccacheEnabled,
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
    <div
      className="fixed inset-0 z-50 flex items-center justify-center overflow-hidden overscroll-none bg-black/70 px-4 py-6"
      onMouseDown={(event) => {
        if (event.target === event.currentTarget) {
          onClose();
        }
      }}
    >
      <div
        ref={dialogRef}
        role="dialog"
        aria-modal="true"
        aria-labelledby={titleId}
        className="flex max-h-[calc(100dvh-3rem)] w-full max-w-3xl flex-col border-4 border-white bg-black shadow-[6px_6px_0_rgba(255,255,255,0.25)]"
      >
        <div className="border-b-2 border-zinc-800 px-6 py-5">
          <p className="font-mono text-xs font-bold uppercase tracking-[0.28em] text-[var(--theme-accent-lime)]">
            Package
          </p>
          <h2 id={titleId} className="mt-2 font-mono text-2xl font-bold uppercase text-white">
            Add package
          </h2>
        </div>

        <form
          onSubmit={handleSubmit}
          className="min-h-0 flex-1 space-y-5 overflow-y-auto overscroll-contain px-6 py-6"
        >
          <label className="block">
            <span className="mb-2 block font-mono text-xs font-bold uppercase tracking-[0.15em] text-zinc-400">
              Package name
            </span>
            <input
              type="text"
              value={name}
              onChange={(event) => setName(event.target.value)}
              placeholder="mesa"
              required
              className="w-full border-2 border-zinc-700 bg-zinc-950 px-4 py-3 font-mono text-sm text-white placeholder:text-zinc-600 outline-none transition duration-100 ease-linear focus:border-[var(--theme-accent-lime)]"
            />
          </label>

          <label className="block">
            <span className="mb-2 block font-mono text-xs font-bold uppercase tracking-[0.15em] text-zinc-400">
              Git repository URL
            </span>
            <input
              type="url"
              value={repoUrl}
              onChange={(event) => setRepoUrl(event.target.value)}
              placeholder="https://github.com/example/repo.git"
              required
              className="w-full border-2 border-zinc-700 bg-zinc-950 px-4 py-3 font-mono text-sm text-white placeholder:text-zinc-600 outline-none transition duration-100 ease-linear focus:border-[var(--theme-accent-lime)]"
            />
          </label>

          <div className="grid gap-4 md:grid-cols-2">
            <label className="flex items-center justify-between border-2 border-zinc-700 bg-zinc-950 px-4 py-3">
              <span>
                <span className="block font-mono text-xs font-bold uppercase tracking-[0.1em] text-white">
                  Shared ccache
                </span>
                <span className="mt-1 block text-xs text-zinc-500">
                  Reuse compiler cache per package and mock chroot.
                </span>
              </span>
              <input
                type="checkbox"
                checked={ccacheEnabled}
                onChange={(event) => setCcacheEnabled(event.target.checked)}
              />
            </label>

            <label className="flex items-center justify-between border-2 border-zinc-700 bg-zinc-950 px-4 py-3">
              <span>
                <span className="block font-mono text-xs font-bold uppercase tracking-[0.1em] text-white">
                  Enabled
                </span>
                <span className="mt-1 block text-xs text-zinc-500">
                  Allow new builds for this package.
                </span>
              </span>
              <input
                type="checkbox"
                checked={enabled}
                onChange={(event) => setEnabled(event.target.checked)}
              />
            </label>

            <label className="flex items-center justify-between border-2 border-zinc-700 bg-zinc-950 px-4 py-3">
              <span>
                <span className="block font-mono text-xs font-bold uppercase tracking-[0.1em] text-white">
                  Enable polling
                </span>
                <span className="mt-1 block text-xs text-zinc-500">
                  Automatically watch the source for updates.
                </span>
              </span>
              <input
                type="checkbox"
                checked={poll}
                onChange={(event) => setPoll(event.target.checked)}
              />
            </label>

            <label className="flex items-center justify-between border-2 border-zinc-700 bg-zinc-950 px-4 py-3">
              <span>
                <span className="block font-mono text-xs font-bold uppercase tracking-[0.1em] text-white">
                  Publish SRPM
                </span>
                <span className="mt-1 block text-xs text-zinc-500">
                  Keep source RPM publication enabled for this package.
                </span>
              </span>
              <input
                type="checkbox"
                checked={publishSrpm}
                onChange={(event) => setPublishSrpm(event.target.checked)}
              />
            </label>

            <label className="flex items-center justify-between border-2 border-zinc-700 bg-zinc-950 px-4 py-3">
              <span>
                <span className="block font-mono text-xs font-bold uppercase tracking-[0.1em] text-white">
                  Publish debug packages
                </span>
                <span className="mt-1 block text-xs text-zinc-500">
                  Include debuginfo and debugsource RPMs in repository.
                </span>
              </span>
              <input
                type="checkbox"
                checked={publishDebuginfo}
                onChange={(event) => setPublishDebuginfo(event.target.checked)}
              />
            </label>

            <label className="flex items-center justify-between border-2 border-zinc-700 bg-zinc-950 px-4 py-3 md:col-span-2">
              <span>
                <span className="block font-mono text-xs font-bold uppercase tracking-[0.1em] text-white">
                  Network access
                </span>
                <span className="mt-1 block text-xs text-zinc-500">
                  Allow mock builds to access the network for packages that
                  cannot build fully offline.
                </span>
              </span>
              <input
                type="checkbox"
                checked={networkAccess}
                onChange={(event) => setNetworkAccess(event.target.checked)}
              />
            </label>
          </div>

          <div className="grid gap-4 lg:grid-cols-2">
            <div className="border-2 border-zinc-700 bg-zinc-950 p-4 lg:col-span-2">
              <div className="flex flex-col gap-3 md:flex-row md:items-center md:justify-between">
                <div>
                  <span className="block font-mono text-xs font-bold uppercase tracking-[0.15em] text-zinc-400">
                    Mock chroots
                  </span>
                  <span className="mt-1 block text-xs text-zinc-500">
                    Each selected chroot becomes a separate build job.
                  </span>
                </div>
                <button
                  type="button"
                  onClick={() => setShowChrootPicker(true)}
                  className="border-2 border-zinc-700 bg-black px-4 py-2 font-mono text-xs font-bold uppercase tracking-[0.15em] text-zinc-300 transition duration-100 ease-linear hover:-translate-x-[1px] hover:-translate-y-[1px] hover:border-white hover:bg-zinc-950"
                >
                  Choose chroots
                </button>
              </div>
              <div className="mt-4 border-2 border-zinc-700 bg-zinc-950 px-4 py-3 text-sm font-mono text-zinc-200">
                {chrootsLoading ? (
                  <MockTargetCheckIndicator label="Checking mock targets…" />
                ) : mockChroots.length > 0 ? (
                  formatMockChroots(mockChroots, "No chroots selected")
                ) : (
                  "No chroots selected"
                )}
              </div>
            </div>

            <label className="block">
              <span className="mb-2 block font-mono text-xs font-bold uppercase tracking-[0.15em] text-zinc-400">
                Poll interval (seconds)
              </span>
              <input
                type="number"
                min="1"
                step="1"
                value={pollIntervalSeconds}
                onChange={(event) => setPollIntervalSeconds(event.target.value)}
                className="w-full border-2 border-zinc-700 bg-zinc-950 px-4 py-3 font-mono text-sm text-white outline-none transition duration-100 ease-linear focus:border-[var(--theme-accent-lime)]"
                required
              />
            </label>

            <label className="block">
              <span className="mb-2 block font-mono text-xs font-bold uppercase tracking-[0.15em] text-zinc-400">
                Build timeout (seconds)
              </span>
              <input
                type="number"
                min="1"
                step="1"
                value={buildTimeoutSeconds}
                onChange={(event) => setBuildTimeoutSeconds(event.target.value)}
                className="w-full border-2 border-zinc-700 bg-zinc-950 px-4 py-3 font-mono text-sm text-white outline-none transition duration-100 ease-linear focus:border-[var(--theme-accent-lime)]"
                required
              />
            </label>

            <label className="block lg:col-span-2">
              <span className="mb-2 block font-mono text-xs font-bold uppercase tracking-[0.15em] text-zinc-400">
                History count
              </span>
              <input
                type="number"
                min="1"
                step="1"
                value={packageHistoryCount}
                onChange={(event) => setPackageHistoryCount(event.target.value)}
                className="w-full border-2 border-zinc-700 bg-zinc-950 px-4 py-3 font-mono text-sm text-white outline-none transition duration-100 ease-linear focus:border-[var(--theme-accent-lime)]"
                required
              />
            </label>

            <div className="border-2 border-zinc-700 bg-zinc-950 p-4 lg:col-span-2">
              <div className="flex flex-col gap-3 md:flex-row md:items-center md:justify-between">
                <span className="font-mono text-xs font-bold uppercase tracking-[0.15em] text-zinc-400">
                  CPU limit (cores)
                </span>
                <label className="flex items-center gap-2 font-mono text-xs font-bold uppercase tracking-[0.12em] text-zinc-300">
                  <input
                    type="checkbox"
                    checked={cpuLimitEnabled}
                    onChange={(event) => {
                      const nextEnabled = event.target.checked;
                      setCpuLimitEnabled(nextEnabled);
                      if (nextEnabled && !cpuLimitCores) {
                        setCpuLimitCores(String(CPU_DEFAULT_CORES));
                      }
                    }}
                  />
                  Limit CPU
                </label>
              </div>
              <div
                className={`mt-4 border-2 border-zinc-800 px-3 py-4 transition ${
                  cpuLimitEnabled ? "bg-black" : "bg-zinc-900/60 opacity-70"
                }`}
              >
                <input
                  type="range"
                  min={CPU_MIN_CORES}
                  max={cpuSliderMax}
                  step={CPU_STEP_CORES}
                  value={cpuSliderValue}
                  onChange={(event) => setCpuLimitCores(event.target.value)}
                  disabled={!cpuLimitEnabled}
                  aria-label="CPU limit in cores"
                  className="h-3 w-full cursor-pointer appearance-none bg-zinc-900 accent-[var(--theme-accent-lime)] disabled:cursor-not-allowed [&::-moz-range-thumb]:h-5 [&::-moz-range-thumb]:w-3 [&::-moz-range-thumb]:border-2 [&::-moz-range-thumb]:border-black [&::-moz-range-thumb]:bg-[var(--theme-accent-lime)] [&::-webkit-slider-thumb]:h-5 [&::-webkit-slider-thumb]:w-3 [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:border-2 [&::-webkit-slider-thumb]:border-black [&::-webkit-slider-thumb]:bg-[var(--theme-accent-lime)]"
                />
              </div>
              <div className="mt-3 flex items-center justify-between gap-3 font-mono text-xs font-bold uppercase tracking-[0.12em]">
                <span className="text-[var(--theme-accent-lime)]">
                  {cpuLimitEnabled ? `${cpuSliderValue} cores` : "Unlimited"}
                </span>
                <span className="text-zinc-500">
                  {CPU_MIN_CORES} - {cpuSliderMax} cores
                </span>
              </div>
            </div>

            <div className="border-2 border-zinc-700 bg-zinc-950 p-4 lg:col-span-2">
              <div className="flex flex-col gap-3 md:flex-row md:items-center md:justify-between">
                <span className="font-mono text-xs font-bold uppercase tracking-[0.15em] text-zinc-400">
                  Memory limit (MB)
                </span>
                <label className="flex items-center gap-2 font-mono text-xs font-bold uppercase tracking-[0.12em] text-zinc-300">
                  <input
                    type="checkbox"
                    checked={memoryLimitEnabled}
                    onChange={(event) => {
                      const nextEnabled = event.target.checked;
                      setMemoryLimitEnabled(nextEnabled);
                      if (nextEnabled && !memoryLimitMb) {
                        setMemoryLimitMb(String(MEMORY_DEFAULT_MB));
                      }
                    }}
                  />
                  Limit RAM
                </label>
              </div>
              <div
                className={`mt-4 border-2 border-zinc-800 px-3 py-4 transition ${
                  memoryLimitEnabled ? "bg-black" : "bg-zinc-900/60 opacity-70"
                }`}
              >
                <input
                  type="range"
                  min={MEMORY_MIN_MB}
                  max={memorySliderMax}
                  step={MEMORY_STEP_MB}
                  value={memorySliderValue}
                  onChange={(event) => setMemoryLimitMb(event.target.value)}
                  disabled={!memoryLimitEnabled}
                  aria-label="Memory limit in megabytes"
                  className="h-3 w-full cursor-pointer appearance-none bg-zinc-900 accent-[var(--theme-accent-lime)] disabled:cursor-not-allowed [&::-moz-range-thumb]:h-5 [&::-moz-range-thumb]:w-3 [&::-moz-range-thumb]:border-2 [&::-moz-range-thumb]:border-black [&::-moz-range-thumb]:bg-[var(--theme-accent-lime)] [&::-webkit-slider-thumb]:h-5 [&::-webkit-slider-thumb]:w-3 [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:border-2 [&::-webkit-slider-thumb]:border-black [&::-webkit-slider-thumb]:bg-[var(--theme-accent-lime)]"
                />
              </div>
              <div className="mt-3 flex items-center justify-between gap-3 font-mono text-xs font-bold uppercase tracking-[0.12em]">
                <span className="text-[var(--theme-accent-lime)]">
                  {memoryLimitEnabled ? `${memorySliderValue} MB` : "Unlimited"}
                </span>
                <span className="text-zinc-500">
                  {MEMORY_MIN_MB} - {memorySliderMax} MB
                </span>
              </div>
            </div>
          </div>

          <label className="block">
            <span className="mb-2 block font-mono text-xs font-bold uppercase tracking-[0.15em] text-zinc-400">
              Build environment
            </span>
            <textarea
              value={buildEnv}
              onChange={(event) => setBuildEnv(event.target.value)}
              rows={6}
              placeholder={
                "KEY=value\nMESON_ARGS=-Dgallium-drivers=swrast\nRUSTFLAGS=-C debuginfo=1"
              }
              className="w-full border-2 border-zinc-700 bg-zinc-950 px-4 py-3 font-mono text-sm text-white placeholder:text-zinc-600 outline-none transition duration-100 ease-linear focus:border-[var(--theme-accent-lime)]"
            />
            <span className="mt-2 block text-xs text-zinc-500">
              One `KEY=value` entry per line. Applied to SRPM creation and mock
              rebuild steps.
            </span>
          </label>

           <div className="border-2 border-zinc-700 bg-zinc-950 p-4">
            <div className="flex flex-col gap-3 md:flex-row md:items-center md:justify-between">
              <div>
                <div className="font-mono text-xs font-bold uppercase tracking-[0.15em] text-zinc-400">Spec file</div>
                <div className="mt-1 text-xs text-zinc-500">
                  Browse the repository and select the `.spec` file to build.
                </div>
              </div>
              <button
                type="button"
                onClick={() => setShowSpecPicker(true)}
                disabled={browsing}
                 className="border-2 border-zinc-700 bg-black px-4 py-2 font-mono text-xs font-bold uppercase tracking-[0.15em] text-zinc-300 transition duration-100 ease-linear hover:-translate-x-[1px] hover:-translate-y-[1px] hover:border-white hover:bg-zinc-950 disabled:opacity-60"
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
               className="mt-4 w-full border-2 border-zinc-700 bg-black px-4 py-3 font-mono text-sm text-white outline-none transition duration-100 ease-linear focus:border-[var(--theme-accent-lime)] focus:ring-2 focus:ring-[var(--theme-accent-lime)]"
            />
          </div>

          {error && (
             <div className="border-2 border-zinc-700 bg-black px-4 py-3 text-sm text-zinc-200">
              {error}
            </div>
          )}

          <div className="flex justify-end gap-3 pt-2">
            <button
              type="button"
              onClick={onClose}
               className="border-2 border-zinc-700 bg-black px-4 py-2 font-mono text-xs font-bold uppercase tracking-[0.15em] text-zinc-300 transition duration-100 ease-linear hover:-translate-x-[1px] hover:-translate-y-[1px] hover:border-white hover:bg-zinc-950"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={submitting}
               className="border-2 border-[var(--theme-accent-lime)] bg-[var(--theme-accent-lime)] px-4 py-2 font-mono text-xs font-bold uppercase tracking-[0.15em] text-black transition duration-100 ease-linear hover:-translate-x-[1px] hover:-translate-y-[1px] hover:bg-[#d8ff72] disabled:opacity-70"
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
                className="border-2 border-zinc-700 bg-black px-4 py-2 font-mono text-xs font-bold uppercase tracking-[0.15em] text-zinc-300 transition duration-100 ease-linear hover:-translate-x-[1px] hover:-translate-y-[1px] hover:border-white hover:bg-zinc-950 disabled:opacity-60"
            >
              <FaIcon icon={faMagnifyingGlass} className="mr-2" />
              {browsing ? "Cloning repository…" : "Load repository files"}
            </button>
            {(browsing || activeBrowseProgress) && (
              <div className="border-2 border-zinc-700 bg-zinc-950 px-4 py-3">
                <div className="flex items-center justify-between gap-3">
                  <span className="font-mono text-xs font-bold uppercase tracking-[0.15em] text-zinc-300">
                    Git clone progress
                  </span>
                  <span className="font-mono text-xs text-zinc-300">
                    {Math.round(browseProgressPercent)}%
                  </span>
                </div>
                <div className="mt-3 h-2 w-full overflow-hidden border border-zinc-700 bg-zinc-900">
                  <div
                    className={`h-full transition-[width] duration-300 ${
                      browseProgressState === "failed"
                        ? "bg-red-500"
                        : browseProgressState === "completed"
                          ? "bg-[var(--theme-terminal-green)]"
                          : "bg-[var(--theme-accent-lime)]"
                    }`}
                    style={{ width: `${Math.max(0, Math.min(100, browseProgressPercent))}%` }}
                  />
                </div>
                <p className="mt-2 font-mono text-xs uppercase tracking-[0.12em] text-zinc-400">
                  {stateLabel(browseProgressState)} · {browseProgressMessage}
                </p>
                {browseProgressIssue ? (
                  <p className="mt-2 text-xs text-zinc-500">{browseProgressIssue}</p>
                ) : null}
              </div>
            )}
            {browseError ? (
                <div className="border-2 border-zinc-700 bg-black px-4 py-3 text-sm text-zinc-200">
                {browseError}
              </div>
            ) : null}
             <div className="max-h-[50vh] overflow-auto border-2 border-zinc-700 bg-black">
              {selectableFiles.length > 0 ? (
                selectableFiles.map((file) => (
                  <button
                    key={file}
                    type="button"
                    onClick={() => {
                      setSpecPath(file);
                      setShowSpecPicker(false);
                    }}
                     className={`block w-full border-b-2 border-zinc-800 px-4 py-2 text-left font-mono text-sm transition last:border-b-0 ${
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
             <div className="max-h-[50vh] overflow-y-auto border-2 border-zinc-700 bg-black">
            {chrootsLoading ? (
              <div className="px-4 py-3">
                <MockTargetCheckIndicator label="Checking mock targets…" />
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
                      className="h-4 w-4 border-zinc-700 bg-zinc-900"
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
