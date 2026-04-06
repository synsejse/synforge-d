import type {
  ApiError,
  BuildArtifact,
  BuildJobListResponse,
  BuildJobResponse,
  JobArtifactListResponse,
  JobArtifactMetaResponse,
  LogChunkResponse,
  LogManifestResponse,
  LogMetaResponse,
  PruneJobsResponse,
} from "../types";
import { API_BASE, ApiClientError } from "./client";
import { PackageApiClient } from "./packages";

interface GetJobLogChunkOptions {
  cursor?: number;
  offset?: number;
  limit?: number;
  source: string;
}

export class JobApiClient extends PackageApiClient {
  async listCompletedJobs(
    options: {
      limit?: number;
      offset?: number;
      status?: string;
      packageName?: string;
      mockChroot?: string;
    } = {},
  ): Promise<BuildJobListResponse> {
    const params = new URLSearchParams();
    if (options.limit !== undefined) {
      params.set("limit", String(options.limit));
    }
    if (options.offset !== undefined) {
      params.set("offset", String(options.offset));
    }
    if (options.status && options.status !== "all") {
      params.set("status", options.status);
    }
    if (options.packageName?.trim()) {
      params.set("package_name", options.packageName.trim());
    }
    if (options.mockChroot?.trim()) {
      params.set("mock_chroot", options.mockChroot.trim());
    }
    return this.request(
      "GET",
      `/api/v1/jobs/completed${params.toString() ? `?${params.toString()}` : ""}`,
    );
  }

  async listActiveJobs(
    options: {
      limit?: number;
      offset?: number;
      packageName?: string;
      mockChroot?: string;
    } = {},
  ): Promise<BuildJobListResponse> {
    const params = new URLSearchParams();
    if (options.limit !== undefined) {
      params.set("limit", String(options.limit));
    }
    if (options.offset !== undefined) {
      params.set("offset", String(options.offset));
    }
    if (options.packageName?.trim()) {
      params.set("package_name", options.packageName.trim());
    }
    if (options.mockChroot?.trim()) {
      params.set("mock_chroot", options.mockChroot.trim());
    }
    return this.request(
      "GET",
      `/api/v1/jobs/active${params.toString() ? `?${params.toString()}` : ""}`,
    );
  }

  async getJob(id: string): Promise<BuildJobResponse> {
    return this.request("GET", `/api/v1/jobs/${encodeURIComponent(id)}`);
  }

  async listJobArtifacts(id: string): Promise<JobArtifactListResponse> {
    return this.request("GET", `/api/v1/jobs/${encodeURIComponent(id)}/artifacts`);
  }

  async getJobArtifactMeta(
    id: string,
    file: string,
  ): Promise<JobArtifactMetaResponse> {
    return this.request(
      "GET",
      `/api/v1/jobs/${encodeURIComponent(id)}/artifacts/${file
        .split("/")
        .map(encodeURIComponent)
        .join("/")}/meta`,
    );
  }

  async getJobLogManifest(id: string): Promise<LogManifestResponse> {
    return this.request("GET", `/api/v1/jobs/${encodeURIComponent(id)}/logs`);
  }

  async getJobLogMeta(id: string, source: string): Promise<LogMetaResponse> {
    return this.request(
      "GET",
      `/api/v1/jobs/${encodeURIComponent(id)}/logs/${encodeURIComponent(source)}/meta`,
    );
  }

  async getJobLogChunk(
    id: string,
    options: GetJobLogChunkOptions,
  ): Promise<LogChunkResponse> {
    const params = new URLSearchParams({
      limit: String(options.limit ?? 65536),
    });
    if (options.cursor !== undefined) {
      params.set("cursor", String(options.cursor));
    }
    if (options.offset !== undefined) {
      params.set("offset", String(options.offset));
    }
    return this.request(
      "GET",
      `/api/v1/jobs/${encodeURIComponent(id)}/logs/${encodeURIComponent(options.source)}/stream?${params.toString()}`,
    );
  }

  async downloadJobLog(id: string, source: string): Promise<void> {
    const path = `/api/v1/jobs/${encodeURIComponent(id)}/logs/${encodeURIComponent(source)}/stream`;
    const res = await fetch(`${API_BASE}${path}`, {
      method: "GET",
      headers: this.authHeaders(false),
      credentials: "include",
    });

    if (!res.ok) {
      const error: ApiError = await res.json().catch(() => ({
        code: "internal_error",
        message: res.statusText,
      }));
      throw new ApiClientError(res.status, error);
    }

    let fullLog = "";
    let cursor = 0;
    let complete = false;

    while (!complete) {
      const chunk = await this.getJobLogChunk(id, {
        cursor,
        limit: 1024 * 1024,
        source,
      });
      fullLog += chunk.contents;
      cursor = chunk.cursor;
      complete = chunk.complete;
    }

    const blob = new Blob([fullLog], { type: "text/plain;charset=utf-8" });
    const objectUrl = window.URL.createObjectURL(blob);
    const fileName = source;
    const anchor = document.createElement("a");
    anchor.href = objectUrl;
    anchor.download = fileName;
    document.body.appendChild(anchor);
    anchor.click();
    anchor.remove();
    window.URL.revokeObjectURL(objectUrl);
  }

  async deleteJob(id: string): Promise<BuildJobResponse> {
    return this.request("DELETE", `/api/v1/jobs/${encodeURIComponent(id)}`);
  }

  async pruneFailedJobs(): Promise<PruneJobsResponse> {
    return this.request("POST", "/api/v1/jobs/prune-failed", {});
  }

  async downloadJobArtifact(id: string, artifact: BuildArtifact): Promise<void> {
    const path = `/api/v1/jobs/${encodeURIComponent(id)}/artifacts/${artifact.file
      .split("/")
      .map((segment) => encodeURIComponent(segment))
      .join("/")}`;

    const res = await fetch(`${API_BASE}${path}`, {
      method: "GET",
      headers: this.authHeaders(false),
      credentials: "include",
    });

    if (!res.ok) {
      const error: ApiError = await res.json().catch(() => ({
        code: "internal_error",
        message: res.statusText,
      }));
      throw new ApiClientError(res.status, error);
    }

    const blob = await res.blob();
    const objectUrl = window.URL.createObjectURL(blob);
    const disposition = res.headers.get("content-disposition") || "";
    const match = disposition.match(/filename="([^"]+)"/i);
    const fallbackName = artifact.file.split("/").at(-1) || "artifact.bin";
    const fileName = match?.[1] || fallbackName;
    const anchor = document.createElement("a");
    anchor.href = objectUrl;
    anchor.download = fileName;
    document.body.appendChild(anchor);
    anchor.click();
    anchor.remove();
    window.URL.revokeObjectURL(objectUrl);
  }
}
