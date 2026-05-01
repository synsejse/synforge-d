import { faMagnifyingGlass } from "@fortawesome/free-solid-svg-icons";
import { formatMockChroots } from "../../../../lib/utils";
import FaIcon from "../../../../components/ui/fa-icon";
import MockTargetCheckIndicator from "./mock-target-check-indicator";
import ResourceLimitSlider from "./resource-limit-slider";

interface BuildSettingsSectionProps {
  buildEnv: string;
  buildTimeoutSeconds: string;
  browsing: boolean;
  chrootsLoading: boolean;
  cpuDefaultCores: number;
  cpuLimitCores: string;
  cpuLimitEnabled: boolean;
  cpuSliderMax: number;
  cpuSliderValue: number;
  memoryDefaultMb: number;
  memoryLimitEnabled: boolean;
  memoryLimitMb: string;
  memorySliderMax: number;
  memorySliderValue: number;
  mockChroots: string[];
  onChooseChroots: () => void;
  onChooseSpec: () => void;
  packageHistoryCount: string;
  pollIntervalSeconds: string;
  setBuildEnv: (value: string) => void;
  setBuildTimeoutSeconds: (value: string) => void;
  setCpuLimitCores: (value: string) => void;
  setCpuLimitEnabled: (value: boolean) => void;
  setMemoryLimitEnabled: (value: boolean) => void;
  setMemoryLimitMb: (value: string) => void;
  setPackageHistoryCount: (value: string) => void;
  setPollIntervalSeconds: (value: string) => void;
  setSpecPath: (value: string) => void;
  specPath: string;
}

const CPU_MIN_CORES = 1;
const CPU_STEP_CORES = 1;
const MEMORY_MIN_MB = 256;
const MEMORY_STEP_MB = 256;

