import type {
  BuildArtifact,
  PackageListResponse,
  PackageResponse,
  CreatePackageRequest,
  UpdatePackageRequest,
  BrowseRepositoryRequest,
  BrowseRepositoryResponse,
  MockChrootListResponse,
  PackageBuildHistoryResponse,
  PackageRepoFilesResponse,
  RepoInventoryResponse,
  RepoSummaryResponse,
  BuildJobListResponse,
  BuildJobResponse,
  LogChunkResponse,
  LogManifestResponse,
  LogMetaResponse,
  EffectiveConfigResponse,
  ConfigSchemaResponse,
  UpdateRuntimeSettingsRequest,
  PruneJobsResponse,
  ApiError,
  SessionLoginRequest,
  SessionResponse,
  UserListResponse,
  UserResponse,
  CreateUserRequest as CreateUserPayload,
  UpdateUserRequest as UpdateUserPayload,
  ChangePasswordRequest,
  UserMetricsResponse,
} from "./types";

const API_BASE = import.meta.env.PUBLIC_API_URL || "";

interface GetJobLogChunkOptions {
  cursor?: number;
  offset?: number;
  limit?: number;
  source?: string;
}

class ApiClient {
  private authHeaders(contentType = true): Record<string, string> {
    const headers: Record<string, string> = {};
    if (contentType) {
      headers["Content-Type"] = "application/json";
    }
    return headers;
  }

  private async request<T>(
    method: string,
    path: string,
    body?: unknown,
  ): Promise<T> {
    const headers = this.authHeaders();

    const res = await fetch(`${API_BASE}${path}`, {
      method,
      headers,
      credentials: "include",
      body: body ? JSON.stringify(body) : undefined,
    });

    if (!res.ok) {
      const error: ApiError = await res.json().catch(() => ({
        code: "internal_error",
        message: res.statusText,
      }));
      if (res.status === 401 && typeof window !== "undefined") {
        window.dispatchEvent(
          new CustomEvent("synforge:auth-required", {
            detail: {
              path,
              error,
            },
          }),
        );
      }
      throw new ApiClientError(res.status, error);
    }

    if (res.status === 204) {
      return undefined as T;
    }

    return res.json();
  }

  async getSession(): Promise<SessionResponse> {
    return this.request("GET", "/api/v1/session");
  }

  async login(req: SessionLoginRequest): Promise<SessionResponse> {
    return this.request("POST", "/api/v1/session/login", req);
  }

  async logout(): Promise<void> {
    return this.request("POST", "/api/v1/session/logout", {});
  }

  // Packages
  async listPackages(): Promise<PackageListResponse> {
    return this.listPackagesPage(50, 0);
  }

  async listPackagesPage(
    limit = 50,
    offset = 0,
    options: { search?: string; enabled?: boolean | "all" } = {},
  ): Promise<PackageListResponse> {
    const params = new URLSearchParams({
      limit: String(limit),
      offset: String(offset),
    });
    if (options.search?.trim()) {
      params.set("search", options.search.trim());
    }
    if (options.enabled !== undefined && options.enabled !== "all") {
      params.set("enabled", String(options.enabled));
    }
    return this.request("GET", `/api/v1/packages?${params.toString()}`);
  }

  async getPackage(name: string): Promise<PackageResponse> {
    return this.request("GET", `/api/v1/packages/${encodeURIComponent(name)}`);
  }

  async createPackage(req: CreatePackageRequest): Promise<PackageResponse> {
    return this.request("POST", "/api/v1/packages", req);
  }

  async browseRepository(
    req: BrowseRepositoryRequest,
  ): Promise<BrowseRepositoryResponse> {
    return this.request("POST", "/api/v1/repositories/browse", req);
  }

  async listMockChroots(): Promise<MockChrootListResponse> {
    return this.request("GET", "/api/v1/mock/chroots");
  }

  async updatePackage(
    name: string,
    req: UpdatePackageRequest,
  ): Promise<PackageResponse> {
    return this.request(
      "PUT",
      `/api/v1/packages/${encodeURIComponent(name)}`,
      req,
    );
  }

  async deletePackage(name: string): Promise<void> {
    return this.request(
      "DELETE",
      `/api/v1/packages/${encodeURIComponent(name)}`,
    );
  }

  async getPackageBuilds(name: string): Promise<PackageBuildHistoryResponse> {
    return this.request(
      "GET",
      `/api/v1/packages/${encodeURIComponent(name)}/builds`,
    );
  }

  async getPackageRepoFiles(name: string): Promise<PackageRepoFilesResponse> {
    return this.request(
      "GET",
      `/api/v1/packages/${encodeURIComponent(name)}/repo-files`,
    );
  }

  async getRepoInventory(
    limit = 50,
    offset = 0,
    options: {
      packageName?: string;
      mockChroot?: string;
      kind?: "rpm" | "srpm" | "log" | "other" | "all";
    } = {},
  ): Promise<RepoInventoryResponse> {
    const params = new URLSearchParams({
      limit: String(limit),
      offset: String(offset),
    });
    if (options.packageName?.trim()) {
      params.set("package_name", options.packageName.trim());
    }
    if (options.mockChroot?.trim()) {
      params.set("mock_chroot", options.mockChroot.trim());
    }
    if (options.kind && options.kind !== "all") {
      params.set("kind", options.kind);
    }
    return this.request("GET", `/api/v1/repo/files?${params.toString()}`);
  }

