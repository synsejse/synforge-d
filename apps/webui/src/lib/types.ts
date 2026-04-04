// API Types matching the Rust daemon

export interface SpecSource {
  repo_url: string;
  spec_file: string;
  poll: boolean;
}

export interface BuildEnvVar {
  key: string;
  value: string;
}

export interface PackageDefinition {
  name: string;
  description: string;
  enabled: boolean;
  repo_subdir: string;
  publish_srpm: boolean;
  network_access: boolean;
  mock_chroots: string[];
  source: SpecSource;
  poll_interval_seconds: number;
  build_timeout_seconds: number;
  package_history_count: number;
  build_env: BuildEnvVar[];
  spec_file: string;
  version: string;
  release: string;
}

export interface PackageState {
  last_revision: string | null;
  last_successful_build_id: string | null;
  active_job_id: string | null;
  targets: PackageTargetState[];
}

export interface PackageTargetState {
  mock_chroot: string;
  last_revision: string | null;
  last_successful_build_id: string | null;
  active_job_id: string | null;
  active_status: BuildStatus | null;
}

export interface PackageResponse {
  package: PackageDefinition;
  state: PackageState;
}

export interface PackageListResponse {
  packages: PackageResponse[];
  page: PageInfo;
}

export interface PageInfo {
  limit: number;
  offset: number;
  returned: number;
  total?: number;
  has_more: boolean;
}

export interface CreatePackageRequest {
  name: string;
  source: SpecSource;
  network_access: boolean;
  mock_chroots: string[];
  poll_interval_seconds: number;
  build_timeout_seconds: number;
  package_history_count: number;
  build_env: BuildEnvVar[];
}

export interface UpdatePackageRequest {
  source: SpecSource;
  enabled?: boolean;
  publish_srpm?: boolean;
  network_access?: boolean;
  mock_chroots?: string[];
  poll_interval_seconds?: number;
  build_timeout_seconds?: number;
  package_history_count?: number;
  build_env?: BuildEnvVar[];
}

export interface BrowseRepositoryRequest {
  repo_url: string;
}

export interface BrowseRepositoryResponse {
  repo_url: string;
  head_commit: string;
  files: string[];
  spec_files: string[];
}

export interface MockChrootListResponse {
  chroots: string[];
}

export type BuildTrigger = "poll" | "manual_rebuild" | "manual_refresh" | "api";
export type BuildStatus =
  | "pending"
  | "running"
  | "succeeded"
  | "failed"
  | "timed_out";
export type ArtifactKind = "rpm" | "srpm" | "log" | "other";
export type UserPermission = "read" | "write" | "repo";

export interface BuildArtifact {
  id: string;
  file: string;
  sha256: string;
  size_bytes: number;
  kind: ArtifactKind;
}

export interface PublishedRepoFile {
  artifact_id: string;
  job_id: string;
  package_name: string;
  mock_chroot: string;
  path: string;
  sha256: string;
  size_bytes: number;
  kind: ArtifactKind;
  published_at: string;
}

export interface BuildJob {
  id: string;
  package_name: string;
  mock_chroot: string;
  revision: string;
  trigger: BuildTrigger;
  status: BuildStatus;
  spec_file: string;
  worker_container_id: string | null;
  created_at: string;
  updated_at: string;
  finished_at: string | null;
  error_message: string | null;
}

export interface BuildJobResponse {
  job: BuildJob;
  artifacts: BuildArtifact[];
}

export interface JobArtifactListResponse {
  job_id: string;
  artifacts: BuildArtifact[];
}

export interface JobArtifactMetaResponse {
  job_id: string;
  artifact: BuildArtifact;
}

export type PackageActionDisposition = "queued" | "skipped" | "blocked";

export interface PackageActionTargetResult {
  package_name: string;
  mock_chroot: string;
  disposition: PackageActionDisposition;
  reason: string | null;
  job_id: string | null;
  revision: string | null;
}

export interface PackageActionResponse {
  package_name: string;
  trigger: BuildTrigger;
  results: PackageActionTargetResult[];
}

