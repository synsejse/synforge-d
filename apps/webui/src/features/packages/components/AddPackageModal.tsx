import {
  useEffect,
  useId,
  useMemo,
  useRef,
  useState,
  type SyntheticEvent,
} from "react";
import { faPlus } from "@fortawesome/free-solid-svg-icons";
import { packagesApi } from "../api";
import type {
  BrowseRepositoryProgressView,
  CreatePackageRequest,
  ServerHardwareResponse,
  SpecSource,
} from "../../../lib/types";
import FaIcon from "../../../components/ui/FaIcon";
import BuildSettingsSection from "./add-package/BuildSettingsSection";
import ChrootPickerDialog from "./add-package/ChrootPickerDialog";
import {
  encodeBuildEnv,
  parseBuildEnv,
  parseOptionalCpuLimit,
  parseOptionalMegabytes,
} from "./add-package/form-utils";
import SourceBasicsSection from "./add-package/SourceBasicsSection";
import SpecPickerDialog from "./add-package/SpecPickerDialog";

interface AddPackageModalProps {
  onClose: () => void;
  onSuccess: () => void;
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
  const [ccacheMaxSizeMb, setCcacheMaxSizeMb] = useState("");
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
        const response = await packagesApi.listMockChroots();
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
    packagesApi
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
        const response = await packagesApi.getBrowseRepositoryProgress();
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
      const response = await packagesApi.browseRepository({ repo_url: trimmedRepoUrl });
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
        const progressResponse = await packagesApi.getBrowseRepositoryProgress();
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
      ccache_max_size_mb: parseOptionalMegabytes(ccacheMaxSizeMb),
      build_env: parseBuildEnv(buildEnv),
    };

    try {
      await packagesApi.createPackage(request);
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
          <SourceBasicsSection
            ccacheEnabled={ccacheEnabled}
            ccacheMaxSizeMb={ccacheMaxSizeMb}
            enabled={enabled}
            name={name}
            networkAccess={networkAccess}
            poll={poll}
            publishDebuginfo={publishDebuginfo}
            publishSrpm={publishSrpm}
            repoUrl={repoUrl}
            setCcacheEnabled={setCcacheEnabled}
            setCcacheMaxSizeMb={setCcacheMaxSizeMb}
            setEnabled={setEnabled}
            setName={setName}
            setNetworkAccess={setNetworkAccess}
            setPoll={setPoll}
            setPublishDebuginfo={setPublishDebuginfo}
            setPublishSrpm={setPublishSrpm}
            setRepoUrl={setRepoUrl}
          />

          <BuildSettingsSection
            buildEnv={buildEnv}
            buildTimeoutSeconds={buildTimeoutSeconds}
            browsing={browsing}
            chrootsLoading={chrootsLoading}
            cpuDefaultCores={CPU_DEFAULT_CORES}
            cpuLimitCores={cpuLimitCores}
            cpuLimitEnabled={cpuLimitEnabled}
            cpuSliderMax={cpuSliderMax}
            cpuSliderValue={cpuSliderValue}
            memoryDefaultMb={MEMORY_DEFAULT_MB}
            memoryLimitEnabled={memoryLimitEnabled}
            memoryLimitMb={memoryLimitMb}
            memorySliderMax={memorySliderMax}
            memorySliderValue={memorySliderValue}
            mockChroots={mockChroots}
            onChooseChroots={() => setShowChrootPicker(true)}
            onChooseSpec={() => setShowSpecPicker(true)}
            packageHistoryCount={packageHistoryCount}
            pollIntervalSeconds={pollIntervalSeconds}
            setBuildEnv={setBuildEnv}
            setBuildTimeoutSeconds={setBuildTimeoutSeconds}
            setCpuLimitCores={setCpuLimitCores}
            setCpuLimitEnabled={setCpuLimitEnabled}
            setMemoryLimitEnabled={setMemoryLimitEnabled}
            setMemoryLimitMb={setMemoryLimitMb}
            setPackageHistoryCount={setPackageHistoryCount}
            setPollIntervalSeconds={setPollIntervalSeconds}
            setSpecPath={setSpecPath}
            specPath={specPath}
          />

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
        <SpecPickerDialog
          activeBrowseProgress={activeBrowseProgress}
          browseError={browseError}
          browseProgressIssue={browseProgressIssue}
          browseProgressMessage={browseProgressMessage}
          browseProgressPercent={browseProgressPercent}
          browseProgressState={browseProgressState}
          browsing={browsing}
          onBrowse={handleBrowse}
          onClose={() => setShowSpecPicker(false)}
          onSelectSpec={(file) => {
            setSpecPath(file);
            setShowSpecPicker(false);
          }}
          selectableFiles={selectableFiles}
          specPath={specPath}
        />
      )}

      {showChrootPicker && (
        <ChrootPickerDialog
          availableChroots={availableChroots}
          chrootsLoading={chrootsLoading}
          mockChroots={mockChroots}
          onClose={() => setShowChrootPicker(false)}
          onToggleChroot={toggleChroot}
        />
      )}
    </div>
  );
}
