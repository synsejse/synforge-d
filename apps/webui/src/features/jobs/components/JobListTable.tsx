import {
  faStop,
  faTrash,
} from "@fortawesome/free-solid-svg-icons";
import type {
  BuildJobResponse,
  JobResourceUsageSample,
  ServerHardwareResponse,
} from "../../../lib/types";
import { formatDateTime, formatDurationBetween } from "../../../lib/datetime";
import Badge from "../../../components/ui/Badge";
import Button from "../../../components/ui/Button";
import FaIcon from "../../../components/ui/FaIcon";
import type { JobViewMode } from "../types";
import JobUsageBar from "./JobUsageBar";

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
      <div className="flex min-h-[300px] items-center justify-center px-6 py-12">
        <div className="text-center">
          <div className="font-mono text-sm text-zinc-500">
            {mode === "active" ? "No active jobs" : "No jobs found"}
          </div>
        </div>
      </div>
    );
  }

  return (
    <>
      <div className="space-y-3 p-4 md:hidden">
        {jobs.map((entry) => (
          <MobileJobCard
            key={`mobile:${entry.job.id}`}
            entry={entry}
            killingJobId={killingJobId}
            mode={mode}
            onDelete={onDelete}
            onKill={onKill}
            serverHardware={serverHardware}
            usage={usageByJob[entry.job.id] ?? null}
          />
        ))}
      </div>
      <div className="hidden overflow-x-auto md:block">
        <table className="w-full min-w-[640px] lg:min-w-[980px]">
          <thead className="border-b-2 border-[var(--theme-border-strong)] bg-zinc-950">
            <tr>
              {[
                "Package",
                "Target",
                "Revision",
                "Status",
                "Duration",
                "Created",
              ].map((label) => (
                <th
                  key={label}
                  scope="col"
                  className="px-5 py-4 text-left font-mono text-xs font-bold uppercase tracking-[0.2em] text-zinc-500"
                >
                  {label}
                </th>
              ))}
              {mode === "active" && (
                <th
                  scope="col"
                  className="px-5 py-4 text-left font-mono text-xs font-bold uppercase tracking-[0.2em] text-zinc-500"
                >
                  Live Usage
                </th>
              )}
              <th
                scope="col"
                className="px-5 py-4 text-left font-mono text-xs font-bold uppercase tracking-[0.2em] text-zinc-500"
              >
                Actions
              </th>
            </tr>
          </thead>
          <tbody>
            {jobs.map((entry, idx) => (
              <DesktopJobRow
                key={entry.job.id}
                entry={entry}
                index={idx}
                killingJobId={killingJobId}
                mode={mode}
                onDelete={onDelete}
                onKill={onKill}
                serverHardware={serverHardware}
                usage={usageByJob[entry.job.id] ?? null}
              />
            ))}
          </tbody>
        </table>
      </div>
    </>
  );
}

interface JobRowProps {
  entry: BuildJobResponse;
  killingJobId: string | null;
  mode: JobViewMode;
  onDelete: (job: BuildJobResponse) => void;
  onKill: (job: BuildJobResponse) => void;
  serverHardware: ServerHardwareResponse | null;
  usage: JobResourceUsageSample | null;
}

function MobileJobCard({
  entry,
  killingJobId,
  mode,
  onDelete,
  onKill,
  serverHardware,
  usage,
}: JobRowProps) {
  const isLive = isLiveJob(entry);
  return (
    <article className="border-2 border-zinc-700 bg-black p-4">
      <div className="flex items-start justify-between gap-3">
        <div className="min-w-0">
          <JobLink entry={entry} />
          <div className="mt-1 break-all font-mono text-xs text-zinc-600">
            {entry.job.id}
          </div>
        </div>
        <JobStatusBadge entry={entry} />
      </div>
      <div className="mt-3 space-y-2 font-mono text-xs text-zinc-400">
        <JobFact label="Target">
          <Badge variant="ghost">{entry.job.mock_chroot}</Badge>
        </JobFact>
        <JobFact label="Revision">{entry.job.revision}</JobFact>
        <JobFact label="Duration">
          {formatDurationBetween(entry.job.created_at, entry.job.finished_at)}
        </JobFact>
        <JobFact label="Created">{formatDateTime(entry.job.created_at)}</JobFact>
        {mode === "active" && isLive && (
          <div className="space-y-3 border-2 border-zinc-700 bg-zinc-950 px-3 py-3">
            <JobUsageBar usage={usage} serverHardware={serverHardware} />
          </div>
        )}
      </div>
      <JobActions
        entry={entry}
        isLive={isLive}
        killingJobId={killingJobId}
        mode={mode}
        onDelete={onDelete}
        onKill={onKill}
        mobile
      />
    </article>
  );
}

