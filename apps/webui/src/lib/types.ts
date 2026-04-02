// API Types matching the Rust daemon

export interface SpecSource {
  repo_url: string;
  spec_path: string;
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
  mock_chroots: string[];
  source: SpecSource;
  poll_interval_seconds: number;
  build_timeout_seconds: number;
  package_history_count: number;
  build_env: BuildEnvVar[];
  spec_path: string;
  version: string;
  release: string;
}

export interface PackageState {
  last_revision: string | null;
  last_successful_build_id: string | null;
  active_job_id: string | null;
}

export interface PackageResponse {
  package: PackageDefinition;
  state: PackageState;
}

export interface PackageListResponse {
  packages: PackageResponse[];
}

export interface CreatePackageRequest {
  name: string;
  source: SpecSource;
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
export type BuildStatus = "pending" | "running" | "succeeded" | "failed" | "timed_out";
export type ArtifactKind = "rpm" | "srpm" | "log" | "other";

export interface BuildArtifact {
  path: string;
  relative_repo_path: string;
  sha256: string;
  size_bytes: number;
  kind: ArtifactKind;
}

export interface PublishedRepoFile {
  job_id: string;
  package_name: string;
  repo_path: string;
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
  spec_path: string;
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

export interface BuildJobListResponse {
  jobs: BuildJobResponse[];
}

export interface PackageBuildInventoryEntry {
  build: BuildJobResponse;
  repo_files: PublishedRepoFile[];
}

export interface PackageBuildHistoryResponse {
  package_name: string;
  builds: PackageBuildInventoryEntry[];
}

export interface PackageRepoFilesResponse {
  package_name: string;
  repo_files: PublishedRepoFile[];
}

export interface RepoInventoryResponse {
  repo_files: PublishedRepoFile[];
}

export interface LogChunkResponse {
  job_id: string;
  contents: string;
  cursor: number;
  complete: boolean;
}

export interface DaemonConfig {
  listen_addr: string;
  bearer_token: string;
  runtime_root: string;
  database_path: string;
  packages_dir: string;
  repo_dir: string;
  jobs_root: string;
  worker_image: string;
  max_concurrent_builds: number;
  public_base_url: string;
  worker_listen_addr: string;
  worker_connect_addr: string;
}

export interface EffectiveConfigResponse {
  config: DaemonConfig;
}

export interface UpdateRuntimeSettingsRequest {
  public_base_url: string;
}

export interface ApiError {
  code: "unauthorized" | "not_found" | "conflict" | "bad_request" | "internal_error";
  message: string;
}
