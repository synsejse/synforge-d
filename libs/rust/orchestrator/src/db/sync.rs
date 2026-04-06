use diesel::prelude::*;
use synforge_core::model::{format_timestamp, now_utc};
use synforge_core::sync::{SyncOperation, SyncStatus, SyncTriggerType};
use uuid::Uuid;

use super::*;
use crate::schema::sync_operations;

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = sync_operations)]
pub struct SyncOperationRow {
    pub id: String,
    pub package_name: String,
    pub trigger_type: String,
    pub status: String,
    pub revision: Option<String>,
    pub error_message: Option<String>,
    pub created_at: String,
}

#[derive(Insertable)]
#[diesel(table_name = sync_operations)]
pub struct NewSyncOperation<'a> {
    pub id: &'a str,
    pub package_name: &'a str,
    pub trigger_type: &'a str,
    pub status: &'a str,
    pub revision: Option<&'a str>,
    pub error_message: Option<&'a str>,
    pub created_at: &'a str,
}

impl From<SyncOperationRow> for SyncOperation {
    fn from(row: SyncOperationRow) -> Self {
        Self {
            id: row.id,
            package_name: row.package_name,
            trigger_type: SyncTriggerType::from_str(&row.trigger_type)
                .unwrap_or(SyncTriggerType::Poll),
            status: SyncStatus::from_str(&row.status).unwrap_or(SyncStatus::Failed),
            revision: row.revision,
            error_message: row.error_message,
            created_at: row.created_at,
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
    let id = Uuid::now_v7().to_string();
    let created_at = format_timestamp(now_utc());
    let package_name = package_name.to_string();
    let trigger_type_str = trigger_type.as_str().to_string();
    let status_str = status.as_str().to_string();
    let revision = revision.map(str::to_string);
    let error_message = error_message.map(str::to_string);

    store
        .with_connection(move |conn| {
            let new_operation = NewSyncOperation {
                id: &id,
                package_name: &package_name,
                trigger_type: &trigger_type_str,
                status: &status_str,
                revision: revision.as_deref(),
                error_message: error_message.as_deref(),
                created_at: &created_at,
            };
            diesel::insert_into(sync_operations::table)
                .values(&new_operation)
                .execute(conn)?;
            Ok(())
        })
        .await
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

    store
        .with_connection(move |conn| {
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
                .load(conn)?;

            Ok(rows.into_iter().map(SyncOperation::from).collect())
        })
        .await
}

pub(super) async fn count_sync_operations(
    store: &DieselStore,
    package_name: Option<String>,
    status: Option<SyncStatus>,
) -> anyhow::Result<u64> {
    let package_name_filter = package_name.clone();
    let status_filter = status.map(|s| s.as_str().to_string());

    store
        .with_connection(move |conn| {
            let mut query = sync_operations::table.into_boxed();

            if let Some(ref pkg) = package_name_filter {
                query = query.filter(sync_operations::package_name.eq(pkg));
            }

            if let Some(ref s) = status_filter {
                query = query.filter(sync_operations::status.eq(s));
            }

            let count = query.count().get_result::<i64>(conn)?;
            Ok(count as u64)
        })
        .await
}

pub(super) async fn get_sync_metrics(
    store: &DieselStore,
) -> anyhow::Result<(usize, usize, Option<String>)> {
    let twenty_four_hours_ago = {
        use time::OffsetDateTime;
        let now = OffsetDateTime::now_utc();
        let past = now - time::Duration::hours(24);
        past.format(&time::format_description::well_known::Rfc3339)?
    };

    store
        .with_connection(move |conn| {
            let succeeded_count = sync_operations::table
                .filter(sync_operations::status.eq(SyncStatus::Succeeded.as_str()))
                .filter(sync_operations::created_at.gt(&twenty_four_hours_ago))
                .count()
                .get_result::<i64>(conn)? as usize;

            let failed_count = sync_operations::table
                .filter(sync_operations::status.eq(SyncStatus::Failed.as_str()))
                .filter(sync_operations::created_at.gt(&twenty_four_hours_ago))
                .count()
                .get_result::<i64>(conn)? as usize;

            let last_failure: Option<String> = sync_operations::table
                .filter(sync_operations::status.eq(SyncStatus::Failed.as_str()))
                .order(sync_operations::created_at.desc())
                .select(sync_operations::created_at)
                .first(conn)
                .optional()?;

            Ok((succeeded_count, failed_count, last_failure))
        })
        .await
}
