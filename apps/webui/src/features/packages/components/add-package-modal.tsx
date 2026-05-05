import {
  useEffect,
  useId,
  useMemo,
  useRef,
  useState,
  type SyntheticEvent,
} from "react";
import { faPlus } from "@fortawesome/free-solid-svg-icons";
import api from "../../../lib/api";
import type { CreatePackageRequest, SpecSource } from "../../../lib/types";
import Button from "../../../components/ui/button";
import FaIcon from "../../../components/ui/fa-icon";
import { useServerHardware } from "../../../components/common/server-hardware-provider";
import BuildSettingsSection from "./add-package/build-settings-section";
import ChrootPickerDialog from "./add-package/chroot-picker-dialog";
import {
  encodeBuildEnv,
  parseBuildEnv,
  parseOptionalCpuLimit,
  parseOptionalMegabytes,
} from "./add-package/form-utils";
import SourceBasicsSection from "./add-package/source-basics-section";
import SpecPickerDialog from "./add-package/spec-picker-dialog";

interface AddPackageModalProps {
  onClose: () => void;
  onSuccess: () => void;
}

export default function AddPackageModal({
  onClose,
  onSuccess,
}: AddPackageModalProps) {
  const serverHardware = useServerHardware();
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
    const firstFocusable = dialogRef.current?.querySelector<HTMLElement>(
      'input, select, textarea, button, [href], [tabindex]:not([tabindex="-1"])',
    );
    firstFocusable?.focus();
  }, []);

  async function handleBrowse() {
    const trimmedRepoUrl = repoUrl.trim();
    if (!trimmedRepoUrl) {
      setBrowseError("Repository URL is required before browsing.");
      return;
    }
    setBrowsing(true);
    setBrowseError(null);
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
        className="flex max-h-[calc(100dvh-3rem)] w-full max-w-3xl flex-col border-4 border-white bg-black shadow-modal"
      >
        <div className="border-b-2 border-edge px-6 py-5">
          <h2 id={titleId} className="text-2xl font-bold text-white">
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
            mockChroots={mockChroots}
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
             <div className="border-2 border-edge-strong bg-black px-4 py-3 text-sm text-strong">
              {error}
            </div>
          )}

          <div className="flex justify-end gap-3 pt-2">
            <Button variant="ghost" size="sm" onClick={onClose}>
              Cancel
            </Button>
            <Button
              type="submit"
              variant="primary"
              size="sm"
              loading={submitting}
            >
              {submitting ? null : <FaIcon icon={faPlus} />}
              {submitting ? "Adding…" : "Add Package"}
            </Button>
          </div>
        </form>
      </div>

      {showSpecPicker && (
        <SpecPickerDialog
          browseError={browseError}
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
