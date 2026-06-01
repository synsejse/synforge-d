import type {
  BuildArtifact,
  BuildJobListResponse,
  BuildJobResponse,
  JobArtifactListResponse,
  JobArtifactMetaResponse,
  JobResourceUsageListResponse,
  JobResourceUsageResponse,
  PruneJobsResponse,
  TimeRange,
  TimeSeriesResponse,
} from "../types";
import { downloadStream, request } from "./client";

interface ListJobsOptions {
  limit?: number;
  offset?: number;
  status?: string;
  packageName?: string;
  mockChroot?: string;
  includeDeleted?: boolean;
}

function listJobsByScope(
  scope: "active" | "completed",
  options: ListJobsOptions = {},
): Promise<BuildJobListResponse> {
  const params = new URLSearchParams();
  params.set("scope", scope);
  if (options.limit !== undefined) params.set("limit", String(options.limit));
  if (options.offset !== undefined) params.set("offset", String(options.offset));
  if (options.status && options.status !== "all") {
    params.set("status", options.status);
  }
  if (options.packageName?.trim()) {
    params.set("package_name", options.packageName.trim());
  }
  if (options.mockChroot?.trim()) {
    params.set("mock_chroot", options.mockChroot.trim());
  }
  if (options.includeDeleted) {
    params.set("include_deleted", "true");
  }
  return request("GET", `/api/v1/jobs?${params.toString()}`);
}

export function listCompletedJobs(
  options: ListJobsOptions = {},
): Promise<BuildJobListResponse> {
  return listJobsByScope("completed", options);
}

export function listActiveJobs(
  options: Omit<ListJobsOptions, "status"> = {},
): Promise<BuildJobListResponse> {
  return listJobsByScope("active", options);
}

export function getJob(id: string): Promise<BuildJobResponse> {
  return request("GET", `/api/v1/jobs/${encodeURIComponent(id)}`);
}

export function listJobUsage(): Promise<JobResourceUsageListResponse> {
  // Live usage feed, not paginated UI; request the server max so no
  // active job's sample is truncated.
  return request("GET", "/api/v1/jobs/usage?limit=200&offset=0");
}

export function getJobUsage(id: string): Promise<JobResourceUsageResponse> {
  return request("GET", `/api/v1/jobs/${encodeURIComponent(id)}/usage`);
}

export function listJobArtifacts(
  id: string,
  limit = 50,
  offset = 0,
): Promise<JobArtifactListResponse> {
  const params = new URLSearchParams({
    limit: String(limit),
    offset: String(offset),
  });
  return request(
    "GET",
    `/api/v1/jobs/${encodeURIComponent(id)}/artifacts?${params.toString()}`,
  );
}

export function getJobArtifactMeta(
  id: string,
  file: string,
): Promise<JobArtifactMetaResponse> {
  return request(
    "GET",
    `/api/v1/jobs/${encodeURIComponent(id)}/artifacts/${file
      .split("/")
      .map(encodeURIComponent)
      .join("/")}/meta`,
  );
}

/**
 * Saves already-buffered log text (collected from the SSE stream) to a file.
 * The chunk/meta byte-range endpoints were removed in favour of the streaming
 * log API, so downloads are produced client-side from the in-memory buffer.
 */
export function downloadLogText(text: string, source: string): void {
  const blob = new Blob([text], { type: "text/plain;charset=utf-8" });
  const filename = source.split("/").pop() || source;
  triggerDownload(blob, filename);
}

export function deleteJob(id: string): Promise<void> {
  return request<void>("DELETE", `/api/v1/jobs/${encodeURIComponent(id)}`);
}

export function killJob(id: string): Promise<BuildJobResponse> {
  return request("POST", `/api/v1/jobs/${encodeURIComponent(id)}/kill`, {});
}

export function retryJob(id: string): Promise<BuildJobResponse> {
  return request("POST", `/api/v1/jobs/${encodeURIComponent(id)}/retry`, {});
}

export function pruneFailedJobs(): Promise<PruneJobsResponse> {
  return request("POST", "/api/v1/jobs/prune-failed", {});
}

export function getJobsTimeseries(
  range: TimeRange = "24h",
): Promise<TimeSeriesResponse> {
  return request("GET", `/api/v1/jobs/timeseries?range=${range}`);
}

export async function downloadJobArtifact(
  id: string,
  artifact: BuildArtifact,
): Promise<void> {
  const path = `/api/v1/jobs/${encodeURIComponent(id)}/artifacts/${artifact.file
    .split("/")
    .map((segment) => encodeURIComponent(segment))
    .join("/")}/content`;

  const res = await downloadStream(path);
  const blob = await res.blob();
  const disposition = res.headers.get("content-disposition") || "";
  const match = disposition.match(/filename="([^"]+)"/i);
  const fallbackName = artifact.file.split("/").at(-1) || "artifact.bin";
  triggerDownload(blob, match?.[1] || fallbackName);
}

function triggerDownload(blob: Blob, filename: string): void {
  const objectUrl = window.URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  anchor.href = objectUrl;
  anchor.download = filename;
  document.body.appendChild(anchor);
  anchor.click();
  anchor.remove();
  window.URL.revokeObjectURL(objectUrl);
}
