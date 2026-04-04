use utoipa::openapi::security::{ApiKey, ApiKeyValue, Http, HttpAuthScheme, SecurityScheme};
use utoipa::{Modify, OpenApi};

use crate::api;

pub struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Default::default);
        components.add_security_scheme(
            "session_auth",
            SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::new("synforge_session"))),
        );
        components.add_security_scheme(
            "basic_auth",
            SecurityScheme::Http(Http::new(HttpAuthScheme::Basic)),
        );
    }
}

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Synforge API",
        version = "v1",
        description = "Daemon API for package orchestration, logs, repository management, setup, and user administration. `/api/v1` uses a signed session cookie for the WebUI. `/repo` uses HTTP Basic Auth for package consumers."
    ),
    paths(
        api::list_packages,
        api::get_package,
        api::create_package,
        api::update_package,
        api::delete_package,
        api::list_mock_chroots,
        api::browse_repository,
        api::get_package_builds,
        api::get_package_repo_files,
        api::trigger_rebuild,
        api::trigger_refresh,
        api::trigger_target_rebuild,
        api::trigger_target_refresh,
        api::list_jobs,
        api::get_job,
        api::delete_job,
        api::prune_failed_jobs,
        api::get_job_log_manifest,
        api::get_job_log_meta,
        api::get_job_log_meta_by_source,
        api::get_job_log_chunk,
        api::get_job_log_chunk_by_source,
        api::download_job_artifact,
        api::get_repo_inventory,
        api::get_repo_summary,
        api::get_effective_config,
        api::get_config_schema,
        api::update_runtime_settings,
        api::get_setup_status,
        api::initialize_setup,
        api::login_session,
        api::logout_session,
        api::get_session,
        api::list_users,
        api::create_user,
        api::update_user,
        api::change_user_password,
        api::delete_user,
        api::get_user_metrics
    ),
    components(
        schemas(
            synforge_core::api::ApiError,
            synforge_core::api::BrowseRepositoryRequest,
            synforge_core::api::BrowseRepositoryResponse,
            synforge_core::api::BuildJobListResponse,
            synforge_core::api::BuildJobResponse,
            synforge_core::api::ChangePasswordRequest,
            synforge_core::api::ConfigFieldDescriptor,
            synforge_core::api::ConfigFieldType,
            synforge_core::api::ConfigSchemaResponse,
            synforge_core::api::CreatePackageRequest,
            synforge_core::api::CreateUserRequest,
            synforge_core::api::EffectiveConfigDto,
            synforge_core::api::EffectiveConfigView,
            synforge_core::api::LogChunkResponse,
            synforge_core::api::LogManifestResponse,
            synforge_core::api::LogMetaResponse,
            synforge_core::api::LogSource,
            synforge_core::api::LogSourceType,
            synforge_core::api::MockChrootListResponse,
            synforge_core::api::PackageActionDisposition,
            synforge_core::api::PackageActionResponse,
            synforge_core::api::PackageActionTargetResult,
            synforge_core::api::PackageBuildHistoryResponse,
            synforge_core::api::PackageBuildInventoryEntry,
            synforge_core::api::PackageListResponse,
            synforge_core::api::PackageRepoFilesResponse,
            synforge_core::api::PackageResponse,
            synforge_core::api::PageInfo,
            synforge_core::api::PruneJobsResponse,
            synforge_core::api::RefreshRequest,
            synforge_core::api::RebuildRequest,
            synforge_core::api::RepoInventoryResponse,
            synforge_core::api::RepoSummaryResponse,
            synforge_core::api::RepoTargetSummary,
            synforge_core::api::SessionLoginRequest,
            synforge_core::api::SessionResponse,
            synforge_core::api::SetupAdminRequest,
            synforge_core::api::SetupInitializeRequest,
            synforge_core::api::SetupStatusResponse,
            synforge_core::api::UpdatePackageRequest,
            synforge_core::api::UpdateRuntimeSettingsRequest,
            synforge_core::api::UpdateUserRequest,
            synforge_core::api::JobListQuery,
            synforge_core::api::UserListResponse,
            synforge_core::api::UserMetricsResponse,
            synforge_core::api::UserResponse,
            synforge_core::model::ArtifactKind,
            synforge_core::model::BuildArtifact,
            synforge_core::model::BuildJob,
            synforge_core::model::BuildStatus,
            synforge_core::model::BuildTrigger,
            synforge_core::model::PackageRuntimeState,
            synforge_core::model::PublishedRepoFile,
            synforge_core::model::UserAccount,
            synforge_core::model::UserPermission,
            synforge_core::model::UserRepoMetrics,
            synforge_core::package::BuildEnvVar,
            synforge_core::package::PackageDefinition,
            synforge_core::package::SpecSource
        )
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "Setup", description = "First-run setup and bootstrap"),
        (name = "Session", description = "WebUI session management"),
        (name = "Packages", description = "Package definitions and package-level operations"),
        (name = "Jobs", description = "Build job lifecycle and artifacts"),
        (name = "Logs", description = "Per-job log source discovery and streaming"),
        (name = "Repository", description = "Managed repository inventory and source browsing"),
        (name = "Settings", description = "Daemon configuration schema and runtime updates"),
        (name = "Users", description = "Administrative user and permissions management")
    )
)]
pub struct ApiDoc;