export default function BuildSettingsSection({
  buildEnv,
  buildTimeoutSeconds,
  browsing,
  chrootsLoading,
  cpuDefaultCores,
  cpuLimitCores,
  cpuLimitEnabled,
  cpuSliderMax,
  cpuSliderValue,
  memoryDefaultMb,
  memoryLimitEnabled,
  memoryLimitMb,
  memorySliderMax,
  memorySliderValue,
  mockChroots,
  onChooseChroots,
  onChooseSpec,
  packageHistoryCount,
  pollIntervalSeconds,
  setBuildEnv,
  setBuildTimeoutSeconds,
  setCpuLimitCores,
  setCpuLimitEnabled,
  setMemoryLimitEnabled,
  setMemoryLimitMb,
  setPackageHistoryCount,
  setPollIntervalSeconds,
  setSpecPath,
  specPath,
}: BuildSettingsSectionProps) {
  return (
    <>
      <div className="grid gap-4 lg:grid-cols-2">
        <div className="border-2 border-edge-strong bg-surface-alt p-4 lg:col-span-2">
          <div className="flex flex-col gap-3 md:flex-row md:items-center md:justify-between">
            <div>
              <span className="block font-mono text-xs font-bold uppercase tracking-[0.15em] text-muted">
                Mock chroots
              </span>
              <span className="mt-1 block text-xs text-soft">
                Each selected chroot becomes a separate build job.
              </span>
            </div>
            <button
              type="button"
              onClick={onChooseChroots}
              className="border-2 border-edge-strong bg-black px-4 py-2 font-mono text-xs font-bold uppercase tracking-[0.15em] text-muted transition duration-100 ease-linear hover:-translate-x-[1px] hover:-translate-y-[1px] hover:border-white hover:bg-surface-alt"
            >
              Choose chroots
            </button>
          </div>
          <div className="mt-4 border-2 border-edge-strong bg-surface-alt px-4 py-3 font-mono text-sm text-strong">
            {chrootsLoading ? (
              <MockTargetCheckIndicator label="Checking mock targets…" />
            ) : mockChroots.length > 0 ? (
              formatMockChroots(mockChroots, "No chroots selected")
            ) : (
              "No chroots selected"
            )}
          </div>
        </div>

        <NumberField
          label="Poll interval (seconds)"
          onChange={setPollIntervalSeconds}
          value={pollIntervalSeconds}
        />
        <NumberField
          label="Build timeout (seconds)"
          onChange={setBuildTimeoutSeconds}
          value={buildTimeoutSeconds}
        />
        <NumberField
          className="lg:col-span-2"
          label="History count"
          onChange={setPackageHistoryCount}
          value={packageHistoryCount}
        />

        <ResourceLimitSlider
          enabled={cpuLimitEnabled}
          label="CPU limit (cores)"
          max={cpuSliderMax}
          min={CPU_MIN_CORES}
          onEnabledChange={(nextEnabled) => {
            setCpuLimitEnabled(nextEnabled);
            if (nextEnabled && !cpuLimitCores) {
              setCpuLimitCores(String(cpuDefaultCores));
            }
          }}
          onValueChange={setCpuLimitCores}
          step={CPU_STEP_CORES}
          toggleLabel="Limit CPU"
          unit="cores"
          value={cpuSliderValue}
        />

        <ResourceLimitSlider
          enabled={memoryLimitEnabled}
          label="Memory limit (MB)"
          max={memorySliderMax}
          min={MEMORY_MIN_MB}
          onEnabledChange={(nextEnabled) => {
            setMemoryLimitEnabled(nextEnabled);
            if (nextEnabled && !memoryLimitMb) {
              setMemoryLimitMb(String(memoryDefaultMb));
            }
          }}
          onValueChange={setMemoryLimitMb}
          step={MEMORY_STEP_MB}
          toggleLabel="Limit RAM"
          unit="MB"
          value={memorySliderValue}
        />
      </div>

      <label className="block">
        <span className="mb-2 block font-mono text-xs font-bold uppercase tracking-[0.15em] text-muted">
          Build environment
        </span>
        <textarea
          value={buildEnv}
          onChange={(event) => setBuildEnv(event.target.value)}
          rows={6}
          placeholder={
            "KEY=value\nMESON_ARGS=-Dgallium-drivers=swrast\nRUSTFLAGS=-C debuginfo=1"
          }
          className="w-full border-2 border-edge-strong bg-surface-alt px-4 py-3 font-mono text-sm text-white placeholder:text-soft outline-none transition duration-100 ease-linear focus:border-accent-lime"
        />
        <span className="mt-2 block text-xs text-soft">
          One `KEY=value` entry per line. Applied to SRPM creation and mock
          rebuild steps.
        </span>
      </label>

      <div className="border-2 border-edge-strong bg-surface-alt p-4">
        <div className="flex flex-col gap-3 md:flex-row md:items-center md:justify-between">
          <div>
            <div className="font-mono text-xs font-bold uppercase tracking-[0.15em] text-muted">
              Spec file
            </div>
            <div className="mt-1 text-xs text-soft">
              Browse the repository and select the `.spec` file to build.
            </div>
          </div>
          <button
            type="button"
            onClick={onChooseSpec}
            disabled={browsing}
            className="border-2 border-edge-strong bg-black px-4 py-2 font-mono text-xs font-bold uppercase tracking-[0.15em] text-muted transition duration-100 ease-linear hover:-translate-x-[1px] hover:-translate-y-[1px] hover:border-white hover:bg-surface-alt disabled:opacity-60"
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
          className="mt-4 w-full border-2 border-edge-strong bg-black px-4 py-3 font-mono text-sm text-white outline-none transition duration-100 ease-linear focus:border-accent-lime focus:ring-2 focus:ring-accent-lime"
        />
      </div>
    </>
  );
}

function NumberField({
  className = "",
  label,
  onChange,
  value,
}: {
  className?: string;
  label: string;
  onChange: (value: string) => void;
  value: string;
}) {
  return (
    <label className={`block ${className}`}>
      <span className="mb-2 block font-mono text-xs font-bold uppercase tracking-[0.15em] text-muted">
        {label}
      </span>
      <input
        type="number"
        min="1"
        step="1"
        value={value}
        onChange={(event) => onChange(event.target.value)}
        className="w-full border-2 border-edge-strong bg-surface-alt px-4 py-3 font-mono text-sm text-white outline-none transition duration-100 ease-linear focus:border-accent-lime"
        required
      />
    </label>
  );
}
