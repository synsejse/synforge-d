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
  if (jobs.length === 0) {
    return (
      <div className="border border-dashed border-edge px-5 py-[60px] text-center font-mono text-[13px] text-[#52525b]">
        {mode === "active" ? "No active jobs." : "No jobs found."}
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-3">
      {jobs.map((entry) => (
        <JobCard
          key={entry.job.id}
          entry={entry}
          mode={mode}
          killing={killingJobId === entry.job.id}
          usage={usageByJob[entry.job.id] ?? null}
          onKill={onKill}
          onRetry={onRetry}
          onDelete={onDelete}
        />
      ))}
    </div>
  );
}
