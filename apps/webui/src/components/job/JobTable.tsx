import { faFolderOpen, faTrash } from "@fortawesome/free-solid-svg-icons";
import { formatDateTime, formatDurationBetween } from "../../lib/datetime";
import type { BuildJobResponse } from "../../lib/types";
import ActionButton from "../ui/ActionButton";
import StatusPill from "../ui/StatusPill";

interface JobTableProps {
  jobs: BuildJobResponse[];
  onDelete: (job: BuildJobResponse) => void;
}

export default function JobTable({ jobs, onDelete }: JobTableProps) {
  return (
    <div className="overflow-x-auto border border-zinc-800 bg-black">
      <table className="w-full min-w-[640px] lg:min-w-[980px]">
        <caption className="sr-only">
          Build jobs with status, target, revision, and row actions.
        </caption>
        <thead className="bg-zinc-950 text-left text-xs uppercase tracking-[0.2em] text-zinc-500">
          <tr>
            <th scope="col" className="px-4 py-3">
              Package
            </th>
            <th scope="col" className="px-4 py-3">
              Target
            </th>
            <th scope="col" className="px-4 py-3">
              Revision
            </th>
            <th scope="col" className="px-4 py-3">
              Status
            </th>
            <th scope="col" className="px-4 py-3">
              Trigger
            </th>
            <th scope="col" className="px-4 py-3">
              Duration
            </th>
            <th scope="col" className="px-4 py-3">
              Created
            </th>
            <th scope="col" className="px-4 py-3">
              Actions
            </th>
          </tr>
        </thead>
        <tbody className="divide-y divide-white/8">
          {jobs.map((entry) => {
            const isLive =
              entry.job.status === "pending" || entry.job.status === "running";
            return (
              <tr key={entry.job.id} className="hover:bg-zinc-950">
                <td className="px-4 py-3">
                  <div className="min-w-[160px] space-y-1">
                    <a
                      href={`/jobs/view/?id=${encodeURIComponent(entry.job.id)}`}
                      className="font-medium text-white transition-all duration-100 ease-linear hover:text-zinc-300"
                    >
                      {entry.job.package_name}
                    </a>
                    <div className="max-w-[180px] break-all text-xs text-zinc-500">
                      {entry.job.id}
                    </div>
                  </div>
                </td>
                <td className="px-4 py-3 text-sm font-mono text-zinc-300">
                  {entry.job.mock_chroot}
                </td>
                <td className="px-4 py-3">
                  <div className="max-w-[420px] break-all font-mono text-sm text-zinc-300">
                    {entry.job.revision}
                  </div>
                </td>
                <td className="px-4 py-3">
                  <StatusPill status={entry.job.status} />
                </td>
                <td className="px-4 py-3 text-sm text-zinc-400">
                  {entry.job.trigger}
                </td>
                <td className="px-4 py-3 text-sm text-zinc-400">
                  {formatDurationBetween(entry.job.created_at, entry.job.finished_at)}
                </td>
                <td className="px-4 py-3 text-sm text-zinc-400">
                  {formatDateTime(entry.job.created_at)}
                </td>
                <td className="px-4 py-3">
                  <div className="flex flex-wrap gap-2">
                    <ActionButton
                      href={`/jobs/view/?id=${encodeURIComponent(entry.job.id)}`}
                      icon={faFolderOpen}
                      aria-label={`Open job ${entry.job.id}`}
                    >
                      Open
                    </ActionButton>
                    <ActionButton
                      onClick={() => onDelete(entry)}
                      disabled={isLive}
                      icon={faTrash}
                      aria-label={`Delete job ${entry.job.id}`}
                      className="text-zinc-300 disabled:cursor-not-allowed disabled:opacity-50"
                    >
                      Delete
                    </ActionButton>
                  </div>
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}
