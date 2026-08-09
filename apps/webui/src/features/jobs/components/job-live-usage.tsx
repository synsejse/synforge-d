import type {
  JobResourceUsageSample,
  ServerHardwareResponse,
} from "../../../lib/types";

interface Props {
  sample?: JobResourceUsageSample | null;
  hardware?: ServerHardwareResponse | null;
}

export default function JobLiveUsage({ sample, hardware }: Props) {
  return (
    <section
      aria-label="Live resource usage"
      className="flex flex-col gap-3 border border-accent-lime bg-black px-4 py-3 sm:flex-row sm:items-center sm:gap-x-6 sm:px-5"
    >
      <span className="font-mono text-[10px] font-bold uppercase tracking-[0.22em] text-accent-lime">
        Live
      </span>
      <LiveUsageMetric
        label="CPU"
        value={formatCpuUsage(sample)}
        percent={cpuUsagePercent(sample)}
        fillClass="bg-accent-lime"
      />
      <LiveUsageMetric
        label="Memory"
        value={formatMemoryUsage(sample, hardware)}
        percent={memoryUsagePercent(sample, hardware)}
        fillClass="bg-accent-cyan"
      />
    </section>
  );
}

function LiveUsageMetric({
  label,
  value,
  percent,
  fillClass,
}: {
  label: string;
  value: string;
  percent: number;
  fillClass: string;
}) {
  const hasSample = value !== "-";
  return (
    <div className="flex flex-1 flex-wrap items-center gap-x-3 gap-y-1">
      <span className="font-mono text-[10px] font-bold uppercase tracking-[0.22em] text-soft">
        {label}
      </span>
      <span className="font-mono text-sm font-bold text-white">{value}</span>
      <div className="flex min-w-[140px] flex-1 items-center gap-2">
        <div className="h-2 flex-1 border border-edge-strong bg-black">
          <div
            className={`h-full transition-all duration-500 ${fillClass}`}
            style={{ width: `${hasSample ? percent : 0}%` }}
          />
        </div>
        <span className="font-mono text-xs text-soft">
          {hasSample ? `${percent.toFixed(1)}%` : "—"}
        </span>
      </div>
    </div>
  );
}

function formatMemory(bytes: number): string {
  if (bytes >= 1024 * 1024 * 1024) {
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GiB`;
  }
  return `${(bytes / (1024 * 1024)).toFixed(0)} MiB`;
}

function resolveMemoryCapacityBytes(
  sample: JobResourceUsageSample,
  hardware?: ServerHardwareResponse | null,
): number | null {
  if (sample.memory_limit_bytes > 0) return sample.memory_limit_bytes;
  const totalMemoryMb = hardware?.total_memory_mb;
  return totalMemoryMb && totalMemoryMb > 0
    ? totalMemoryMb * 1024 * 1024
    : null;
}

function formatMemoryUsage(
  sample?: JobResourceUsageSample | null,
  hardware?: ServerHardwareResponse | null,
): string {
  if (!sample) return "-";
  const capacityBytes = resolveMemoryCapacityBytes(sample, hardware);
  return capacityBytes && capacityBytes > 0
    ? `${formatMemory(sample.memory_usage_bytes)} / ${formatMemory(capacityBytes)}`
    : formatMemory(sample.memory_usage_bytes);
}

function clampPercent(value: number): number {
  if (!Number.isFinite(value)) return 0;
  return Math.max(0, Math.min(100, value));
}

function memoryUsagePercent(
  sample?: JobResourceUsageSample | null,
  hardware?: ServerHardwareResponse | null,
): number {
  if (!sample) return 0;
  const capacityBytes = resolveMemoryCapacityBytes(sample, hardware);
  if (!capacityBytes || capacityBytes <= 0) return 0;
  return clampPercent((sample.memory_usage_bytes / capacityBytes) * 100);
}

function formatCpuUsage(sample?: JobResourceUsageSample | null): string {
  return sample ? `${Math.round(sample.cpu_percent)}% CPU` : "-";
}

function cpuUsagePercent(sample?: JobResourceUsageSample | null): number {
  if (!sample) return 0;
  const cores = sample.online_cpus > 0 ? sample.online_cpus : 1;
  return clampPercent(sample.cpu_percent / cores);
}
