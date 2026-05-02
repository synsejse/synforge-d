import { faMagnifyingGlass, faSave } from "@fortawesome/free-solid-svg-icons";
import type { SyntheticEvent } from "react";
import { formatMockChroots } from "../../../lib/utils";
import FaIcon from "../../../components/ui/fa-icon";
import SelectionDialog from "../../../components/common/selection-dialog";
import {
  TextField,
  NumberField,
  TextAreaField,
  ToggleField,
  FieldGroup,
  DisplayBox,
} from "../../../components/ui/form-fields";

export interface PackageEditFormState {
  repoUrl: string;
  specPath: string;
  poll: boolean;
  mockChroots: string[];
  pollIntervalSeconds: string;
  buildTimeoutSeconds: string;
  packageHistoryCount: string;
  cpuLimitCores: string;
  cpuLimitEnabled: boolean;
  memoryLimitEnabled: boolean;
  memoryLimitMb: string;
  ccache_enabled: boolean;
  ccacheMaxSizeMb: string;
  buildEnv: string;
  enabled: boolean;
  publish_srpm: boolean;
  publish_debuginfo: boolean;
  network_access: boolean;
}

interface PackageEditFormSectionProps {
  form: PackageEditFormState;
  maxCpuCores: number | null;
  maxMemoryMb: number | null;
  saving: boolean;
  availableChroots: string[];
  showSpecPicker: boolean;
  showChrootPicker: boolean;
  browsing: boolean;
  browseError: string | null;
  selectableFiles: string[];
  onSubmit: (event: SyntheticEvent<HTMLFormElement>) => void;
  onFormChange: (next: Partial<PackageEditFormState>) => void;
  onToggleChroot: (chroot: string, checked: boolean) => void;
  onOpenSpecPicker: () => void;
  onCloseSpecPicker: () => void;
  onOpenChrootPicker: () => void;
  onCloseChrootPicker: () => void;
  onBrowseRepository: () => void;
}

