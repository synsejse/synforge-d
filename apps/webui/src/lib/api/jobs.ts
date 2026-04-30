import type {
  BuildArtifact,
  BuildJobListResponse,
  BuildJobResponse,
  JobArtifactListResponse,
  JobArtifactMetaResponse,
  JobResourceUsageListResponse,
  JobResourceUsageResponse,
  LogChunkResponse,
  LogManifestResponse,
  LogMetaResponse,
  PruneJobsResponse,
} from "../types";
import { downloadStream, request } from "./client";

interface GetJobLogChunkOptions {
  cursor?: number;
  offset?: number;
  limit?: number;
  source: string;
}

interface ListJobsOptions {
  limit?: number;
  offset?: number;
  status?: string;
  packageName?: string;
  mockChroot?: string;
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
  return request("GET", "/api/v1/jobs/usage");
}

export function getJobUsage(id: string): Promise<JobResourceUsageResponse> {
  return request("GET", `/api/v1/jobs/${encodeURIComponent(id)}/usage`);
}

export function listJobArtifacts(id: string): Promise<JobArtifactListResponse> {
  return request("GET", `/api/v1/jobs/${encodeURIComponent(id)}/artifacts`);
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

export function getJobLogManifest(id: string): Promise<LogManifestResponse> {
  return request("GET", `/api/v1/jobs/${encodeURIComponent(id)}/logs`);
}

export function getJobLogMeta(id: string, source: string): Promise<LogMetaResponse> {
  return request(
    "GET",
    `/api/v1/jobs/${encodeURIComponent(id)}/logs/${encodeURIComponent(source)}/meta`,
  );
}

export function getJobLogChunk(
  id: string,
  options: GetJobLogChunkOptions,
): Promise<LogChunkResponse> {
  const params = new URLSearchParams({
    limit: String(options.limit ?? 65536),
  });
  if (options.cursor !== undefined) params.set("cursor", String(options.cursor));
  if (options.offset !== undefined) params.set("offset", String(options.offset));
  return request(
    "GET",
    `/api/v1/jobs/${encodeURIComponent(id)}/logs/${encodeURIComponent(options.source)}/chunks?${params.toString()}`,
  );
}

export async function downloadJobLog(id: string, source: string): Promise<void> {
  const path = `/api/v1/jobs/${encodeURIComponent(id)}/logs/${encodeURIComponent(source)}/chunks`;
  await downloadStream(path);

  let fullLog = "";
  let cursor = 0;
  let complete = false;

  while (!complete) {
    const chunk = await getJobLogChunk(id, {
      cursor,
      limit: 1024 * 1024,
      source,
    });
    fullLog += chunk.contents;
    cursor = chunk.cursor;
    complete = chunk.complete;
  }

  const blob = new Blob([fullLog], { type: "text/plain;charset=utf-8" });
  triggerDownload(blob, source);
}

export function deleteJob(id: string): Promise<BuildJobResponse> {
  return request("DELETE", `/api/v1/jobs/${encodeURIComponent(id)}`);
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
