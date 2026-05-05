import { faMagnifyingGlass, faSave } from "@fortawesome/free-solid-svg-icons";
import type { SyntheticEvent } from "react";
import {
  CCACHE_SUPPORTED_ARCHES,
  formatMockChroots,
  incompatibleCcacheChroots,
} from "../../../lib/utils";
import Button from "../../../components/ui/button";
import { Disclosure, DisclosureGroup } from "../../../components/ui/disclosure";
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
  pristine: PackageEditFormState | null;
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
  onDiscard: () => void;
}

export default function PackageEditFormSection({
  form,
  pristine,
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
  onDiscard,
}: PackageEditFormSectionProps) {
  const isDirty =
    pristine != null && JSON.stringify(form) !== JSON.stringify(pristine);
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
      <form onSubmit={onSubmit} className="min-w-0 space-y-4">
        {/* Always-visible essentials — Git URL, spec, chroots, enabled.
            Most package edits touch these and these only. */}
        <section className="space-y-5 border-2 border-edge-strong bg-black p-4 sm:p-6">
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
              <Button
                type="button"
                variant="ghost"
                size="sm"
                onClick={onOpenSpecPicker}
              >
                <FaIcon icon={faMagnifyingGlass} />
                Browse repository
              </Button>
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

          <FieldGroup
            label="Mock chroots"
            description="Each selected chroot becomes a separate build job."
            action={
              <Button
                type="button"
                variant="ghost"
                size="sm"
                onClick={onOpenChrootPicker}
              >
                Choose chroots
              </Button>
            }
          >
            <DisplayBox>
              {form.mockChroots.length > 0
                ? formatMockChroots(form.mockChroots, "No chroots selected")
                : "No chroots selected"}
            </DisplayBox>
          </FieldGroup>

          <ToggleField
            label="Enabled"
            description="Allow new builds for this package."
            checked={form.enabled}
            onChange={(checked) => onFormChange({ enabled: checked })}
          />
        </section>

        {/* Advanced — three groups, all collapsed by default. */}
        <DisclosureGroup>
          <Disclosure
            value="behavior"
            title="Build behavior"
            description="Polling, publishing, network access, and environment variables."
          >
            <div className="space-y-5">
              <div className="grid gap-4 md:grid-cols-2">
                <ToggleField
                  label="Source polling"
                  description="Watch the tracked git repository for new commits."
                  checked={form.poll}
                  onChange={(checked) => onFormChange({ poll: checked })}
                />
                <ToggleField
                  label="Network access"
                  description="Allow mock builds to access the network."
                  checked={form.network_access}
                  onChange={(checked) => onFormChange({ network_access: checked })}
                />
                <ToggleField
                  label="Publish SRPM"
                  description="Keep source RPM publication enabled."
                  checked={form.publish_srpm}
                  onChange={(checked) => onFormChange({ publish_srpm: checked })}
                />
                <ToggleField
                  label="Publish debug packages"
                  description="Include debuginfo and debugsource RPMs."
                  checked={form.publish_debuginfo}
                  onChange={(checked) =>
                    onFormChange({ publish_debuginfo: checked })
                  }
                />
              </div>

              <TextAreaField
                label="Build environment"
                value={form.buildEnv}
                onChange={(value) => onFormChange({ buildEnv: value })}
                placeholder="KEY=value&#10;MESON_ARGS=-Dgallium-drivers=swrast&#10;RUSTFLAGS=-C debuginfo=1"
                hint="One `KEY=value` entry per line. Applied to SRPM creation and mock rebuild steps."
              />
            </div>
          </Disclosure>

          <Disclosure
            value="limits"
            title="Resource limits"
            description="CPU, memory, and shared ccache for builds of this package."
          >
            <div className="space-y-4">
              <ResourceLimitCard
                label="CPU limit"
                unit="cores"
                checkboxLabel="Limit CPU"
                enabled={form.cpuLimitEnabled}
                value={cpuSliderValue}
                min={CPU_MIN_CORES}
                max={cpuSliderMax}
                step={CPU_STEP_CORES}
                onToggle={(next) =>
                  onFormChange({
                    cpuLimitEnabled: next,
                    cpuLimitCores: next
                      ? form.cpuLimitCores || String(CPU_DEFAULT_CORES)
                      : form.cpuLimitCores,
                  })
                }
                onSliderChange={(next) =>
                  onFormChange({ cpuLimitCores: String(next) })
                }
              />

              <ResourceLimitCard
                label="Memory limit"
                unit="MB"
                checkboxLabel="Limit RAM"
                enabled={form.memoryLimitEnabled}
                value={memorySliderValue}
                min={MEMORY_MIN_MB}
                max={memorySliderMax}
                step={MEMORY_STEP_MB}
                onToggle={(next) =>
                  onFormChange({
                    memoryLimitEnabled: next,
                    memoryLimitMb: next
                      ? form.memoryLimitMb || String(MEMORY_DEFAULT_MB)
                      : form.memoryLimitMb,
                  })
                }
                onSliderChange={(next) =>
                  onFormChange({ memoryLimitMb: String(next) })
                }
              />

              {/* Twin cards: matching outer frame + height so the
                  shared-ccache toggle and its size input read as a
                  single paired control instead of misaligned columns. */}
              <div className="grid gap-4 md:grid-cols-[minmax(0,1fr)_minmax(0,220px)] md:items-stretch">
                <ToggleField
                  label="Shared ccache"
                  description="Reuse compiler cache across builds for this package and mock chroot."
                  checked={form.ccache_enabled}
                  onChange={(checked) =>
                    onFormChange({ ccache_enabled: checked })
                  }
                />
                <label className="flex h-full flex-col justify-between gap-2 border-2 border-edge-strong bg-surface-alt px-4 py-3">
                  <span className="block font-mono text-xs font-bold uppercase tracking-[0.1em] text-white">
                    ccache size (MB)
                  </span>
                  <input
                    type="number"
                    min={1}
                    step={1}
                    value={form.ccacheMaxSizeMb}
                    onChange={(event) =>
                      onFormChange({ ccacheMaxSizeMb: event.target.value })
                    }
                    className="w-full border-2 border-edge-strong bg-black px-3 py-2 font-mono text-sm text-white outline-none transition duration-100 ease-linear focus:border-accent-lime focus:ring-2 focus:ring-accent-lime"
                  />
                </label>
              </div>
              <p className="font-mono text-xs text-soft">
                Leave size blank to use Mock&apos;s default cache size. Applies
                per package and mock chroot.
              </p>
              {form.ccache_enabled ? (
                <CcacheCompatibilityNotice chroots={form.mockChroots} />
              ) : null}
            </div>
          </Disclosure>

          <Disclosure
            value="schedule"
            title="Schedule & retention"
            description="Polling cadence, build timeout, and how many old builds to keep."
          >
            <div className="grid gap-4 md:grid-cols-3">
              <NumberField
                label="Poll interval (seconds)"
                value={form.pollIntervalSeconds}
                onChange={(value) =>
                  onFormChange({ pollIntervalSeconds: value })
                }
                required
              />
              <NumberField
                label="Build timeout (seconds)"
                value={form.buildTimeoutSeconds}
                onChange={(value) =>
                  onFormChange({ buildTimeoutSeconds: value })
                }
                required
              />
              <NumberField
                label="History count"
                value={form.packageHistoryCount}
                onChange={(value) =>
                  onFormChange({ packageHistoryCount: value })
                }
                required
              />
            </div>
          </Disclosure>
        </DisclosureGroup>
      </form>

      {/* Sticky save bar — shown only while the form is dirty. */}
      {isDirty ? (
        <div
          role="region"
          aria-label="Unsaved changes"
          className="sticky bottom-0 z-30 -mx-3 mt-4 border-t-4 border-accent-lime bg-black/95 px-4 py-3 backdrop-blur-sm sm:-mx-5 lg:-mx-8"
        >
          <div className="mx-auto flex max-w-[96rem] flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
            <div className="flex items-center gap-3">
              <span
                aria-hidden="true"
                className="inline-block h-2 w-2 animate-pulse bg-accent-lime"
              />
              <span className="font-mono text-xs font-bold uppercase tracking-[0.18em] text-accent-lime">
                Unsaved changes
              </span>
            </div>
            <div className="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-end">
              <Button
                variant="ghost"
                size="sm"
                fullWidth="responsive"
                onClick={onDiscard}
                disabled={saving}
              >
                Discard
              </Button>
              <Button
                variant="primary"
                size="sm"
                fullWidth="responsive"
                onClick={(event) =>
                  onSubmit(event as unknown as SyntheticEvent<HTMLFormElement>)
                }
                loading={saving}
              >
                {saving ? null : <FaIcon icon={faSave} />}
                {saving ? "Saving…" : "Save changes"}
              </Button>
            </div>
          </div>
        </div>
      ) : null}

      {showSpecPicker && (
        <SelectionDialog
          title="Choose spec file"
          subtitle="Browse the tracked repository and select the .spec file to build."
          onClose={onCloseSpecPicker}
        >
          <div className="space-y-4">
            <Button
              type="button"
              variant="ghost"
              size="sm"
              onClick={onBrowseRepository}
              loading={browsing}
              disabled={browsing}
            >
              {browsing ? null : <FaIcon icon={faMagnifyingGlass} />}
              {browsing ? "Browsing…" : "Load repository files"}
            </Button>
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

interface ResourceLimitCardProps {
  label: string;
  unit: string;
  checkboxLabel: string;
  enabled: boolean;
  value: number;
  min: number;
  max: number;
  step: number;
  onToggle: (next: boolean) => void;
  onSliderChange: (value: number) => void;
}

function ResourceLimitCard({
  label,
  unit,
  checkboxLabel,
  enabled,
  value,
  min,
  max,
  step,
  onToggle,
  onSliderChange,
}: ResourceLimitCardProps) {
  return (
    <div className="border-2 border-edge bg-surface-alt/40 p-4">
      <div className="flex flex-col gap-3 md:flex-row md:items-center md:justify-between">
        <span className="font-mono text-xs font-bold uppercase tracking-[0.15em] text-muted">
          {label}
        </span>
        <label className="flex items-center gap-2 font-mono text-xs font-bold uppercase tracking-[0.12em] text-muted">
          <input
            type="checkbox"
            checked={enabled}
            onChange={(event) => onToggle(event.target.checked)}
          />
          {checkboxLabel}
        </label>
      </div>
      <div
        className={`mt-3 transition ${
          enabled ? "" : "opacity-40 pointer-events-none"
        }`}
      >
        <input
          type="range"
          min={min}
          max={max}
          step={step}
          value={value}
          onChange={(event) => onSliderChange(Number(event.target.value))}
          disabled={!enabled}
          aria-label={label}
          className="h-2 w-full cursor-pointer appearance-none bg-edge-strong accent-accent-lime disabled:cursor-not-allowed [&::-moz-range-thumb]:h-4 [&::-moz-range-thumb]:w-3 [&::-moz-range-thumb]:border-2 [&::-moz-range-thumb]:border-black [&::-moz-range-thumb]:bg-accent-lime [&::-webkit-slider-thumb]:h-4 [&::-webkit-slider-thumb]:w-3 [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:border-2 [&::-webkit-slider-thumb]:border-black [&::-webkit-slider-thumb]:bg-accent-lime"
        />
        <div className="mt-2 flex items-center justify-between gap-3 font-mono text-xs font-bold uppercase tracking-[0.12em]">
          <span className="text-accent-lime">
            {enabled ? `${value} ${unit}` : "Unlimited"}
          </span>
          <span className="text-soft">
            {min} – {max} {unit}
          </span>
        </div>
      </div>
    </div>
  );
}

function CcacheCompatibilityNotice({ chroots }: { chroots: string[] }) {
  const incompatible = incompatibleCcacheChroots(chroots);
  const supportedList = CCACHE_SUPPORTED_ARCHES.join(", ");

  if (incompatible.length === 0) {
    return (
      <p className="border-l-2 border-edge-strong px-3 py-2 font-mono text-xs text-soft">
        ccache only works with{" "}
        <span className="text-strong">{supportedList}</span>. Builds for any
        other arch will fail or silently bypass the cache.
      </p>
    );
  }

  return (
    <div className="border-2 border-accent-orange bg-black px-3 py-2">
      <p className="font-mono text-[10px] font-bold uppercase tracking-[0.22em] text-accent-orange">
        ccache incompatible with selected targets
      </p>
      <p className="mt-1 font-mono text-xs text-strong">
        These chroots will fail or skip ccache:{" "}
        <span className="text-accent-orange">{incompatible.join(", ")}</span>.
        Mock&apos;s ccache plugin only supports{" "}
        <span className="text-strong">{supportedList}</span>.
      </p>
    </div>
  );
}
