import { faTrash } from "@fortawesome/free-solid-svg-icons";
import type { BuildJobResponse, BuildStatus } from "../../lib/types";
import { BUILD_STATUS_LABELS } from "../../lib/job-status";
import ActionButton from "../ui/ActionButton";

export type JobViewMode = "history" | "active";

const FILTERS: Array<"all" | BuildStatus> = [
  "all",
  ...(Object.keys(BUILD_STATUS_LABELS) as BuildStatus[]),
];

interface JobModeFilterBarProps {
  mode: JobViewMode;
  filter: string;
  pruning: boolean;
  jobs: BuildJobResponse[];
  onModeChange: (mode: JobViewMode) => void;
  onFilterChange: (filter: "all" | BuildStatus) => void;
  onPruneFailed: () => void;
}

export default function JobModeFilterBar({
  mode,
  filter,
  pruning,
  jobs,
  onModeChange,
  onFilterChange,
  onPruneFailed,
}: JobModeFilterBarProps) {
  return (
    <section className="flex flex-wrap gap-2 border border-zinc-800 bg-black p-4">
      {(["history", "active"] as JobViewMode[]).map((value) => (
        <button
          key={value}
          onClick={() => onModeChange(value)}
          aria-pressed={mode === value}
          className={`border px-4 py-2 text-sm transition ${
            mode === value
              ? "border-zinc-200 bg-zinc-100 text-black"
              : "border-zinc-800 bg-black text-zinc-300 hover:border-zinc-600 hover:bg-zinc-950"
          }`}
        >
          {value === "history" ? "History" : "Active"}
        </button>
      ))}
      {mode === "history" ? (
        <>
          <ActionButton
            onClick={onPruneFailed}
            disabled={
              pruning ||
              !jobs.some(
                (entry) =>
                  entry.job.status === "failed" ||
                  entry.job.status === "timed_out",
              )
            }
            icon={faTrash}
            className="text-zinc-300 disabled:cursor-not-allowed disabled:opacity-50"
          >
            {pruning ? "Pruning…" : "Prune Failed"}
          </ActionButton>
          {FILTERS.map((value) => (
            <button
              key={value}
              onClick={() => onFilterChange(value)}
              aria-pressed={filter === value}
              className={`border px-4 py-2 text-sm transition ${
                filter === value
                  ? "border-zinc-200 bg-zinc-100 text-black"
                  : "border-zinc-800 bg-black text-zinc-300 hover:border-zinc-600 hover:bg-zinc-950"
              }`}
            >
              {value}
            </button>
          ))}
        </>
      ) : null}
    </section>
  );
}
