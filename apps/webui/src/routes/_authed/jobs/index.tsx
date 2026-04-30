import { createFileRoute } from "@tanstack/react-router";
import JobListPage from "../../../features/jobs/job-list-page";
import {
  isHistoryBuildStatus,
  type HistoryBuildStatus,
} from "../../../lib/job-status";
import type { JobViewMode } from "../../../features/jobs/types";

export interface JobsListSearch {
  mode?: JobViewMode;
  filter?: "all" | HistoryBuildStatus;
  offset?: number;
  packageFilter?: string;
  targetFilter?: string;
}

function readMode(value: unknown): JobViewMode {
  return value === "active" ? "active" : "history";
}

function readFilter(value: unknown): "all" | HistoryBuildStatus {
  if (typeof value !== "string" || value === "all") return "all";
  return isHistoryBuildStatus(value) ? value : "all";
}

export const Route = createFileRoute("/_authed/jobs/")({
  validateSearch: (search: Record<string, unknown>): JobsListSearch => ({
    mode: readMode(search.mode),
    filter: readFilter(search.filter ?? search.status),
    offset:
      typeof search.offset === "number" ? search.offset : Number(search.offset ?? 0) || 0,
    packageFilter:
      typeof search.packageFilter === "string"
        ? search.packageFilter
        : typeof search.package === "string"
          ? search.package
          : "",
    targetFilter:
      typeof search.targetFilter === "string"
        ? search.targetFilter
        : typeof search.target === "string"
          ? search.target
          : "",
  }),
  component: JobListPage,
});
