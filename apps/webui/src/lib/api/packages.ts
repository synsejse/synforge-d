import type {
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
  RepoSetupInfoResponse,
  RepoSummaryResponse,
  ServerHardwareResponse,
  UpdatePackageRequest,
} from "../types";
import { request } from "./client";

export function listPackages(): Promise<PackageListResponse> {
  return listPackagesPage(50, 0);
}

export function listPackagesPage(
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
  return request("GET", `/api/v1/packages?${params.toString()}`);
}

export function getPackage(name: string): Promise<PackageResponse> {
  return request("GET", `/api/v1/packages/${encodeURIComponent(name)}`);
}

export function createPackage(req: CreatePackageRequest): Promise<PackageResponse> {
  return request("POST", "/api/v1/packages", req);
}

export function browseRepository(
  req: BrowseRepositoryRequest,
): Promise<BrowseRepositoryResponse> {
  return request("POST", "/api/v1/repositories/browse", req);
}

export function listMockChroots(): Promise<MockChrootListResponse> {
  return request("GET", "/api/v1/mock/chroots");
}

export function updatePackage(
  name: string,
  req: UpdatePackageRequest,
): Promise<PackageResponse> {
  return request("PUT", `/api/v1/packages/${encodeURIComponent(name)}`, req);
}

export function deletePackage(name: string): Promise<void> {
  return request("DELETE", `/api/v1/packages/${encodeURIComponent(name)}`);
}

export function getPackageBuilds(
  name: string,
  limit = 12,
  offset = 0,
  options: { includeDeleted?: boolean } = {},
): Promise<PackageBuildHistoryResponse> {
  const params = new URLSearchParams({
    limit: String(limit),
    offset: String(offset),
  });
  if (options.includeDeleted) {
    params.set("include_deleted", "true");
  }
  return request(
    "GET",
    `/api/v1/packages/${encodeURIComponent(name)}/builds?${params.toString()}`,
  );
}

export function getRepoInventory(
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
  return request("GET", `/api/v1/repo/files?${params.toString()}`);
}

export function getRepoSummary(): Promise<RepoSummaryResponse> {
  return request("GET", "/api/v1/repo/summary");
}

export function getRepoSetupInfo(): Promise<RepoSetupInfoResponse> {
  return request("GET", "/api/v1/repo/setup-info");
}

export function getServerHardware(): Promise<ServerHardwareResponse> {
  return request("GET", "/api/v1/system/hardware");
}

export function rebuildPackage(name: string): Promise<PackageActionResponse> {
  return request("POST", `/api/v1/packages/${encodeURIComponent(name)}/rebuild`, {});
}

export function refreshPackage(name: string): Promise<PackageActionResponse> {
  return request("POST", `/api/v1/packages/${encodeURIComponent(name)}/refresh`, {});
}

export function refreshAllPackages(): Promise<RefreshAllPackagesResponse> {
  return request("POST", "/api/v1/packages/refresh-all", {});
}

export function getRefreshAllPackagesProgress(): Promise<RefreshAllPackagesProgressResponse> {
  return request("GET", "/api/v1/packages/refresh-all/progress");
}

export function rebuildPackageTarget(
  name: string,
  mockChroot: string,
): Promise<PackageActionTargetResult> {
  return request(
    "POST",
    `/api/v1/packages/${encodeURIComponent(name)}/targets/${encodeURIComponent(mockChroot)}/rebuild`,
    {},
  );
}

export function refreshPackageTarget(
  name: string,
  mockChroot: string,
): Promise<PackageActionTargetResult> {
  return request(
    "POST",
    `/api/v1/packages/${encodeURIComponent(name)}/targets/${encodeURIComponent(mockChroot)}/refresh`,
    {},
  );
}
