import type { components, operations, paths } from "../generated/api/api-schema";

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
export type JobResourceUsageSample = ApiSchema["JobResourceUsageSample"];
export type JobResourceUsageListResponse = ApiSchema["JobResourceUsageListResponse"];
export type JobResourceUsageResponse = ApiSchema["JobResourceUsageResponse"];

export type PackageActionDisposition = ApiSchema["PackageActionDisposition"];
export type PackageActionTargetResult = ApiSchema["PackageActionTargetResult"];
export type PackageActionResponse = ApiSchema["PackageActionResponse"];
export type RefreshAllPackagesState = ApiSchema["RefreshAllPackagesState"];
export type RefreshAllPackagesProgressView =
  ApiSchema["RefreshAllPackagesProgressView"];
export type RefreshAllPackagesProgressResponse =
  ApiSchema["RefreshAllPackagesProgressResponse"];
export type RefreshAllPackagesResponse =
  ApiSchema["RefreshAllPackagesResponse"];

export type BuildJobListResponse = ApiSchema["BuildJobListResponse"];
export type PruneJobsResponse = ApiSchema["PruneJobsResponse"];
export type PackageBuildInventoryEntry =
  ApiSchema["PackageBuildInventoryEntry"];
export type PackageBuildHistoryResponse =
  ApiSchema["PackageBuildHistoryResponse"];

export type RepoInventoryResponse = ApiSchema["RepoInventoryResponse"];
export type RepoTargetSummary = ApiSchema["RepoTargetSummary"];
export type RepoSummaryResponse = ApiSchema["RepoSummaryResponse"];
export type RepoSetupInfoResponse = ApiSchema["RepoSetupInfoResponse"];

export type LogSourceType = ApiSchema["LogSourceType"];
export type LogSource = ApiSchema["LogSource"];
export type LogManifestResponse = ApiSchema["LogManifestResponse"];

export type DaemonConfig = ApiSchema["EffectiveConfigView"];
export type EffectiveConfigResponse = ApiSchema["EffectiveConfigDto"];

export type ConfigFieldType = ApiSchema["ConfigFieldType"];
export type ConfigFieldDescriptor = ApiSchema["ConfigFieldDescriptor"];
export type ConfigSchemaResponse = ApiSchema["ConfigSchemaResponse"];
export type RepoSigningStatusView = ApiSchema["RepoSigningStatusView"];
export type RepoSigningStatusResponse = ApiSchema["RepoSigningStatusResponse"];
export type RepoSigningReconcileMode = ApiSchema["RepoSigningReconcileMode"];
export type RepoSigningReconcileState = ApiSchema["RepoSigningReconcileState"];
export type RepoSigningReconcileProgressView =
  ApiSchema["RepoSigningReconcileProgressView"];
export type RepoSigningReconcileProgressResponse =
  ApiSchema["RepoSigningReconcileProgressResponse"];
export type UpdateRepoSigningConfigRequest =
  ApiSchema["UpdateRepoSigningConfigRequest"];
export type GenerateRepoSigningKeyResponse =
  ApiSchema["GenerateRepoSigningKeyResponse"];
export type ImportRepoSigningKeyRequest =
  ApiSchema["ImportRepoSigningKeyRequest"];
export type ImportRepoSigningKeyResponse =
  ApiSchema["ImportRepoSigningKeyResponse"];
export type TestRepoSigningResponse = ApiSchema["TestRepoSigningResponse"];
export type ExportRepoSigningKeyResponse =
  ApiSchema["ExportRepoSigningKeyResponse"];
export type ExportRepoSigningPublicKeyResponse =
  ApiSchema["ExportRepoSigningPublicKeyResponse"];

export type UpdateRuntimeSettingsRequest = ApiSchema["UpdateRuntimeSettingsRequest"];

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
export type SyncStage = ApiSchema["SyncStage"];
export type SyncEventLevel = ApiSchema["SyncEventLevel"];
export type SyncOperation = ApiSchema["SyncOperation"];
export type SyncOperationEvent = ApiSchema["SyncOperationEvent"];
export type SyncOperationListQuery = ApiSchema["SyncOperationListQuery"];
export type PackageSyncOperationListQuery =
  ApiSchema["PackageSyncOperationListQuery"];
export type SyncOperationListResponse = ApiSchema["SyncOperationListResponse"];
export type SyncOperationDetailResponse =
  ApiSchema["SyncOperationDetailResponse"];
export type SyncEnqueueResponse = ApiSchema["SyncEnqueueResponse"];
export type SyncBatchStatus = ApiSchema["SyncBatchStatus"];
export type SyncBatch = ApiSchema["SyncBatch"];
export type SyncBatchDetailResponse = ApiSchema["SyncBatchDetailResponse"];
export type SyncBatchListResponse = ApiSchema["SyncBatchListResponse"];
export type SyncMetricsResponse = ApiSchema["SyncMetricsResponse"];
export type SyncScheduleEntry = ApiSchema["SyncScheduleEntry"];
export type SyncScheduleResponse = ApiSchema["SyncScheduleResponse"];
export type MockChrootCacheStats = ApiSchema["MockChrootCacheStats"];
export type GitMirrorCacheStats = ApiSchema["GitMirrorCacheStats"];
export type CacheStatsResponse = ApiSchema["CacheStatsResponse"];
export type ServerHardwareResponse = ApiSchema["ServerHardwareResponse"];
export type TimeSeriesPoint = ApiSchema["TimeSeriesPoint"];
export type TimeSeriesResponse = ApiSchema["TimeSeriesResponse"];
export type TimeRange = "24h" | "7d" | "30d";
