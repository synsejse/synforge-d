use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use synforge_core::model::{format_timestamp, now_utc};
use synforge_core::sync::{SyncOperation, SyncStage, SyncStatus, SyncTriggerType};
use time::OffsetDateTime;
use uuid::Uuid;

use super::*;
use crate::schema::sync_operations;

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = sync_operations)]
pub(super) struct SyncOperationRow {
    pub id: Uuid,
    pub package_name: String,
    pub trigger_type: String,
    pub status: String,
    pub stage: String,
    pub revision: Option<String>,
    pub previous_revision: Option<String>,
    pub changed: Option<bool>,
    pub target_mock_chroot: Option<String>,
    pub batch_id: Option<Uuid>,
    pub retry_of: Option<Uuid>,
    pub cancellation_requested: bool,
    pub queued_targets: i64,
    pub skipped_targets: i64,
    pub blocked_targets: i64,
    pub error_message: Option<String>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub started_at: Option<OffsetDateTime>,
    pub finished_at: Option<OffsetDateTime>,
}

#[derive(Insertable)]
#[diesel(table_name = sync_operations)]
pub struct NewSyncOperation<'a> {
    pub id: Uuid,
    pub package_name: &'a str,
    pub trigger_type: &'a str,
    pub status: &'a str,
    pub stage: &'a str,
    pub revision: Option<&'a str>,
    pub error_message: Option<&'a str>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub started_at: Option<OffsetDateTime>,
    pub finished_at: Option<OffsetDateTime>,
}

impl From<SyncOperationRow> for SyncOperation {
    fn from(row: SyncOperationRow) -> Self {
        Self {
            id: row.id.to_string(),
            package_name: row.package_name,
            trigger_type: row.trigger_type.parse().unwrap_or(SyncTriggerType::Poll),
            status: row.status.parse().unwrap_or(SyncStatus::Failed),
            stage: row.stage.parse().unwrap_or(SyncStage::Completed),
            revision: row.revision,
            previous_revision: row.previous_revision,
            changed: row.changed,
            target_mock_chroot: row.target_mock_chroot,
            batch_id: row.batch_id.map(|id| id.to_string()),
            retry_of: row.retry_of.map(|id| id.to_string()),
            cancellation_requested: row.cancellation_requested,
            queued_targets: row.queued_targets.max(0) as u64,
            skipped_targets: row.skipped_targets.max(0) as u64,
            blocked_targets: row.blocked_targets.max(0) as u64,
            error_message: row.error_message,
            created_at: format_timestamp(row.created_at),
            updated_at: format_timestamp(row.updated_at),
            started_at: row.started_at.map(format_timestamp),
            finished_at: row.finished_at.map(format_timestamp),
        }
    }
}

pub(super) async fn insert_sync_operation(
    store: &DieselStore,
    package_name: &str,
    trigger_type: SyncTriggerType,
    status: SyncStatus,
    revision: Option<&str>,
    error_message: Option<&str>,
) -> anyhow::Result<()> {
    let id = Uuid::now_v7();
    let created_at = now_utc();
    let package_name = package_name.to_string();
    let trigger_type_str = trigger_type.as_str().to_string();
    let status_str = status.as_str().to_string();
    let stage = SyncStage::Completed.as_str().to_string();
    let revision = revision.map(str::to_string);
    let error_message = error_message.map(str::to_string);

    let mut conn = store.get_connection().await?;
    let new_operation = NewSyncOperation {
        id,
        package_name: &package_name,
        trigger_type: &trigger_type_str,
        status: &status_str,
        stage: &stage,
        revision: revision.as_deref(),
        error_message: error_message.as_deref(),
        created_at,
        updated_at: created_at,
        started_at: Some(created_at),
        finished_at: Some(created_at),
    };
    diesel::insert_into(sync_operations::table)
        .values(&new_operation)
        .execute(&mut conn)
        .await?;
    Ok(())
}