export default function PackageEditFormSection({
  form,
  maxCpuCores,
  maxMemoryMb,
  saving,
  availableChroots,
  showSpecPicker,
  showChrootPicker,
  browsing,
  browseError,
  selectableFiles,
  onSubmit,
  onFormChange,
  onToggleChroot,
  onOpenSpecPicker,
  onCloseSpecPicker,
  onOpenChrootPicker,
  onCloseChrootPicker,
  onBrowseRepository,
}: PackageEditFormSectionProps) {
  const CPU_MIN_CORES = 1;
  const CPU_STEP_CORES = 1;
  const cpuSliderMax = Math.max(CPU_MIN_CORES, maxCpuCores ?? 64);
  const CPU_DEFAULT_CORES = Math.min(4, cpuSliderMax);
  const cpuSliderValue = Math.min(
    cpuSliderMax,
    Math.max(CPU_MIN_CORES, Math.floor(Number(form.cpuLimitCores) || CPU_DEFAULT_CORES)),
  );
  const MEMORY_MIN_MB = 256;
  const MEMORY_STEP_MB = 256;
  const memorySliderMax = Math.max(
    MEMORY_MIN_MB,
    maxMemoryMb
      ? Math.floor(maxMemoryMb / MEMORY_STEP_MB) * MEMORY_STEP_MB
      : 32768,
  );
  const MEMORY_DEFAULT_MB = Math.min(1024, memorySliderMax);
  const memorySliderValue = Math.min(
    memorySliderMax,
    Math.max(MEMORY_MIN_MB, Number(form.memoryLimitMb) || MEMORY_DEFAULT_MB),
  );

  return (
    <>
      <form onSubmit={onSubmit} className="min-w-0 border-2 border-edge-strong bg-black p-4 sm:p-6">
        <div className="mb-6">
          <h2 className="text-xl font-bold text-white">Edit package</h2>
          <p className="mt-2 text-sm text-muted">
            Update the tracked repository, selected spec path, polling behavior,
            and package state from one place.
          </p>
        </div>

        <div className="space-y-5">
          <TextField
            label="Git repository URL"
            value={form.repoUrl}
            onChange={(value) => onFormChange({ repoUrl: value })}
            type="url"
            required
          />

          <FieldGroup
            label="Repository spec path"
            description="Choose the .spec file from the tracked repository."
            action={
              <button
                type="button"
                onClick={onOpenSpecPicker}
                className="border-2 border-edge-strong bg-black px-4 py-2 font-mono text-xs font-bold uppercase tracking-[0.15em] text-muted transition duration-100 ease-linear hover:-translate-x-[1px] hover:-translate-y-[1px] hover:border-white hover:bg-surface-alt"
              >
                <FaIcon icon={faMagnifyingGlass} className="mr-2" />
                Browse repository
              </button>
            }
          >
            <input
              type="text"
              value={form.specPath}
              onChange={(event) => onFormChange({ specPath: event.target.value })}
              placeholder="path/to/package.spec"
              className="w-full border-2 border-edge-strong bg-black px-4 py-3 font-mono text-sm text-white placeholder:text-soft outline-none transition duration-100 ease-linear focus:border-accent-lime"
              required
            />
          </FieldGroup>

          <div className="grid gap-4 md:grid-cols-2">
            <NumberField
              label="Poll interval (seconds)"
              value={form.pollIntervalSeconds}
              onChange={(value) => onFormChange({ pollIntervalSeconds: value })}
              required
            />

            <NumberField
              label="Build timeout (seconds)"
              value={form.buildTimeoutSeconds}
              onChange={(value) => onFormChange({ buildTimeoutSeconds: value })}
              required
            />

            <NumberField
              label="History count"
              value={form.packageHistoryCount}
              onChange={(value) => onFormChange({ packageHistoryCount: value })}
              required
              className="md:col-span-2"
            />

            <NumberField
              label="Shared ccache size (MB)"
              value={form.ccacheMaxSizeMb}
              onChange={(value) => onFormChange({ ccacheMaxSizeMb: value })}
              min={1}
              className="md:col-span-2"
            />
            <p className="md:col-span-2 -mt-2 text-xs text-soft">
              Leave blank to use Mock&apos;s default cache size. Applies per package and mock chroot.
            </p>

            <div className="border-2 border-edge-strong bg-surface-alt p-4 md:col-span-2">
              <div className="flex flex-col gap-3 md:flex-row md:items-center md:justify-between">
                <span className="font-mono text-xs font-bold uppercase tracking-[0.15em] text-muted">
                  CPU limit (cores)
                </span>
                <label className="flex items-center gap-2 font-mono text-xs font-bold uppercase tracking-[0.12em] text-muted">
                  <input
                    type="checkbox"
                    checked={form.cpuLimitEnabled}
                    onChange={(event) =>
                      onFormChange({
                        cpuLimitEnabled: event.target.checked,
                        cpuLimitCores: event.target.checked
                          ? form.cpuLimitCores || String(CPU_DEFAULT_CORES)
                          : form.cpuLimitCores,
                      })
                    }
                  />
                  Limit CPU
                </label>
              </div>
              <div
                className={`mt-4 border-2 border-edge px-3 py-4 transition ${
                  form.cpuLimitEnabled ? "bg-black" : "bg-surface-hover/60 opacity-70"
                }`}
              >
                <input
                  type="range"
                  min={CPU_MIN_CORES}
                  max={cpuSliderMax}
                  step={CPU_STEP_CORES}
                  value={cpuSliderValue}
                  onChange={(event) =>
                    onFormChange({ cpuLimitCores: event.target.value })
                  }
                  disabled={!form.cpuLimitEnabled}
                  aria-label="CPU limit in cores"
                  className="h-3 w-full cursor-pointer appearance-none bg-surface-hover accent-accent-lime disabled:cursor-not-allowed [&::-moz-range-thumb]:h-5 [&::-moz-range-thumb]:w-3 [&::-moz-range-thumb]:border-2 [&::-moz-range-thumb]:border-black [&::-moz-range-thumb]:bg-accent-lime [&::-webkit-slider-thumb]:h-5 [&::-webkit-slider-thumb]:w-3 [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:border-2 [&::-webkit-slider-thumb]:border-black [&::-webkit-slider-thumb]:bg-accent-lime"
                />
              </div>
              <div className="mt-3 flex items-center justify-between gap-3 font-mono text-xs font-bold uppercase tracking-[0.12em]">
                <span className="text-accent-lime">
                  {form.cpuLimitEnabled ? `${cpuSliderValue} cores` : "Unlimited"}
                </span>
                <span className="text-soft">
                  {CPU_MIN_CORES} - {cpuSliderMax} cores
                </span>
              </div>
            </div>

            <div className="border-2 border-edge-strong bg-surface-alt p-4 md:col-span-2">
              <div className="flex flex-col gap-3 md:flex-row md:items-center md:justify-between">
                <span className="font-mono text-xs font-bold uppercase tracking-[0.15em] text-muted">
                  Memory limit (MB)
                </span>
                <label className="flex items-center gap-2 font-mono text-xs font-bold uppercase tracking-[0.12em] text-muted">
                  <input
                    type="checkbox"
                    checked={form.memoryLimitEnabled}
                    onChange={(event) =>
                      onFormChange({
                        memoryLimitEnabled: event.target.checked,
                        memoryLimitMb: event.target.checked
                          ? form.memoryLimitMb || String(MEMORY_DEFAULT_MB)
                          : form.memoryLimitMb,
                      })
                    }
                  />
                  Limit RAM
                </label>
              </div>
              <div
                className={`mt-4 border-2 border-edge px-3 py-4 transition ${
                  form.memoryLimitEnabled ? "bg-black" : "bg-surface-hover/60 opacity-70"
                }`}
              >
                <input
                  type="range"
                  min={MEMORY_MIN_MB}
                  max={memorySliderMax}
                  step={MEMORY_STEP_MB}
                  value={memorySliderValue}
                  onChange={(event) =>
                    onFormChange({ memoryLimitMb: event.target.value })
                  }
                  disabled={!form.memoryLimitEnabled}
                  aria-label="Memory limit in megabytes"
                  className="h-3 w-full cursor-pointer appearance-none bg-surface-hover accent-accent-lime disabled:cursor-not-allowed [&::-moz-range-thumb]:h-5 [&::-moz-range-thumb]:w-3 [&::-moz-range-thumb]:border-2 [&::-moz-range-thumb]:border-black [&::-moz-range-thumb]:bg-accent-lime [&::-webkit-slider-thumb]:h-5 [&::-webkit-slider-thumb]:w-3 [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:border-2 [&::-webkit-slider-thumb]:border-black [&::-webkit-slider-thumb]:bg-accent-lime"
                />
              </div>
              <div className="mt-3 flex items-center justify-between gap-3 font-mono text-xs font-bold uppercase tracking-[0.12em]">
                <span className="text-accent-lime">
                  {form.memoryLimitEnabled ? `${memorySliderValue} MB` : "Unlimited"}
                </span>
                <span className="text-soft">
                  {MEMORY_MIN_MB} - {memorySliderMax} MB
                </span>
              </div>
            </div>

            <FieldGroup
              label="Mock chroots"
              description="Each selected chroot becomes a separate build job."
              className="md:col-span-2"
              action={
                <button
                  type="button"
                  onClick={onOpenChrootPicker}
                  className="border-2 border-edge-strong bg-black px-4 py-2 font-mono text-xs font-bold uppercase tracking-[0.15em] text-muted transition duration-100 ease-linear hover:-translate-x-[1px] hover:-translate-y-[1px] hover:border-white hover:bg-surface-alt"
                >
                  Choose chroots
                </button>
              }
            >
              <DisplayBox>
                {form.mockChroots.length > 0
                  ? formatMockChroots(form.mockChroots, "No chroots selected")
                  : "No chroots selected"}
              </DisplayBox>
            </FieldGroup>

            <ToggleField
              label="Shared ccache"
              description="Reuse compiler cache across builds for this package and mock chroot."
              checked={form.ccache_enabled}
              onChange={(checked) => onFormChange({ ccache_enabled: checked })}
            />

            <ToggleField
              label="Enabled"
              description="Allow new builds for this package."
              checked={form.enabled}
              onChange={(checked) => onFormChange({ enabled: checked })}
            />

            <ToggleField
              label="Source Polling"
              description="Watch the tracked git repository for new commits."
              checked={form.poll}
              onChange={(checked) => onFormChange({ poll: checked })}
            />

            <ToggleField
              label="Publish SRPM"
              description="Keep source RPM publication enabled for this package."
              checked={form.publish_srpm}
              onChange={(checked) => onFormChange({ publish_srpm: checked })}
            />

            <ToggleField
              label="Publish debug packages"
              description="Include debuginfo and debugsource RPMs in repository."
              checked={form.publish_debuginfo}
              onChange={(checked) => onFormChange({ publish_debuginfo: checked })}
            />

            <ToggleField
              label="Network access"
              description="Allow mock builds for this package to access the network."
              checked={form.network_access}
              onChange={(checked) => onFormChange({ network_access: checked })}
              className="md:col-span-2"
            />
          </div>

          <TextAreaField
            label="Build environment"
            value={form.buildEnv}
            onChange={(value) => onFormChange({ buildEnv: value })}
            placeholder="KEY=value&#10;MESON_ARGS=-Dgallium-drivers=swrast&#10;RUSTFLAGS=-C debuginfo=1"
            hint="One `KEY=value` entry per line. Applied to SRPM creation and mock rebuild steps."
          />

          <div className="flex justify-end">
            <button
              type="submit"
              disabled={saving}
              className="border-2 border-accent-lime bg-accent-lime px-5 py-2.5 font-mono text-xs font-bold uppercase tracking-[0.15em] text-black transition duration-100 ease-linear hover:-translate-x-[1px] hover:-translate-y-[1px] hover:bg-[#d8ff72] disabled:opacity-70"
            >
              <FaIcon icon={faSave} className="mr-2" />
              {saving ? "Saving…" : "Save Changes"}
            </button>
          </div>
        </div>
      </form>

      {showSpecPicker && (
        <SelectionDialog
          title="Choose spec file"
          subtitle="Browse the tracked repository and select the .spec file to build."
          onClose={onCloseSpecPicker}
        >
          <div className="space-y-4">
            <button
              type="button"
              onClick={onBrowseRepository}
              disabled={browsing}
              className="border-2 border-edge-strong bg-black px-4 py-2 font-mono text-xs font-bold uppercase tracking-[0.15em] text-muted transition duration-100 ease-linear hover:-translate-x-[1px] hover:-translate-y-[1px] hover:border-white hover:bg-surface-alt disabled:opacity-60"
            >
              <FaIcon icon={faMagnifyingGlass} className="mr-2" />
              {browsing ? "Browsing…" : "Load repository files"}
            </button>
            {browseError ? (
              <div className="border-2 border-edge-strong bg-black px-4 py-3 text-sm text-strong">
                {browseError}
              </div>
            ) : null}
            <div className="max-h-[50vh] overflow-auto border-2 border-edge-strong bg-black">
              {selectableFiles.length > 0 ? (
                selectableFiles.map((file) => (
                  <button
                    key={file}
                    type="button"
                    onClick={() => {
                      onFormChange({ specPath: file });
                      onCloseSpecPicker();
                    }}
                    className={`block w-full break-all border-b-2 border-edge px-4 py-2 text-left font-mono text-sm transition last:border-b-0 ${
                      form.specPath === file
                        ? "bg-surface-alt text-white"
                        : "bg-black text-muted hover:bg-surface-alt"
                    }`}
                  >
                    {file}
                  </button>
                ))
              ) : (
                <div className="px-4 py-3 text-sm text-muted">
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
          onClose={onCloseChrootPicker}
        >
          <div className="max-h-[50vh] overflow-y-auto border-2 border-edge-strong bg-black">
            <div className="divide-y divide-edge">
              {availableChroots.map((chroot) => (
                <label
                  key={chroot}
                  className="flex items-center justify-between gap-4 px-4 py-3 text-sm text-strong hover:bg-surface-alt"
                >
                  <span className="font-mono">{chroot}</span>
                  <input
                    type="checkbox"
                    checked={form.mockChroots.includes(chroot)}
                    onChange={(event) =>
                      onToggleChroot(chroot, event.target.checked)
                    }
                  />
                </label>
              ))}
            </div>
          </div>
        </SelectionDialog>
      )}
    </>
  );
}