function DesktopJobRow({
  entry,
  index,
  killingJobId,
  mode,
  onDelete,
  onKill,
  serverHardware,
  usage,
}: JobRowProps & { index: number }) {
  const isLive = isLiveJob(entry);
  return (
    <tr
      className={`border-b-2 border-[var(--theme-border)] transition-all hover:bg-zinc-950 ${
        index % 2 === 0 ? "bg-black" : "bg-zinc-950/40"
      }`}
    >
      <td className="px-5 py-4">
        <div className="min-w-[160px]">
          <JobLink entry={entry} />
          <div className="mt-1 max-w-[200px] truncate font-mono text-xs text-zinc-600">
            {entry.job.id}
          </div>
        </div>
      </td>
      <td className="px-5 py-4">
        <Badge variant="ghost">{entry.job.mock_chroot}</Badge>
      </td>
      <td className="px-5 py-4">
        <div className="max-w-[300px] truncate font-mono text-sm text-zinc-300">
          {entry.job.revision}
        </div>
      </td>
      <td className="px-5 py-4">
        <JobStatusBadge entry={entry} />
      </td>
      <td className="px-5 py-4 font-mono text-sm text-zinc-400">
        {formatDurationBetween(entry.job.created_at, entry.job.finished_at)}
      </td>
      <td className="px-5 py-4 font-mono text-sm text-zinc-400">
        {formatDateTime(entry.job.created_at)}
      </td>
      {mode === "active" && (
        <td className="px-5 py-4">
          {isLive && usage ? (
            <div className="min-w-[320px] border-2 border-zinc-700 bg-zinc-950 p-3">
              <JobUsageBar usage={usage} serverHardware={serverHardware} />
            </div>
          ) : (
            <span className="font-mono text-xs text-zinc-600">-</span>
          )}
        </td>
      )}
      <td className="px-5 py-4">
        <JobActions
          entry={entry}
          isLive={isLive}
          killingJobId={killingJobId}
          mode={mode}
          onDelete={onDelete}
          onKill={onKill}
        />
      </td>
    </tr>
  );
}

function JobActions({
  entry,
  isLive,
  killingJobId,
  mobile = false,
  mode,
  onDelete,
  onKill,
}: {
  entry: BuildJobResponse;
  isLive: boolean;
  killingJobId: string | null;
  mobile?: boolean;
  mode: JobViewMode;
  onDelete: (job: BuildJobResponse) => void;
  onKill: (job: BuildJobResponse) => void;
}) {
  const widthClass = mobile ? "w-full sm:w-auto" : "";
  return (
    <div className={mobile ? "mt-4 grid gap-2 sm:flex sm:flex-wrap" : "flex flex-wrap gap-2"}>
      <Button
        variant="ghost"
        size="sm"
        className={widthClass}
        onClick={() => {
          window.location.href = `/jobs/view/?id=${encodeURIComponent(entry.job.id)}`;
        }}
      >
        Open
      </Button>
      {mode === "active" && isLive && (
        <Button
          variant="warning"
          size="sm"
          className={widthClass}
          onClick={() => onKill(entry)}
          disabled={killingJobId === entry.job.id}
        >
          <FaIcon icon={faStop} />
          Kill Active
        </Button>
      )}
      {mode !== "active" && (
        <Button
          variant="danger"
          size="sm"
          className={widthClass}
          onClick={() => onDelete(entry)}
          disabled={isLive}
        >
          <FaIcon icon={faTrash} />
          Delete
        </Button>
      )}
    </div>
  );
}

function JobFact({
  children,
  label,
}: {
  children: React.ReactNode;
  label: string;
}) {
  return (
    <div className="break-all">
      <span className="text-zinc-500">{label}:</span> {children}
    </div>
  );
}

function JobLink({ entry }: { entry: BuildJobResponse }) {
  return (
    <a
      href={`/jobs/view/?id=${encodeURIComponent(entry.job.id)}`}
      className="font-display font-bold text-white transition hover:text-[var(--theme-accent-lime)]"
    >
      {entry.job.package_name}
    </a>
  );
}

function JobStatusBadge({ entry }: { entry: BuildJobResponse }) {
  return (
    <Badge variant={getStatusVariant(entry.job.status)} pulse={isLiveJob(entry)}>
      {entry.job.status}
    </Badge>
  );
}

function isLiveJob(entry: BuildJobResponse): boolean {
  return entry.job.status === "pending" || entry.job.status === "running";
}

function getStatusVariant(status: string) {
  if (status === "succeeded") return "success";
  if (status === "failed" || status === "timed_out") return "error";
  if (status === "running") return "lime";
  if (status === "pending") return "warning";
  return "default";
}
