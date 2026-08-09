import {
  Disclosure,
  DisclosureGroup,
} from "../../../../components/ui/disclosure";
import {
  NumberField,
  TextAreaField,
  ToggleField,
} from "../../../../components/ui/form-fields";
import {
  CCACHE_SUPPORTED_ARCHES,
  incompatibleCcacheChroots,
} from "../../../../lib/utils";
import type { AddPackageFormState } from "./form-state";
import ResourceLimitSlider from "./resource-limit-slider";

interface Props {
  form: AddPackageFormState;
  maxCpuCores: number | null;
  maxMemoryMb: number | null;
  onChange: (next: Partial<AddPackageFormState>) => void;
}

const CPU_MIN_CORES = 1;
const MEMORY_MIN_MB = 256;
const MEMORY_STEP_MB = 256;

export default function BuildSettingsSection({
  form,
  maxCpuCores,
  maxMemoryMb,
  onChange,
}: Props) {
  const cpuSliderMax = Math.max(CPU_MIN_CORES, maxCpuCores ?? 64);
  const cpuDefaultCores = Math.min(4, cpuSliderMax);
  const cpuSliderValue = Math.min(
    cpuSliderMax,
    Math.max(CPU_MIN_CORES, Math.floor(Number(form.cpuLimitCores) || cpuDefaultCores)),
  );
  const memorySliderMax = Math.max(
    MEMORY_MIN_MB,
    maxMemoryMb
      ? Math.floor(maxMemoryMb / MEMORY_STEP_MB) * MEMORY_STEP_MB
      : 32768,
  );
  const memoryDefaultMb = Math.min(1024, memorySliderMax);
  const memorySliderValue = Math.min(
    memorySliderMax,
    Math.max(MEMORY_MIN_MB, Number(form.memoryLimitMb) || memoryDefaultMb),
  );

  return (
    <div className="space-y-4">
      <div className="grid gap-4 md:grid-cols-2">
        <ToggleField
          label="Shared ccache"
          description="Reuse compiler output for this package and target."
          checked={form.ccacheEnabled}
          onChange={(ccacheEnabled) => onChange({ ccacheEnabled })}
        />
        {form.ccacheEnabled ? (
          <NumberField
            label="Cache size per target (MB)"
            value={form.ccacheMaxSizeMb}
            onChange={(ccacheMaxSizeMb) => onChange({ ccacheMaxSizeMb })}
            min={1}
          />
        ) : null}
      </div>

      {form.ccacheEnabled ? (
        <CcacheCompatibilityNotice chroots={form.mockChroots} />
      ) : null}

      <NumberField
        label="Build timeout (seconds)"
        value={form.buildTimeoutSeconds}
        onChange={(buildTimeoutSeconds) => onChange({ buildTimeoutSeconds })}
        min={1}
        required
      />

      <DisclosureGroup>
        <Disclosure
          value="advanced-build"
          title="Advanced build settings"
          description="Polling cadence, retention, resource limits, network, and environment variables."
        >
          <div className="grid gap-4 md:grid-cols-2">
            {form.poll ? (
              <NumberField
                label="Poll interval (seconds)"
                value={form.pollIntervalSeconds}
                onChange={(pollIntervalSeconds) =>
                  onChange({ pollIntervalSeconds })
                }
                min={1}
                required
              />
            ) : null}
            <NumberField
              label="History count"
              value={form.packageHistoryCount}
              onChange={(packageHistoryCount) =>
                onChange({ packageHistoryCount })
              }
              min={1}
              required
            />

            <ResourceLimitSlider
              enabled={form.cpuLimitEnabled}
              label="CPU limit (cores)"
              max={cpuSliderMax}
              min={CPU_MIN_CORES}
              onEnabledChange={(cpuLimitEnabled) =>
                onChange({
                  cpuLimitEnabled,
                  cpuLimitCores:
                    cpuLimitEnabled && !form.cpuLimitCores
                      ? String(cpuDefaultCores)
                      : form.cpuLimitCores,
                })
              }
              onValueChange={(cpuLimitCores) => onChange({ cpuLimitCores })}
              step={1}
              toggleLabel="Limit CPU"
              unit="cores"
              value={cpuSliderValue}
            />

            <ResourceLimitSlider
              enabled={form.memoryLimitEnabled}
              label="Memory limit (MB)"
              max={memorySliderMax}
              min={MEMORY_MIN_MB}
              onEnabledChange={(memoryLimitEnabled) =>
                onChange({
                  memoryLimitEnabled,
                  memoryLimitMb:
                    memoryLimitEnabled && !form.memoryLimitMb
                      ? String(memoryDefaultMb)
                      : form.memoryLimitMb,
                })
              }
              onValueChange={(memoryLimitMb) => onChange({ memoryLimitMb })}
              step={MEMORY_STEP_MB}
              toggleLabel="Limit RAM"
              unit="MB"
              value={memorySliderValue}
            />

            <ToggleField
              className="md:col-span-2"
              label="Network access"
              description="Allow mock builds to access the network. Use only for packages that cannot build offline."
              checked={form.networkAccess}
              onChange={(networkAccess) => onChange({ networkAccess })}
            />

            <TextAreaField
              className="md:col-span-2"
              label="Build environment"
              value={form.buildEnv}
              onChange={(buildEnv) => onChange({ buildEnv })}
              rows={6}
              placeholder={
                "KEY=value\nMESON_ARGS=-Dgallium-drivers=swrast\nRUSTFLAGS=-C debuginfo=1"
              }
              hint="One KEY=value entry per line. Applied to SRPM creation and mock rebuild steps."
            />
          </div>
        </Disclosure>
      </DisclosureGroup>
    </div>
  );
}

function CcacheCompatibilityNotice({ chroots }: { chroots: string[] }) {
  const incompatible = incompatibleCcacheChroots(chroots);
  const supportedList = CCACHE_SUPPORTED_ARCHES.join(", ");

  return (
    <div
      className={
        incompatible.length > 0
          ? "border border-accent-orange bg-black px-4 py-3"
          : "border-l-2 border-edge-strong px-4 py-3"
      }
    >
      <p className="font-mono text-xs font-bold uppercase tracking-[0.16em] text-strong">
        ccache architectures: {supportedList}
      </p>
      {incompatible.length > 0 ? (
        <p className="mt-1 text-xs text-accent-orange">
          Unsupported selected targets: {incompatible.join(", ")}.
        </p>
      ) : null}
    </div>
  );
}
