import { Link } from "@tanstack/react-router";
import type { BuildJobResponse } from "../../../lib/types";
import { formatDateTime } from "../../../lib/datetime";
import StatusPill from "../../../components/ui/status-pill";
import { formatCompactId } from "../../../lib/identifiers";

export default function SyncBuildLinks({ builds }: { builds: BuildJobResponse[] }) {
  if (builds.length === 0) {
    return (
      <p className="font-mono text-sm text-soft">
        No builds were spawned. This is normal when the source is unchanged or targets are busy.
      </p>
    );
  }
  return (
    <div className="space-y-3">
      {builds.map(({ job }) => (
        <Link
          key={job.id}
          to="/jobs/view"
          search={{ id: job.id }}
          className="flex flex-col gap-3 border border-edge bg-surface-alt p-4 transition hover:border-accent-cyan sm:flex-row sm:items-center"
        >
          <div className="min-w-0 flex-1">
            <div className="font-mono text-sm font-bold text-white">{job.mock_chroot}</div>
            <div className="mt-1 break-all font-mono text-xs text-soft">
              <span title={job.id}>{formatCompactId(job.id)}</span> ·{" "}
              {formatDateTime(job.created_at)}
            </div>
          </div>
          <StatusPill status={job.status} />
        </Link>
      ))}
    </div>
  );
}