pub(super) async fn list_sync_operations(
    store: &DieselStore,
    limit: usize,
    offset: usize,
    package_name: Option<String>,
    status: Option<SyncStatus>,
) -> anyhow::Result<Vec<SyncOperation>> {
    let package_name_filter = package_name.clone();
    let status_filter = status.map(|s| s.as_str().to_string());
    let mut conn = store.get_connection().await?;
    let mut query = sync_operations::table.into_boxed();

    if let Some(ref pkg) = package_name_filter {
        query = query.filter(sync_operations::package_name.eq(pkg));
    }

    if let Some(ref s) = status_filter {
        query = query.filter(sync_operations::status.eq(s));
    }

    let rows = query
        .order(sync_operations::created_at.desc())
        .limit(limit as i64)
        .offset(offset as i64)
        .select(SyncOperationRow::as_select())
        .load(&mut conn)
        .await?;

    Ok(rows.into_iter().map(SyncOperation::from).collect())
}

pub(super) async fn count_sync_operations(
    store: &DieselStore,
    package_name: Option<String>,
    status: Option<SyncStatus>,
) -> anyhow::Result<u64> {
    let package_name_filter = package_name.clone();
    let status_filter = status.map(|s| s.as_str().to_string());
    let mut conn = store.get_connection().await?;
    let mut query = sync_operations::table.into_boxed();

    if let Some(ref pkg) = package_name_filter {
        query = query.filter(sync_operations::package_name.eq(pkg));
    }

    if let Some(ref s) = status_filter {
        query = query.filter(sync_operations::status.eq(s));
    }

    let count = query.count().get_result::<i64>(&mut conn).await?;
    Ok(count as u64)
}

/// Selects raw `(created_at, status)` rows for sync operations since
/// `cutoff`. Bucketing into time windows happens at the service layer
/// where we already need to fold succeeded/failed counts and fill empty
/// buckets — keeps this query a plain Diesel select.
pub(super) async fn list_recent_sync_status_events(
    store: &DieselStore,
    cutoff: OffsetDateTime,
) -> anyhow::Result<Vec<(OffsetDateTime, String)>> {
    let mut conn = store.get_connection().await?;
    let rows: Vec<(OffsetDateTime, String)> = sync_operations::table
        .filter(sync_operations::created_at.gt(&cutoff))
        .select((sync_operations::created_at, sync_operations::status))
        .order_by(sync_operations::created_at.asc())
        .load(&mut conn)
        .await?;
    Ok(rows)
}

pub(super) async fn last_sync_at_per_package(
    store: &DieselStore,
) -> anyhow::Result<Vec<(String, OffsetDateTime)>> {
    use diesel::dsl::max;
    let mut conn = store.get_connection().await?;
    let rows: Vec<(String, Option<OffsetDateTime>)> = sync_operations::table
        .group_by(sync_operations::package_name)
        .select((
            sync_operations::package_name,
            max(sync_operations::created_at),
        ))
        .load(&mut conn)
        .await?;
    Ok(rows
        .into_iter()
        .filter_map(|(name, ts)| ts.map(|t| (name, t)))
        .collect())
}

pub(super) async fn get_sync_metrics(
    store: &DieselStore,
) -> anyhow::Result<(usize, usize, Option<String>)> {
    let twenty_four_hours_ago = OffsetDateTime::now_utc() - time::Duration::hours(24);

    let mut conn = store.get_connection().await?;
    let succeeded_count = sync_operations::table
        .filter(sync_operations::status.eq(SyncStatus::Succeeded.as_str()))
        .filter(sync_operations::created_at.gt(&twenty_four_hours_ago))
        .count()
        .get_result::<i64>(&mut conn)
        .await? as usize;

    let failed_count = sync_operations::table
        .filter(sync_operations::status.eq(SyncStatus::Failed.as_str()))
        .filter(sync_operations::created_at.gt(&twenty_four_hours_ago))
        .count()
        .get_result::<i64>(&mut conn)
        .await? as usize;

    let last_failure: Option<OffsetDateTime> = sync_operations::table
        .filter(sync_operations::status.eq(SyncStatus::Failed.as_str()))
        .order(sync_operations::created_at.desc())
        .select(sync_operations::created_at)
        .first(&mut conn)
        .await
        .optional()?;

    Ok((
        succeeded_count,
        failed_count,
        last_failure.map(format_timestamp),
    ))
}
