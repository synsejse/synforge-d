import type {
  BuildJobResponse,
  JobResourceUsageSample,
} from "../../../lib/types";
import type { JobViewMode } from "../types";
import JobCard from "./job-card";

interface JobListTableProps {
  jobs: BuildJobResponse[];
  killingJobId: string | null;
  mode: JobViewMode;
  onDelete: (job: BuildJobResponse) => void;
  onRetry: (job: BuildJobResponse) => void;
  onKill: (job: BuildJobResponse) => void;
  usageByJob: Record<string, JobResourceUsageSample>;
}

export default function JobListTable({
  jobs,
  killingJobId,
  mode,
  onDelete,
  onRetry,
  onKill,
  usageByJob,
}: JobListTableProps) {
  const pending = jobs
    .filter((entry) => entry.job.status === "pending")
    .sort(
      (left, right) =>
        Date.parse(left.job.created_at) - Date.parse(right.job.created_at),
    );
  const queuePositions = new Map(
    pending.map((entry, index) => [entry.job.id, index + 1]),
  );

  return (
    <div className="flex flex-col gap-3">
      {jobs.map((entry) => (
        <JobCard
          key={entry.job.id}
          entry={entry}
          mode={mode}
          killing={killingJobId === entry.job.id}
          usage={usageByJob[entry.job.id] ?? null}
          queuePosition={queuePositions.get(entry.job.id) ?? null}
          queueLength={pending.length}
          onKill={onKill}
          onRetry={onRetry}
          onDelete={onDelete}
        />
      ))}
    </div>
  );
}
