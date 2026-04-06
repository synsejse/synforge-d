import { formatBytes } from "../../lib/bytes";
import type { RepoTargetSummary } from "../../lib/types";
import EmptyState from "../ui/EmptyState";

interface RepositoryTargetsSectionProps {
  targets: RepoTargetSummary[];
}

export default function RepositoryTargetsSection({
  targets,
}: RepositoryTargetsSectionProps) {
  return (
    <section className="border-2 border-zinc-700 bg-black p-6">
      <div className="flex items-end justify-between gap-4">
        <div>
          <div className="font-mono text-xs uppercase tracking-[0.22em] text-zinc-500">
            Build Targets
          </div>
          <h2 className="mt-2 font-mono text-xl font-bold uppercase text-white">
            Published Target Coverage
          </h2>
        </div>
        <div className="font-mono text-xs text-zinc-500">
          Per-target package, build, and size totals
        </div>
      </div>

      {targets.length === 0 ? (
        <div className="mt-5">
          <EmptyState>No published repository targets yet.</EmptyState>
        </div>
      ) : (
        <div className="mt-5 grid gap-4 md:grid-cols-2 xl:grid-cols-3">
          {targets.map((target) => (
            <article
              key={target.mock_chroot}
              className="border-2 border-zinc-700 bg-zinc-950/40 p-5"
            >
              <div className="font-mono text-lg font-bold text-white">
                {target.mock_chroot}
              </div>
              <div className="mt-4 grid gap-3 sm:grid-cols-3">
                <TargetStat label="Packages" value={target.package_count} />
                <TargetStat label="Builds" value={target.build_count} />
                <TargetStat label="Size" value={formatBytes(target.size_bytes)} />
              </div>
            </article>
          ))}
        </div>
      )}
    </section>
  );
}

function TargetStat({
  label,
  value,
}: {
  label: string;
  value: string | number;
}) {
  return (
    <div className="border-2 border-zinc-700 bg-black px-3 py-3">
      <div className="font-mono text-[10px] uppercase tracking-[0.18em] text-zinc-500">
        {label}
      </div>
      <div className="mt-2 font-mono text-sm font-medium text-zinc-200">{value}</div>
    </div>
  );
}