export interface BuildJobListResponse {
  jobs: BuildJobResponse[];
  page: PageInfo;
}

export interface PackageBuildInventoryEntry {
  build: BuildJobResponse;
  repo_files: PublishedRepoFile[];
}

export interface PackageBuildHistoryResponse {
  package_name: string;
  builds: PackageBuildInventoryEntry[];
  page: PageInfo;
}

export interface RepoInventoryResponse {
  repo_files: PublishedRepoFile[];
  page: PageInfo;
}

export interface RepoTargetSummary {
  mock_chroot: string;
  package_count: number;
  build_count: number;
  size_bytes: number;
}

export interface RepoSummaryResponse {
  package_count: number;
  target_count: number;
  build_count: number;
  stored_bytes: number;
  published_file_count: number;
  targets: RepoTargetSummary[];
  recent_files: PublishedRepoFile[];
}

export interface LogChunkResponse {
  job_id: string;
  source: string;
  contents: string;
  start_line: number;
  cursor: number;
  complete: boolean;
}

export interface LogMetaResponse {
  job_id: string;
  source: string;
  file_size: number;
  max_cursor: number;
}

export type LogSourceType = "structured" | "raw";

export interface LogSource {
  file: string;
  size: number;
  source_type: LogSourceType;
}

export interface LogManifestResponse {
  job_id: string;
  sources: LogSource[];
}

export interface DaemonConfig {
  config_path: string;
  bootstrap_completed: boolean;
  listen_addr: string;
  runtime_root: string;
  database_url: string;
  packages_dir: string;
  repo_dir: string;
  jobs_root: string;
  worker_image: string;
  max_concurrent_builds: number;
  db_pool_size: number;
  queue_buffer_size: number;
  poller_tick_seconds: number;
  worker_result_timeout_seconds: number;
  worker_socket_timeout_seconds: number;
  git_operation_timeout_seconds: number;
  public_base_url: string;
}

export interface EffectiveConfigResponse {
  config: DaemonConfig;
}

export type ConfigFieldType = "string" | "number";

export interface ConfigFieldDescriptor {
  key: string;
  label: string;
  description: string;
  section_key: string;
  section_label: string;
  type: ConfigFieldType;
  required: boolean;
  min_value?: number;
  editable_in_setup: boolean;
  editable_in_runtime: boolean;
  default_value: string | number;
}

export interface ConfigSchemaResponse {
  fields: ConfigFieldDescriptor[];
}

export interface UpdateRuntimeSettingsRequest {
  settings: Record<string, string | number>;
}

export interface ApiError {
  code:
    | "unauthorized"
    | "not_found"
    | "conflict"
    | "bad_request"
    | "internal_error";
  message: string;
}

export interface UserAccount {
  id: string;
  handle: string;
  display_name: string;
  active: boolean;
  permissions: UserPermission[];
  created_at: string;
  updated_at: string;
}

export interface UserRepoMetrics {
  user_id: string;
  downloaded_bytes: number;
  updated_at: string;
}

export interface UserResponse {
  user: UserAccount;
  metrics: UserRepoMetrics;
}

export interface UserListResponse {
  users: UserResponse[];
}

export interface CreateUserRequest {
  handle: string;
  display_name: string;
  password: string;
  permissions: UserPermission[];
  active: boolean;
}

export interface UpdateUserRequest {
  handle: string;
  display_name: string;
  permissions: UserPermission[];
  active: boolean;
}

export interface ChangePasswordRequest {
  password: string;
}

export interface SessionLoginRequest {
  handle: string;
  password: string;
}

export interface SetupStatusResponse {
  initialized: boolean;
}

export interface SetupAdminRequest {
  handle: string;
  display_name: string;
  password: string;
}

export interface SetupInitializeRequest {
  settings: Record<string, string | number>;
  admin: SetupAdminRequest;
}

export interface UserMetricsResponse {
  metrics: UserRepoMetrics;
}

export interface SessionResponse {
  user: UserAccount;
}