  async getRepoSummary(): Promise<RepoSummaryResponse> {
    return this.request("GET", "/api/v1/repo/summary");
  }

  async rebuildPackage(name: string): Promise<BuildJobResponse> {
    return this.request(
      "POST",
      `/api/v1/packages/${encodeURIComponent(name)}/rebuild`,
      {},
    );
  }

  async refreshPackage(name: string): Promise<BuildJobResponse> {
    return this.request(
      "POST",
      `/api/v1/packages/${encodeURIComponent(name)}/refresh`,
      {},
    );
  }

  // Jobs
  async listJobs(
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
      `/api/v1/jobs${params.toString() ? `?${params.toString()}` : ""}`,
    );
  }

  async getJob(id: string): Promise<BuildJobResponse> {
    return this.request("GET", `/api/v1/jobs/${encodeURIComponent(id)}`);
  }

  async getJobLogManifest(id: string): Promise<LogManifestResponse> {
    return this.request("GET", `/api/v1/jobs/${encodeURIComponent(id)}/logs`);
  }

  async getJobLogMeta(id: string, source?: string): Promise<LogMetaResponse> {
    const params = new URLSearchParams();
    if (source) {
      params.set("source", source);
    }
    return this.request(
      "GET",
      `/api/v1/jobs/${encodeURIComponent(id)}/logs/meta${params.toString() ? `?${params.toString()}` : ""}`,
    );
  }

  async getJobLogChunk(
    id: string,
    options: GetJobLogChunkOptions = {},
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
    if (options.source) {
      params.set("source", options.source);
    }
    return this.request(
      "GET",
      `/api/v1/jobs/${encodeURIComponent(id)}/logs/stream?${params.toString()}`,
    );
  }

  async downloadJobLog(id: string, source?: string): Promise<void> {
    const params = new URLSearchParams();
    if (source) {
      params.set("source", source);
    }
    const path = `/api/v1/jobs/${encodeURIComponent(id)}/logs/stream${params.toString() ? `?${params.toString()}` : ""}`;

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

    // Download the full log by fetching chunks until complete
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

    // Create blob and download
    const blob = new Blob([fullLog], { type: "text/plain;charset=utf-8" });
    const objectUrl = window.URL.createObjectURL(blob);
    const fileName = source || "build.log";
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

  async downloadJobArtifact(
    id: string,
    artifact: BuildArtifact,
  ): Promise<void> {
    const path = `/api/v1/jobs/${encodeURIComponent(id)}/artifacts/${artifact.relative_repo_path
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
      if (res.status === 401 && typeof window !== "undefined") {
        window.dispatchEvent(
          new CustomEvent("synforge:auth-required", {
            detail: {
              path,
              error,
            },
          }),
        );
      }
      throw new ApiClientError(res.status, error);
    }

    const blob = await res.blob();
    const objectUrl = window.URL.createObjectURL(blob);
    const disposition = res.headers.get("content-disposition") || "";
    const match = disposition.match(/filename="([^"]+)"/i);
    const fallbackName =
      artifact.relative_repo_path.split("/").at(-1) || "artifact.bin";
    const fileName = match?.[1] || fallbackName;
    const anchor = document.createElement("a");
    anchor.href = objectUrl;
    anchor.download = fileName;
    document.body.appendChild(anchor);
    anchor.click();
    anchor.remove();
    window.URL.revokeObjectURL(objectUrl);
  }

  // Config
  async getConfig(): Promise<EffectiveConfigResponse> {
    return this.request("GET", "/api/v1/config/effective");
  }

  async getConfigSchema(): Promise<ConfigSchemaResponse> {
    return this.request("GET", "/api/v1/config/schema");
  }

  async updateRuntimeSettings(
    req: UpdateRuntimeSettingsRequest,
  ): Promise<EffectiveConfigResponse> {
    return this.request("POST", "/api/v1/config/runtime", req);
  }

  // Users
  async listUsers(): Promise<UserListResponse> {
    return this.request("GET", "/api/v1/users");
  }

  async createUser(req: CreateUserPayload): Promise<UserResponse> {
    return this.request("POST", "/api/v1/users", req);
  }

  async updateUser(id: string, req: UpdateUserPayload): Promise<UserResponse> {
    return this.request("PUT", `/api/v1/users/${encodeURIComponent(id)}`, req);
  }

  async changeUserPassword(
    id: string,
    req: ChangePasswordRequest,
  ): Promise<void> {
    return this.request(
      "POST",
      `/api/v1/users/${encodeURIComponent(id)}/password`,
      req,
    );
  }

  async deleteUser(id: string): Promise<UserResponse> {
    return this.request("DELETE", `/api/v1/users/${encodeURIComponent(id)}`);
  }

  async getUserMetrics(id: string): Promise<UserMetricsResponse> {
    return this.request("GET", `/api/v1/users/${encodeURIComponent(id)}`);
  }
}

export class ApiClientError extends Error {
  status: number;
  error: ApiError;

  constructor(status: number, error: ApiError) {
    super(error.message);
    this.name = "ApiClientError";
    this.status = status;
    this.error = error;
  }
}

export const api = new ApiClient();
export default api;
