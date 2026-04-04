use std::collections::BTreeMap;
use std::path::PathBuf;

use diesel::backend::Backend;
use diesel::deserialize::{self, FromSql, FromSqlRow};
use diesel::expression::AsExpression;
use diesel::serialize::{self, Output, ToSql};
use diesel::sql_types::Text;
use serde::{Deserialize, Serialize};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, AsExpression, FromSqlRow, ToSchema,
)]
#[serde(rename_all = "snake_case")]
#[diesel(sql_type = Text)]
pub enum BuildTrigger {
    Poll,
    ManualRefresh,
    ManualRebuild,
    Api,
}

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, AsExpression, FromSqlRow, ToSchema,
)]
#[serde(rename_all = "snake_case")]
#[diesel(sql_type = Text)]
pub enum BuildStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
    TimedOut,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct BuildArtifact {
    pub id: Uuid,
    pub package_name: String,
    pub mock_chroot: String,
    #[schema(value_type = String)]
    pub path: PathBuf,
    pub sha256: String,
    pub size_bytes: u64,
    pub kind: ArtifactKind,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct PublishedRepoFile {
    pub artifact_id: Uuid,
    pub job_id: Uuid,
    pub package_name: String,
    pub mock_chroot: String,
    #[schema(value_type = String)]
    pub repo_path: PathBuf,
    pub sha256: String,
    pub size_bytes: u64,
    pub kind: ArtifactKind,
    pub published_at: OffsetDateTime,
}

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, AsExpression, FromSqlRow, ToSchema,
)]
#[serde(rename_all = "snake_case")]
#[diesel(sql_type = Text)]
pub enum ArtifactKind {
    Rpm,
    Srpm,
    Log,
    Other,
}

#[derive(
    Debug,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    AsExpression,
    FromSqlRow,
    ToSchema,
)]
#[serde(rename_all = "snake_case")]
#[diesel(sql_type = Text)]
pub enum UserPermission {
    Read,
    Write,
    Repo,
}

macro_rules! impl_text_enum {
    ($name:ident {
        $($variant:ident => [$primary:literal $(, $alias:literal)*]),+ $(,)?
    }) => {
        impl $name {
            fn db_text(self) -> &'static str {
                match self {
                    $(Self::$variant => $primary,)+
                }
            }

            fn from_db_text(value: &str) -> deserialize::Result<Self> {
                match value {
                    $($primary $(| $alias)* => Ok(Self::$variant),)+
                    _ => Err(format!("invalid {} value: {}", stringify!($name), value).into()),
                }
            }
        }

        impl<DB> ToSql<Text, DB> for $name
        where
            DB: Backend,
            str: ToSql<Text, DB>,
        {
            fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, DB>) -> serialize::Result {
                self.db_text().to_sql(out)
            }
        }

        impl<DB> FromSql<Text, DB> for $name
        where
            DB: Backend,
            String: FromSql<Text, DB>,
        {
            fn from_sql(value: DB::RawValue<'_>) -> deserialize::Result<Self> {
                let value = String::from_sql(value)?;
                Self::from_db_text(&value)
            }
        }
    };
}

impl_text_enum!(BuildTrigger {
    Poll => ["poll"],
    ManualRefresh => ["manual_refresh", "manualrefresh"],
    ManualRebuild => ["manual_rebuild", "manualrebuild"],
    Api => ["api"],
});

impl_text_enum!(BuildStatus {
    Pending => ["pending"],
    Running => ["running"],
    Succeeded => ["succeeded"],
    Failed => ["failed"],
    TimedOut => ["timed_out", "timedout"],
});

impl_text_enum!(ArtifactKind {
    Rpm => ["rpm"],
    Srpm => ["srpm"],
    Log => ["log"],
    Other => ["other"],
});

impl_text_enum!(UserPermission {
    Read => ["read"],
    Write => ["write"],
    Repo => ["repo"],
});

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct BuildJob {
    pub id: Uuid,
    pub package_name: String,
    pub mock_chroot: String,
    pub revision: String,
    pub trigger: BuildTrigger,
    pub status: BuildStatus,
    #[schema(value_type = String)]
    pub spec_path: PathBuf,
    pub worker_container_id: Option<String>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub finished_at: Option<OffsetDateTime>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct UserAccount {
    pub id: Uuid,
    pub handle: String,
    pub display_name: String,
    pub active: bool,
    pub permissions: Vec<UserPermission>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl UserAccount {
    pub fn has_permission(&self, permission: UserPermission) -> bool {
        self.permissions.contains(&permission)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct UserRepoMetrics {
    pub user_id: Uuid,
    pub downloaded_bytes: u64,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserSummary {
    pub user: UserAccount,
    pub metrics: UserRepoMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct PackageRuntimeState {
    pub last_revision: Option<String>,
    pub last_successful_build_id: Option<Uuid>,
    pub active_job_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SourceRevision {
    pub source_name: String,
    pub revision: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkerJobPayload {
    pub job_id: Uuid,
    pub workspace_dir: PathBuf,
    pub timeout_seconds: u64,
    pub action: WorkerAction,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum WorkerAction {
    Parse(WorkerParsePayload),
    Build(Box<WorkerBuildPayload>),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkerParsePayload {
    pub package_name: String,
    pub source: crate::package::SpecSource,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkerBuildPayload {
    pub package_name: String,
    pub package: crate::package::PackageDefinition,
    pub mock_chroot: String,
    pub trigger: BuildTrigger,
    pub revision: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checkout_commit: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkerParseResult {
    pub parsed: crate::package::ParsedSpec,
    pub revision: crate::package::SpecRevision,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkerBuildResult {
    pub job_id: Uuid,
    pub package_name: String,
    pub status: BuildStatus,
    pub artifacts: Vec<BuildArtifact>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum WorkerResult {
    Parse(WorkerParseResult),
    Build(WorkerBuildResult),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RepoPublication {
    pub package_name: String,
    pub repo_root: PathBuf,
    pub published_at: OffsetDateTime,
    pub files: Vec<PublishedRepoFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EffectiveConfigResponse {
    pub config: crate::config::DaemonConfig,
}

pub fn now_utc() -> OffsetDateTime {
    OffsetDateTime::now_utc()
}

pub fn format_timestamp(value: OffsetDateTime) -> String {
    value
        .format(&Rfc3339)
        .unwrap_or_else(|_| value.unix_timestamp().to_string())
}

pub fn env_map_to_vec(map: &BTreeMap<String, String>) -> Vec<(String, String)> {
    map.iter()
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect()
}
