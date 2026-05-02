import type {
  BuildJobResponse,
  JobResourceUsageSample,
  ServerHardwareResponse,
} from "../../../lib/types";
import EmptyState from "../../../components/ui/empty-state";
import type { JobViewMode } from "../types";
import JobCard from "./job-card";

interface JobListTableProps {
  jobs: BuildJobResponse[];
  killingJobId: string | null;
  mode: JobViewMode;
  onDelete: (job: BuildJobResponse) => void;
  onKill: (job: BuildJobResponse) => void;
  serverHardware: ServerHardwareResponse | null;
  usageByJob: Record<string, JobResourceUsageSample>;
}

export default function JobListTable({
  jobs,
  killingJobId,
  mode,
  onDelete,
  onKill,
  serverHardware,
  usageByJob,
}: JobListTableProps) {
  if (jobs.length === 0) {
    return (
      <div className="p-4">
        <EmptyState
          title={mode === "active" ? "No active jobs" : "No jobs found"}
          description={
            mode === "active"
              ? "Nothing is currently pending or running."
              : "Adjust filters or queue new builds to see results here."
          }
        />
      </div>
    );
  }

  return (
    <div className="space-y-3 p-4">
      {jobs.map((entry) => (
        <JobCard
          key={entry.job.id}
          entry={entry}
          mode={mode}
          killing={killingJobId === entry.job.id}
          usage={usageByJob[entry.job.id] ?? null}
          serverHardware={serverHardware}
          onKill={onKill}
          onDelete={onDelete}
        />
      ))}
    </div>
  );
}
