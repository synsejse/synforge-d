import type {
  BuildJobResponse,
  JobResourceUsageSample,
} from "../../lib/types";
import LoadingBlock from "../../components/ui/loading-block";
import InFlightCard from "./in-flight-card";
import QueuedBuildCard from "./queued-build-card";

interface Props {
  loading: boolean;
  jobs: BuildJobResponse[];
  usageByJob: Map<string, JobResourceUsageSample>;
  now: number;
}

export default function ActiveBuildsSection({
  loading,
  jobs,
  usageByJob,
  now,
}: Props) {
  const runningCount = jobs.filter(
    (entry) => entry.job.status === "running",
  ).length;
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
    <section className="border border-edge bg-black">
      <header className="flex items-center justify-between gap-4 border-b border-edge px-6 py-4">
        <div className="flex items-center gap-2.5">
          <span className="relative flex h-2 w-2">
            {runningCount > 0 ? (
              <span className="absolute inline-flex h-full w-full animate-ping bg-accent-lime opacity-75" />
            ) : null}
            <span
              className={`relative inline-flex h-2 w-2 ${runningCount > 0 ? "bg-accent-lime" : "bg-soft"}`}
            />
          </span>
          <h2 className="font-mono text-sm font-bold uppercase tracking-[0.06em] text-white">
            Active builds
          </h2>
        </div>
        {!loading ? (
          <span className="font-mono text-xs font-bold uppercase tracking-[0.12em] text-soft">
            {jobs.length} active · {runningCount} building
          </span>
        ) : null}
      </header>

      {loading ? (
        <div className="p-5 sm:p-6">
          <LoadingBlock label="Loading active builds…" lines={2} />
        </div>
      ) : jobs.length === 0 ? (
        <div className="p-5 sm:p-6">
          <div className="flex min-h-28 items-center justify-center border border-dashed border-edge px-5 py-8 font-mono text-sm text-soft">
            Nothing is queued or building right now.
          </div>
        </div>
      ) : (
        <div className="space-y-4 p-5 sm:p-6">
          {jobs.map((entry) =>
            entry.job.status === "pending" ? (
              <QueuedBuildCard
                key={entry.job.id}
                entry={entry}
                position={queuePositions.get(entry.job.id) ?? 1}
                queueLength={pending.length}
                now={now}
              />
            ) : (
              <InFlightCard
                key={entry.job.id}
                entry={entry}
                usage={usageByJob.get(entry.job.id) ?? null}
                now={now}
              />
            ),
          )}
        </div>
      )}
    </section>
  );
}
