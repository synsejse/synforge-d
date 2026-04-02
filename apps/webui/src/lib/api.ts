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
  BuildJobListResponse,
  BuildJobResponse,
  LogChunkResponse,
  EffectiveConfigResponse,
  UpdateRuntimeSettingsRequest,
  ApiError,
} from "./types";

const API_BASE = import.meta.env.PUBLIC_API_URL || "";
const TOKEN_STORAGE_KEY = "synforge.bearerToken";

function loadStoredToken(): string {
  if (typeof window === "undefined") {
    return "";
  }
  return window.localStorage.getItem(TOKEN_STORAGE_KEY) || "";
}

class ApiClient {
  private token: string;

  constructor(token: string = loadStoredToken()) {
    this.token = token;
  }

  setToken(token: string) {
    this.token = token;
  }

  private authHeaders(contentType = true): Record<string, string> {
    const headers: Record<string, string> = {};
    if (contentType) {
      headers["Content-Type"] = "application/json";
    }
    if (this.token) {
      headers["Authorization"] = `Bearer ${this.token}`;
    }
    return headers;
  }

  private async request<T>(
    method: string,
    path: string,
    body?: unknown
  ): Promise<T> {
    const headers = this.authHeaders();

    const res = await fetch(`${API_BASE}${path}`, {
      method,
      headers,
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
          })
        );
      }
      throw new ApiClientError(res.status, error);
    }

    if (res.status === 204) {
      return undefined as T;
    }

    return res.json();
  }

  // Packages
  async listPackages(): Promise<PackageListResponse> {
    return this.request("GET", "/api/v1/packages");
  }

  async getPackage(name: string): Promise<PackageResponse> {
    return this.request("GET", `/api/v1/packages/${encodeURIComponent(name)}`);
  }

  async createPackage(req: CreatePackageRequest): Promise<PackageResponse> {
    return this.request("POST", "/api/v1/packages", req);
  }

  async browseRepository(
    req: BrowseRepositoryRequest
  ): Promise<BrowseRepositoryResponse> {
    return this.request("POST", "/api/v1/repositories/browse", req);
  }

  async listMockChroots(): Promise<MockChrootListResponse> {
    return this.request("GET", "/api/v1/mock/chroots");
  }

  async updatePackage(
    name: string,
    req: UpdatePackageRequest
  ): Promise<PackageResponse> {
    return this.request(
      "PUT",
      `/api/v1/packages/${encodeURIComponent(name)}`,
      req
    );
  }

  async deletePackage(name: string): Promise<void> {
    return this.request("DELETE", `/api/v1/packages/${encodeURIComponent(name)}`);
  }

  async getPackageBuilds(name: string): Promise<PackageBuildHistoryResponse> {
    return this.request("GET", `/api/v1/packages/${encodeURIComponent(name)}/builds`);
  }

  async getPackageRepoFiles(name: string): Promise<PackageRepoFilesResponse> {
    return this.request("GET", `/api/v1/packages/${encodeURIComponent(name)}/repo-files`);
  }

  async getRepoInventory(): Promise<RepoInventoryResponse> {
    return this.request("GET", "/api/v1/repo/files");
  }

  async rebuildPackage(name: string): Promise<BuildJobResponse> {
    return this.request(
      "POST",
      `/api/v1/packages/${encodeURIComponent(name)}/rebuild`,
      {}
    );
  }

  async refreshPackage(name: string): Promise<BuildJobResponse> {
    return this.request(
      "POST",
      `/api/v1/packages/${encodeURIComponent(name)}/refresh`,
      {}
    );
  }

  // Jobs
  async listJobs(): Promise<BuildJobListResponse> {
    return this.request("GET", "/api/v1/jobs");
  }

  async getJob(id: string): Promise<BuildJobResponse> {
    return this.request("GET", `/api/v1/jobs/${encodeURIComponent(id)}`);
  }

  async getJobLogChunk(
    id: string,
    cursor: number = 0,
    limit: number = 65536
  ): Promise<LogChunkResponse> {
    return this.request(
      "GET",
      `/api/v1/jobs/${encodeURIComponent(id)}/log/stream?cursor=${encodeURIComponent(String(cursor))}&limit=${encodeURIComponent(String(limit))}`
    );
  }

  async deleteJob(id: string): Promise<BuildJobResponse> {
    return this.request("DELETE", `/api/v1/jobs/${encodeURIComponent(id)}`);
  }

  async downloadJobArtifact(id: string, artifact: BuildArtifact): Promise<void> {
    const path = `/api/v1/jobs/${encodeURIComponent(id)}/artifacts/${artifact.relative_repo_path
      .split("/")
      .map((segment) => encodeURIComponent(segment))
      .join("/")}`;

    const res = await fetch(`${API_BASE}${path}`, {
      method: "GET",
      headers: this.authHeaders(false),
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
          })
        );
      }
      throw new ApiClientError(res.status, error);
    }

    const blob = await res.blob();
    const objectUrl = window.URL.createObjectURL(blob);
    const disposition = res.headers.get("content-disposition") || "";
    const match = disposition.match(/filename="([^"]+)"/i);
    const fallbackName = artifact.relative_repo_path.split("/").at(-1) || "artifact.bin";
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

  async updateRuntimeSettings(
    req: UpdateRuntimeSettingsRequest
  ): Promise<EffectiveConfigResponse> {
    return this.request("POST", "/api/v1/config/runtime", req);
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
export { TOKEN_STORAGE_KEY };
