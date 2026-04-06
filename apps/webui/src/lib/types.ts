import type {
  components,
  operations,
  paths,
} from "./generated/api-schema";

export type ApiSchema = components["schemas"];
export type ApiPaths = paths;
export type ApiOperations = operations;

export type SpecSource = ApiSchema["SpecSource"];
export type BuildEnvVar = ApiSchema["BuildEnvVar"];
export type PackageDefinition = ApiSchema["PackageDefinition"];
export type PackageTargetState = ApiSchema["PackageTargetRuntimeState"];
export type PackageState = ApiSchema["PackageRuntimeState"];
export type PackageResponse = ApiSchema["PackageResponse"];
export type PackageListResponse = ApiSchema["PackageListResponse"];
export type PageInfo = ApiSchema["PageInfo"];

export type CreatePackageRequest = ApiSchema["CreatePackageRequest"];
export type UpdatePackageRequest = ApiSchema["UpdatePackageRequest"];

export type BrowseRepositoryRequest = ApiSchema["BrowseRepositoryRequest"];
export type BrowseRepositoryResponse = ApiSchema["BrowseRepositoryResponse"];

export type MockChrootListResponse = ApiSchema["MockChrootListResponse"];

export type BuildTrigger = ApiSchema["BuildTrigger"];
export type BuildStatus = ApiSchema["BuildStatus"];
export type ArtifactKind = ApiSchema["ArtifactKind"];
export type UserPermission = ApiSchema["UserPermission"];

export type BuildArtifact = ApiSchema["BuildArtifact"];
export type PublishedRepoFile = ApiSchema["PublishedRepoFile"];
export type BuildJob = ApiSchema["BuildJob"];
export type BuildJobResponse = ApiSchema["BuildJobResponse"];
export type JobArtifactListResponse = ApiSchema["JobArtifactListResponse"];
export type JobArtifactMetaResponse = ApiSchema["JobArtifactMetaResponse"];

export type PackageActionDisposition = ApiSchema["PackageActionDisposition"];
export type PackageActionTargetResult = ApiSchema["PackageActionTargetResult"];
export type PackageActionResponse = ApiSchema["PackageActionResponse"];

export type BuildJobListResponse = ApiSchema["BuildJobListResponse"];
export type PackageBuildInventoryEntry = ApiSchema["PackageBuildInventoryEntry"];
export type PackageBuildHistoryResponse = ApiSchema["PackageBuildHistoryResponse"];

export type RepoInventoryResponse = ApiSchema["RepoInventoryResponse"];
export type RepoTargetSummary = ApiSchema["RepoTargetSummary"];
export type RepoSummaryResponse = ApiSchema["RepoSummaryResponse"];

export type LogChunkResponse = ApiSchema["LogChunkResponse"];
export type LogMetaResponse = ApiSchema["LogMetaResponse"];
export type LogSourceType = ApiSchema["LogSourceType"];
export type LogSource = ApiSchema["LogSource"];
export type LogManifestResponse = ApiSchema["LogManifestResponse"];

export type DaemonConfig = ApiSchema["EffectiveConfigView"];
export interface EffectiveConfigResponse {
  config: DaemonConfig;
}

export type ConfigFieldType = ApiSchema["ConfigFieldType"];
export type ConfigFieldDescriptor = ApiSchema["ConfigFieldDescriptor"];
export type ConfigSchemaResponse = ApiSchema["ConfigSchemaResponse"];

export type UpdateRuntimeSettingsRequest = {
  settings: Record<string, string | number>;
};

export type ApiError = ApiSchema["ApiError"];

export type UserAccount = ApiSchema["UserAccount"];
export type UserRepoMetrics = ApiSchema["UserRepoMetrics"];
export type UserResponse = ApiSchema["UserResponse"];
export type UserListResponse = ApiSchema["UserListResponse"];
export type CreateUserRequest = ApiSchema["CreateUserRequest"];
export type UpdateUserRequest = ApiSchema["UpdateUserRequest"];
export type ChangePasswordRequest = ApiSchema["ChangePasswordRequest"];
export type UserMetricsResponse = ApiSchema["UserMetricsResponse"];

export type SessionLoginRequest = ApiSchema["SessionLoginRequest"];
export type SetupStatusResponse = ApiSchema["SetupStatusResponse"];
export type SetupAdminRequest = ApiSchema["SetupAdminRequest"];
export type SetupInitializeRequest = ApiSchema["SetupInitializeRequest"];
export type SessionResponse = ApiSchema["SessionResponse"];

export type SyncTriggerType = ApiSchema["SyncTriggerType"];
export type SyncStatus = ApiSchema["SyncStatus"];
export type SyncOperation = ApiSchema["SyncOperation"];
export type SyncOperationListQuery = ApiSchema["SyncOperationListQuery"];
export type PackageSyncOperationListQuery =
  ApiSchema["PackageSyncOperationListQuery"];
export type SyncOperationListResponse = ApiSchema["SyncOperationListResponse"];
export type SyncMetricsResponse = ApiSchema["SyncMetricsResponse"];
export type MockChrootCacheStats = ApiSchema["MockChrootCacheStats"];
export type GitMirrorCacheStats = ApiSchema["GitMirrorCacheStats"];
export type CacheStatsResponse = ApiSchema["CacheStatsResponse"];
