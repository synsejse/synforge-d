import type {
  JobResourceUsageSample,
  ServerHardwareResponse,
} from "../../../lib/types";

interface JobUsageBarProps {
  serverHardware: ServerHardwareResponse | null;
  usage: JobResourceUsageSample | null;
}

export default function JobUsageBar({
  serverHardware,
  usage,
}: JobUsageBarProps) {
  return (
    <div className="space-y-3">
      <UsageBarRow
        label="RAM"
        value={formatMemoryUsage(usage, serverHardware)}
        percent={memoryUsagePercent(usage, serverHardware)}
        fillClass="bg-accent-amber"
        valueClass="text-accent-amber"
      />
      <UsageBarRow
        label="CPU"
        value={formatCpuUsage(usage)}
        percent={cpuUsagePercent(usage)}
        fillClass="bg-accent-cyan"
        valueClass="text-accent-cyan"
      />
    </div>
  );
}

function formatCpuUsage(sample: JobResourceUsageSample | null): string {
  if (!sample) return "-";
  return `${Math.round(sample.cpu_percent)}% CPU`;
}

function cpuUsagePercent(sample: JobResourceUsageSample | null): number {
  if (!sample) return 0;
  const cores = sample.online_cpus > 0 ? sample.online_cpus : 1;
  return clampPercent((sample.cpu_percent / (cores * 100)) * 100);
}

function formatMemory(bytes: number) {
  if (bytes >= 1024 * 1024 * 1024) {
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GiB`;
  }
  return `${(bytes / (1024 * 1024)).toFixed(0)} MiB`;
}

function clampPercent(value: number): number {
  if (!Number.isFinite(value)) return 0;
  return Math.max(0, Math.min(100, value));
}

function resolveMemoryCapacityBytes(
  sample: JobResourceUsageSample,
  hardware: ServerHardwareResponse | null,
): number | null {
  if (sample.memory_limit_bytes > 0) {
    return sample.memory_limit_bytes;
  }
  const totalMemoryMb = hardware?.total_memory_mb;
  if (totalMemoryMb && totalMemoryMb > 0) {
    return totalMemoryMb * 1024 * 1024;
  }
  return null;
}

function memoryUsagePercent(
  sample: JobResourceUsageSample | null,
  hardware: ServerHardwareResponse | null,
): number {
  if (!sample) return 0;
  const memoryCapacityBytes = resolveMemoryCapacityBytes(sample, hardware);
  if (!memoryCapacityBytes || memoryCapacityBytes <= 0) return 0;
  return clampPercent((sample.memory_usage_bytes / memoryCapacityBytes) * 100);
}

function formatMemoryUsage(
  sample: JobResourceUsageSample | null,
  hardware: ServerHardwareResponse | null,
): string {
  if (!sample) return "-";
  const memoryCapacityBytes = resolveMemoryCapacityBytes(sample, hardware);
  if (memoryCapacityBytes && memoryCapacityBytes > 0) {
    return `${formatMemory(sample.memory_usage_bytes)} / ${formatMemory(memoryCapacityBytes)}`;
  }
  return formatMemory(sample.memory_usage_bytes);
}

function UsageBarRow({
  label,
  value,
  percent,
  fillClass,
  valueClass,
}: {
  label: string;
  value: string;
  percent: number;
  fillClass: string;
  valueClass: string;
}) {
  const hasSample = value !== "-";
  return (
    <div className="space-y-2">
      <div className="flex items-center justify-between gap-3 font-mono text-xs uppercase tracking-[0.12em]">
        <span className="text-soft">{label}</span>
        <span className={valueClass}>{value}</span>
      </div>
      <div className="h-5 border-2 border-edge-strong bg-black p-[3px]">
        <div
          className={`h-full transition-all duration-700 ${fillClass}`}
          style={{ width: `${hasSample ? percent : 0}%` }}
        />
      </div>
    </div>
  );
}
