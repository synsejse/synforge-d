import type {
  BrowseRepositoryProgressResponse,
  BrowseRepositoryRequest,
  BrowseRepositoryResponse,
  CreatePackageRequest,
  MockChrootListResponse,
  PackageActionResponse,
  PackageActionTargetResult,
  PackageBuildHistoryResponse,
  PackageListResponse,
  PackageResponse,
  RefreshAllPackagesProgressResponse,
  RefreshAllPackagesResponse,
  RepoInventoryResponse,
  RepoSummaryResponse,
  UpdatePackageRequest,
} from "../types";
import { BaseApiClient } from "./client";

export class PackageApiClient extends BaseApiClient {
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

  async getBrowseRepositoryProgress(): Promise<BrowseRepositoryProgressResponse> {
    return this.request("GET", "/api/v1/repositories/browse/progress");
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
    return this.request("DELETE", `/api/v1/packages/${encodeURIComponent(name)}`);
  }

  async getPackageBuilds(
    name: string,
    limit = 12,
    offset = 0,
  ): Promise<PackageBuildHistoryResponse> {
    const params = new URLSearchParams({
      limit: String(limit),
      offset: String(offset),
    });
    return this.request(
      "GET",
      `/api/v1/packages/${encodeURIComponent(name)}/builds?${params.toString()}`,
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

  async rebuildPackage(name: string): Promise<PackageActionResponse> {
    return this.request(
      "POST",
      `/api/v1/packages/${encodeURIComponent(name)}/rebuild`,
      {},
    );
  }

  async refreshPackage(name: string): Promise<PackageActionResponse> {
    return this.request(
      "POST",
      `/api/v1/packages/${encodeURIComponent(name)}/refresh`,
      {},
    );
  }

  async refreshAllPackages(): Promise<RefreshAllPackagesResponse> {
    return this.request("POST", "/api/v1/packages/refresh-all", {});
  }

  async getRefreshAllPackagesProgress(): Promise<RefreshAllPackagesProgressResponse> {
    return this.request("GET", "/api/v1/packages/refresh-all/progress");
  }

  async rebuildPackageTarget(
    name: string,
    mockChroot: string,
  ): Promise<PackageActionTargetResult> {
    return this.request(
      "POST",
      `/api/v1/packages/${encodeURIComponent(name)}/targets/${encodeURIComponent(mockChroot)}/rebuild`,
      {},
    );
  }

  async refreshPackageTarget(
    name: string,
    mockChroot: string,
  ): Promise<PackageActionTargetResult> {
    return this.request(
      "POST",
      `/api/v1/packages/${encodeURIComponent(name)}/targets/${encodeURIComponent(mockChroot)}/refresh`,
      {},
    );
  }
}
