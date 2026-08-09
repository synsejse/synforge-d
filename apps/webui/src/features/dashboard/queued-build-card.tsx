import { Link } from "@tanstack/react-router";
import type { BuildJobResponse } from "../../lib/types";
import { formatDurationSeconds } from "../../lib/datetime";
import { formatCompactId } from "../../lib/identifiers";
import StatusPill from "../../components/ui/status-pill";

interface Props {
  entry: BuildJobResponse;
  position: number;
  queueLength: number;
  now: number;
}

export default function QueuedBuildCard({
  entry,
  position,
  queueLength,
  now,
}: Props) {
  const job = entry.job;
  const createdAt = Date.parse(job.created_at);
  const queuedFor = formatDurationSeconds(
    Number.isFinite(createdAt) ? Math.max(0, (now - createdAt) / 1000) : 0,
  );

  return (
    <Link
      to="/jobs/view"
      search={{ id: job.id }}
      className="block border border-edge border-l-2 border-l-accent-orange bg-surface-alt p-4 transition-colors hover:border-accent-orange"
    >
      <div className="flex flex-wrap items-center gap-3">
        <span className="font-mono text-sm font-bold uppercase text-white">
          {job.package_name}
        </span>
        <StatusPill status="pending" />
        <span className="border border-edge bg-black px-2 py-1 font-mono text-xs uppercase text-muted">
          {job.mock_chroot}
        </span>
        <span className="ml-auto font-mono text-xs font-bold text-accent-orange">
          Queue {position} of {queueLength}
        </span>
      </div>
      <div className="mt-3 flex flex-wrap items-center justify-between gap-3 font-mono text-xs text-soft">
        <span>Queued for {queuedFor}</span>
        <span title={job.id}>{formatCompactId(job.id)}</span>
      </div>
    </Link>
  );
}
