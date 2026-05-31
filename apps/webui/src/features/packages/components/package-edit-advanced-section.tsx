import { Disclosure, DisclosureGroup } from "../../../components/ui/disclosure";
import {
  NumberField,
  TextAreaField,
  ToggleField,
} from "../../../components/ui/form-fields";
import {
  CcacheCompatibilityNotice,
  ResourceLimitCard,
} from "./package-edit-limits";
import type { PackageEditFormState } from "./package-edit-form-state";

interface PackageEditAdvancedSectionProps {
  form: PackageEditFormState;
  maxCpuCores: number | null;
  maxMemoryMb: number | null;
  onFormChange: (next: Partial<PackageEditFormState>) => void;
}

export default function PackageEditAdvancedSection({
  form,
  maxCpuCores,
  maxMemoryMb,
  onFormChange,
}: PackageEditAdvancedSectionProps) {
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
    maxMemoryMb ? Math.floor(maxMemoryMb / MEMORY_STEP_MB) * MEMORY_STEP_MB : 32768,
  );
  const MEMORY_DEFAULT_MB = Math.min(1024, memorySliderMax);
  const memorySliderValue = Math.min(
    memorySliderMax,
    Math.max(MEMORY_MIN_MB, Number(form.memoryLimitMb) || MEMORY_DEFAULT_MB),
  );

  return (
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
              onChange={(checked) => onFormChange({ publish_debuginfo: checked })}
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
            onSliderChange={(next) => onFormChange({ cpuLimitCores: String(next) })}
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
            onSliderChange={(next) => onFormChange({ memoryLimitMb: String(next) })}
          />

          {/* Twin cards: matching outer frame + height so the
              shared-ccache toggle and its size input read as a
              single paired control instead of misaligned columns. */}
          <div className="grid gap-4 md:grid-cols-[minmax(0,1fr)_minmax(0,220px)] md:items-stretch">
            <ToggleField
              label="Shared ccache"
              description="Reuse compiler cache across builds for this package and mock chroot."
              checked={form.ccache_enabled}
              onChange={(checked) => onFormChange({ ccache_enabled: checked })}
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
            Leave size blank to use Mock&apos;s default cache size. Applies per
            package and mock chroot.
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
          />
        </div>
      </Disclosure>
    </DisclosureGroup>
  );
}
