import { useMemo } from "react";
import {
  faStop,
  faTrash,
} from "@fortawesome/free-solid-svg-icons";
import { Link, useNavigate } from "@tanstack/react-router";
import type {
  BuildJobResponse,
  JobResourceUsageSample,
  ServerHardwareResponse,
} from "../../../lib/types";
import { formatDateTime, formatDurationBetween } from "../../../lib/datetime";
import Badge from "../../../components/ui/badge";
import Button from "../../../components/ui/button";
import DataTable, { type DataTableColumn } from "../../../components/ui/data-table";
import FaIcon from "../../../components/ui/fa-icon";
import type { JobViewMode } from "../types";
import JobUsageBar from "./job-usage-bar";

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
  const columns = useMemo<DataTableColumn<BuildJobResponse>[]>(() => {
    const cols: DataTableColumn<BuildJobResponse>[] = [
      {
        key: "package",
        header: "Package",
        mobile: "title",
        cell: (entry) => (
          <div className="min-w-[160px]">
            <Link
              to="/jobs/view"
              search={{ id: entry.job.id }}
              className="font-display font-bold text-white transition hover:text-accent-lime"
            >
              {entry.job.package_name}
            </Link>
            <div className="mt-1 max-w-[200px] truncate font-mono text-xs text-soft">
              {entry.job.id}
            </div>
          </div>
        ),
      },
      {
        key: "target",
        header: "Target",
        mobile: "field",
        cell: (entry) => <Badge variant="ghost">{entry.job.mock_chroot}</Badge>,
      },
      {
        key: "revision",
        header: "Revision",
        mobile: "field",
        cell: (entry) => (
          <div className="max-w-[300px] truncate font-mono text-sm text-muted md:max-w-[300px]">
            {entry.job.revision}
          </div>
        ),
      },
      {
        key: "status",
        header: "Status",
        mobile: "badge",
        cell: (entry) => (
          <Badge
            variant={getStatusVariant(entry.job.status)}
            pulse={isLiveJob(entry)}
          >
            {entry.job.status}
          </Badge>
        ),
      },
      {
        key: "duration",
        header: "Duration",
        mobile: "field",
        className: "font-mono text-sm text-muted",
        cell: (entry) =>
          formatDurationBetween(entry.job.created_at, entry.job.finished_at),
      },
      {
        key: "created",
        header: "Created",
        mobile: "field",
        className: "font-mono text-sm text-muted",
        cell: (entry) => formatDateTime(entry.job.created_at),
      },
    ];

    if (mode === "active") {
      cols.push({
        key: "usage",
        header: "Live Usage",
        // Surfaced in the mobile card as a richer block via cardFooter; on
        // desktop it owns its own column.
        mobile: "hidden",
        cell: (entry) => {
          const usage = usageByJob[entry.job.id] ?? null;
          if (!isLiveJob(entry) || !usage) {
            return <span className="font-mono text-xs text-soft">-</span>;
          }
          return (
            <div className="min-w-[320px] border-2 border-edge-strong bg-surface-alt p-3">
              <JobUsageBar usage={usage} serverHardware={serverHardware} />
            </div>
          );
        },
      });
    }

    cols.push({
      key: "actions",
      header: "Actions",
      mobile: "hidden",
      cell: (entry) => (
        <JobActions
          entry={entry}
          isLive={isLiveJob(entry)}
          killingJobId={killingJobId}
          mode={mode}
          onDelete={onDelete}
          onKill={onKill}
        />
      ),
    });

    return cols;
  }, [killingJobId, mode, onDelete, onKill, serverHardware, usageByJob]);

  const cardFooter = (entry: BuildJobResponse) => {
    const usage = usageByJob[entry.job.id] ?? null;
    const live = isLiveJob(entry);
    return (
      <>
        {mode === "active" && live ? (
          <div className="mt-3 space-y-3 border-2 border-edge-strong bg-surface-alt px-3 py-3">
            <JobUsageBar usage={usage} serverHardware={serverHardware} />
          </div>
        ) : null}
        <JobActions
          entry={entry}
          isLive={live}
          killingJobId={killingJobId}
          mode={mode}
          onDelete={onDelete}
          onKill={onKill}
          mobile
        />
      </>
    );
  };

  return (
    <div className="p-4">
      <DataTable
        columns={columns}
        rows={jobs}
        rowKey={(entry) => entry.job.id}
        rowClassName={(entry) =>
          isLiveJob(entry)
            ? "synforge-row-live border-l-4 !border-l-accent-lime bg-surface-alt/40"
            : ""
        }
        empty={{
          title: mode === "active" ? "No active jobs" : "No jobs found",
          description:
            mode === "active"
              ? "Nothing is currently pending or running."
              : "Adjust filters or queue new builds to see results here.",
        }}
        cardFooter={cardFooter}
      />
    </div>
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
  const navigate = useNavigate();
  const widthClass = mobile ? "w-full sm:w-auto" : "";
  return (
    <div
      className={
        mobile ? "mt-4 grid gap-2 sm:flex sm:flex-wrap" : "flex flex-wrap gap-2"
      }
    >
      <Button
        variant="ghost"
        size="sm"
        className={widthClass}
        onClick={() =>
          navigate({ to: "/jobs/view", search: { id: entry.job.id } })
        }
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

function isLiveJob(entry: BuildJobResponse): boolean {
  return entry.job.status === "pending" || entry.job.status === "running";
}

function getStatusVariant(status: string) {
  if (status === "succeeded") return "success" as const;
  if (status === "failed" || status === "timed_out") return "error" as const;
  if (status === "running") return "lime" as const;
  if (status === "pending") return "warning" as const;
  return "default" as const;
}
