import { Link } from "@tanstack/react-router";
import type { WorkspaceCcacheStats } from "../../lib/types";
import CcacheStatsCard from "../cache/ccache-stats-card";
import {
  formatCcacheCount,
  formatCcacheRate,
  getCcacheMetrics,
} from "../cache/ccache-metrics";

export default function WorkspaceCcacheSection({
  compilerCache,
}: {
  compilerCache: WorkspaceCcacheStats;
}) {
  const rankedTargets = compilerCache.targets
    .map((target) => ({ target, metrics: getCcacheMetrics(target.stats) }))
    .sort(
      (left, right) =>
        (left.metrics.hitRate ?? Number.POSITIVE_INFINITY) -
        (right.metrics.hitRate ?? Number.POSITIVE_INFINITY),
    )
    .slice(0, 8);

  return (
    <section className="space-y-4" aria-labelledby="compiler-cache-title">
      <div>
        <h2
          id="compiler-cache-title"
          className="font-mono text-sm font-bold text-white"
        >
          Build cache effectiveness
        </h2>
        <p className="mt-1.5 text-xs text-muted">
          Compiler-cache results recorded by completed builds across the
          workspace.
        </p>
      </div>

      <div className="grid gap-6 xl:grid-cols-[minmax(0,1.25fr)_minmax(320px,0.75fr)]">
        <CcacheStatsCard
          title="Workspace ccache"
          stats={compilerCache.stats}
          buildCount={compilerCache.build_count}
          description={`${compilerCache.targets.length} package/target combinations with recorded statistics`}
        />

        <article className="border border-edge-strong bg-surface shadow-card-sm">
          <header className="app-section-band px-6 py-4">
            <h3 className="font-mono text-sm font-bold uppercase tracking-[0.12em] text-white">
              What each cache does
            </h3>
          </header>
          <ol className="divide-y divide-edge">
            <CacheLayer
              number="01"
              title="Git mirror"
              description="Reuses fetched source history so syncs avoid full repository clones."
            />
            <CacheLayer
              number="02"
              title="Mock target discovery"
              description="Caches the worker image's available chroot list. It is control-plane metadata, not compiled output."
            />
            <CacheLayer
              number="03"
              title="Compiler ccache"
              description="Reuses object files per package and Mock target—the layer that most affects Mesa and QEMU rebuild time."
            />
          </ol>
        </article>
      </div>

      <article className="border border-edge bg-black">
        <header className="flex flex-wrap items-end justify-between gap-3 border-b border-edge px-6 py-4">
          <div>
            <h3 className="font-mono text-sm font-bold uppercase tracking-[0.1em] text-white">
              Lowest ccache hit rates
            </h3>
            <p className="mt-1 text-xs text-muted">
              Start here when a frequently rebuilt target remains cold.
            </p>
          </div>
          <span className="font-mono text-xs uppercase tracking-[0.12em] text-soft">
            Lowest {rankedTargets.length} shown
          </span>
        </header>

        {rankedTargets.length === 0 ? (
          <div className="px-6 py-10 text-center font-mono text-sm text-soft">
            No compiler-cache statistics have been recorded yet.
          </div>
        ) : (
          <div className="divide-y divide-edge">
            {rankedTargets.map(({ target, metrics }) => (
              <div
                key={`${target.package_name}:${target.mock_chroot}`}
                className="grid gap-3 px-6 py-4 md:grid-cols-[minmax(0,1.4fr)_minmax(0,1.2fr)_auto_auto] md:items-center"
              >
                <Link
                  to="/packages/view"
                  search={{ name: target.package_name }}
                  className="min-w-0 truncate font-mono text-sm font-bold text-white hover:text-accent-lime"
                >
                  {target.package_name}
                </Link>
                <span className="min-w-0 break-all font-mono text-xs text-muted">
                  {target.mock_chroot}
                </span>
                <div className="font-mono text-xs text-soft md:text-right">
                  <span className="font-bold text-strong">
                    {formatCcacheRate(metrics.hitRate)}
                  </span>{" "}
                  hit rate
                </div>
                <div className="font-mono text-xs text-soft md:text-right">
                  {formatCcacheCount(metrics.hits)} hits ·{" "}
                  {formatCcacheCount(target.stats.cache_misses)} misses ·{" "}
                  {formatCcacheCount(target.build_count)} builds
                </div>
              </div>
            ))}
          </div>
        )}
      </article>
    </section>
  );
}

function CacheLayer({
  number,
  title,
  description,
}: {
  number: string;
  title: string;
  description: string;
}) {
  return (
    <li className="flex gap-4 px-6 py-4">
      <span className="font-mono text-xs font-bold text-accent-cyan">
        {number}
      </span>
      <div>
        <h4 className="font-mono text-xs font-bold uppercase tracking-[0.12em] text-strong">
          {title}
        </h4>
        <p className="mt-1 text-xs leading-relaxed text-muted">{description}</p>
      </div>
    </li>
  );
}
