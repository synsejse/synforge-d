import type { PackageTargetCcacheStats } from "../../../lib/types";
import CcacheStatsCard from "../../cache/ccache-stats-card";

interface Props {
  targets: PackageTargetCcacheStats[];
  enabled: boolean;
}

export default function PackageCcacheSummary({ targets, enabled }: Props) {
  if (targets.length === 0) {
    return (
      <section className="border border-edge bg-surface-alt p-5 sm:p-6">
        <h2 className="font-mono text-sm font-bold uppercase tracking-[0.12em] text-white">
          Compiler cache
        </h2>
        <p className="mt-2 font-mono text-xs leading-relaxed text-soft">
          {enabled
            ? "No ccache statistics have been recorded yet. Complete a build to establish the baseline."
            : "ccache is disabled for this package, and no historical cache statistics are available."}
        </p>
      </section>
    );
  }

  return (
    <section aria-labelledby="package-ccache-heading" className="space-y-4">
      <div>
        <h2
          id="package-ccache-heading"
          className="font-mono text-sm font-bold uppercase tracking-[0.12em] text-white"
        >
          Compiler cache by target
        </h2>
        <p className="mt-1 font-mono text-[11px] text-soft">
          Lifetime package totals, including retained statistics from pruned
          builds.
        </p>
      </div>
      <div className="grid gap-4 xl:grid-cols-2">
        {targets.map((target) => (
          <CcacheStatsCard
            key={target.mock_chroot}
            title={target.mock_chroot}
            stats={target.stats}
            buildCount={target.build_count}
            description="Package and Mock-target cache"
          />
        ))}
      </div>
    </section>
  );
}
