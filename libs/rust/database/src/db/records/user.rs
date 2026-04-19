use super::super::*;

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = users)]
pub struct UserRecord {
    pub id: Uuid,
    pub handle: String,
    pub display_name: String,
    pub password_hash: String,
    pub active: bool,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone)]
pub struct UserAuthRecord {
    pub user: UserAccount,
    pub password_hash: String,
}

#[derive(Insertable)]
#[diesel(table_name = users)]
pub(crate) struct NewUserRecord<'a> {
    pub(crate) id: Uuid,
    pub(crate) handle: &'a str,
    pub(crate) display_name: &'a str,
    pub(crate) password_hash: &'a str,
    pub(crate) active: bool,
    pub(crate) created_at: OffsetDateTime,
    pub(crate) updated_at: OffsetDateTime,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = user_permissions)]
pub(crate) struct UserPermissionRecord {
    pub(crate) user_id: Uuid,
    pub(crate) permission: UserPermission,
}

#[derive(Insertable)]
#[diesel(table_name = user_permissions)]
pub(crate) struct NewUserPermissionRecord {
    pub(crate) user_id: Uuid,
    pub(crate) permission: UserPermission,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = user_repo_metrics)]
pub(crate) struct UserRepoMetricsRecord {
    pub(crate) user_id: Uuid,
    pub(crate) downloaded_bytes: i64,
    pub(crate) updated_at: OffsetDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = user_repo_metrics)]
pub(crate) struct NewUserRepoMetricsRecord {
    pub(crate) user_id: Uuid,
    pub(crate) downloaded_bytes: i64,
    pub(crate) updated_at: OffsetDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = runtime_settings)]
pub(crate) struct NewRuntimeSettingRecord<'a> {
    pub(crate) key: &'a str,
    pub(crate) value_json: serde_json::Value,
    pub(crate) updated_at: OffsetDateTime,
}

pub(crate) fn user_from_record(
    row: UserRecord,
    permissions: Vec<UserPermission>,
) -> anyhow::Result<UserAccount> {
    Ok(UserAccount {
        id: row.id,
        handle: row.handle,
        display_name: row.display_name,
        active: row.active,
        permissions,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}

pub(crate) fn user_metrics_from_record(
    row: UserRepoMetricsRecord,
) -> anyhow::Result<UserRepoMetrics> {
    Ok(UserRepoMetrics {
        user_id: row.user_id,
        downloaded_bytes: row.downloaded_bytes.max(0) as u64,
        updated_at: row.updated_at,
    })
}
