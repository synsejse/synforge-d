import type { BuildCcacheStats } from "../../lib/types";
import {
  formatCcacheCount,
  formatCcacheRate,
  getCcacheMetrics,
  type CcacheMetrics,
} from "./ccache-metrics";

interface Props {
  title: string;
  stats: BuildCcacheStats;
  buildCount?: number;
  description?: string;
}

const temperatureStyle: Record<
  CcacheMetrics["temperature"],
  { label: string; className: string }
> = {
  "no-calls": { label: "No calls", className: "border-edge text-soft" },
  cold: { label: "Cold", className: "border-accent-orange text-accent-orange" },
  warming: {
    label: "Warming",
    className: "border-accent-cyan text-accent-cyan",
  },
  warm: { label: "Warm", className: "border-accent-lime text-accent-lime" },
  hot: { label: "Hot", className: "border-success text-success" },
};

export default function CcacheStatsCard({
  title,
  stats,
  buildCount,
  description,
}: Props) {
  const metrics = getCcacheMetrics(stats);
  const temperature = temperatureStyle[metrics.temperature];
  const barWidth = Math.max(0, Math.min(100, metrics.hitRate ?? 0));

  return (
    <section className="border border-edge-strong bg-surface shadow-card-sm">
      <header className="app-section-band flex flex-wrap items-center justify-between gap-3 px-6 py-4">
        <div className="min-w-0">
          <h2 className="break-all font-mono text-sm font-bold uppercase tracking-[0.12em] text-white">
            {title}
          </h2>
          {description ? (
            <p className="mt-1 font-mono text-xs text-soft">
              {description}
            </p>
          ) : null}
        </div>
        <span
          className={`border bg-black px-2 py-1 font-mono text-xs font-bold uppercase tracking-[0.16em] ${temperature.className}`}
        >
          {temperature.label}
        </span>
      </header>

      <div className="space-y-4 p-5 sm:p-6">
        <div className="flex items-end justify-between gap-4">
          <div>
            <p className="font-mono text-xs font-semibold uppercase tracking-[0.18em] text-soft">
              Cacheable hit rate
            </p>
            <p className="mt-1 font-mono text-3xl font-bold text-accent-lime">
              {formatCcacheRate(metrics.hitRate)}
            </p>
          </div>
          {buildCount != null ? (
            <div className="text-right">
              <p className="font-mono text-xs font-semibold uppercase tracking-[0.18em] text-soft">
                Recorded builds
              </p>
              <p className="mt-1 font-mono text-lg font-bold text-strong">
                {formatCcacheCount(buildCount)}
              </p>
            </div>
          ) : null}
        </div>

        <div
          className="h-2 border border-edge-strong bg-black"
          aria-hidden="true"
        >
          <div
            className="h-full bg-accent-lime transition-[width] duration-500"
            style={{ width: `${barWidth}%` }}
          />
        </div>

        <dl className="grid grid-cols-2 gap-4 sm:grid-cols-3">
          <Metric label="Compiler calls" value={stats.compiler_calls} />
          <Metric label="Hits" value={metrics.hits} tone="hit" />
          <Metric label="Misses" value={stats.cache_misses} />
          <Metric label="Direct" value={stats.direct_hits} />
          <Metric label="Uncacheable" value={stats.uncacheable_calls} />
          <Metric label="Errors" value={stats.error_calls} tone="error" />
        </dl>

        <p className="font-mono text-xs leading-relaxed text-soft">
          Hit rate uses cacheable compiler calls only. Preprocessed hits:{" "}
          {formatCcacheCount(stats.preprocessed_hits)}.
        </p>
      </div>
    </section>
  );
}

function Metric({
  label,
  value,
  tone = "default",
}: {
  label: string;
  value: number;
  tone?: "default" | "hit" | "error";
}) {
  const valueClass =
    tone === "hit"
      ? "text-accent-lime"
      : tone === "error" && value > 0
        ? "text-error"
        : "text-strong";
  return (
    <div>
      <dt className="font-mono text-xs font-semibold uppercase tracking-[0.16em] text-soft">
        {label}
      </dt>
      <dd className={`mt-1 font-mono text-base font-bold ${valueClass}`}>
        {formatCcacheCount(value)}
      </dd>
    </div>
  );
}
