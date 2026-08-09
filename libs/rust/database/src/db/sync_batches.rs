use diesel::prelude::*;
use diesel_async::{AsyncConnection, RunQueryDsl};
use synforge_core::{
    model::{format_timestamp, now_utc},
    sync::{SyncBatch, SyncBatchStatus, SyncOperation, SyncStatus, SyncTriggerType},
};
use uuid::Uuid;

use super::{DieselStore, sync::SyncOperationRow};
use crate::schema::{sync_batches, sync_operations};

#[derive(Queryable, Selectable)]
#[diesel(table_name = sync_batches)]
struct SyncBatchRow {
    id: Uuid,
    trigger_type: String,
    status: String,
    total_packages: i64,
    completed_packages: i64,
    succeeded_packages: i64,
    failed_packages: i64,
    cancelled_packages: i64,
    deduplicated_packages: i64,
    enqueue_failed_packages: i64,
    created_at: time::OffsetDateTime,
    updated_at: time::OffsetDateTime,
    started_at: Option<time::OffsetDateTime>,
    finished_at: Option<time::OffsetDateTime>,
    error_message: Option<String>,
}

#[derive(Insertable)]
#[diesel(table_name = sync_batches)]
struct NewSyncBatch<'a> {
    id: Uuid,
    trigger_type: &'a str,
    status: &'a str,
    total_packages: i64,
    created_at: time::OffsetDateTime,
    updated_at: time::OffsetDateTime,
    started_at: Option<time::OffsetDateTime>,
}

impl From<SyncBatchRow> for SyncBatch {
    fn from(row: SyncBatchRow) -> Self {
        Self {
            id: row.id.to_string(),
            trigger_type: row
                .trigger_type
                .parse()
                .unwrap_or(SyncTriggerType::ManualRefresh),
            status: row.status.parse().unwrap_or(SyncBatchStatus::Failed),
            total_packages: non_negative(row.total_packages),
            completed_packages: non_negative(row.completed_packages),
            succeeded_packages: non_negative(row.succeeded_packages),
            failed_packages: non_negative(row.failed_packages),
            cancelled_packages: non_negative(row.cancelled_packages),
            deduplicated_packages: non_negative(row.deduplicated_packages),
            enqueue_failed_packages: non_negative(row.enqueue_failed_packages),
            created_at: format_timestamp(row.created_at),
            updated_at: format_timestamp(row.updated_at),
            started_at: row.started_at.map(format_timestamp),
            finished_at: row.finished_at.map(format_timestamp),
            error_message: row.error_message,
        }
    }
}

pub(super) async fn create_sync_batch(
    store: &DieselStore,
    trigger_type: SyncTriggerType,
    total_packages: u64,
) -> anyhow::Result<SyncBatch> {
    let id = Uuid::now_v7();
    let now = now_utc();
    let mut conn = store.get_connection().await?;
    diesel::insert_into(sync_batches::table)
        .values(NewSyncBatch {
            id,
            trigger_type: trigger_type.as_str(),
            status: SyncBatchStatus::Running.as_str(),
            total_packages: to_i64(total_packages),
            created_at: now,
            updated_at: now,
            started_at: Some(now),
        })
        .execute(&mut conn)
        .await?;
    drop(conn);
    refresh_sync_batch(store, id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("new sync batch {} disappeared", id))
}

pub(super) async fn get_sync_batch(
    store: &DieselStore,
    id: Uuid,
) -> anyhow::Result<Option<SyncBatch>> {
    let mut conn = store.get_connection().await?;
    load_batch(&mut conn, id).await.map_err(Into::into)
}

pub(super) async fn get_latest_sync_batch(
    store: &DieselStore,
) -> anyhow::Result<Option<SyncBatch>> {
    let mut conn = store.get_connection().await?;
    Ok(sync_batches::table
        .order(sync_batches::created_at.desc())
        .select(SyncBatchRow::as_select())
        .first(&mut conn)
        .await
        .optional()?
        .map(Into::into))
}

pub(super) async fn list_sync_batches(
    store: &DieselStore,
    limit: usize,
    offset: usize,
) -> anyhow::Result<Vec<SyncBatch>> {
    let mut conn = store.get_connection().await?;
    let rows = sync_batches::table
        .order(sync_batches::created_at.desc())
        .limit(limit as i64)
        .offset(offset as i64)
        .select(SyncBatchRow::as_select())
        .load(&mut conn)
        .await?;
    Ok(rows.into_iter().map(Into::into).collect())
}

pub(super) async fn count_sync_batches(store: &DieselStore) -> anyhow::Result<u64> {
    let mut conn = store.get_connection().await?;
    let count = sync_batches::table
        .count()
        .get_result::<i64>(&mut conn)
        .await?;
    Ok(non_negative(count))
}

pub(super) async fn list_sync_operations_for_batch(
    store: &DieselStore,
    id: Uuid,
) -> anyhow::Result<Vec<SyncOperation>> {
    let mut conn = store.get_connection().await?;
    let rows = sync_operations::table
        .filter(sync_operations::batch_id.eq(id))
        .order(sync_operations::created_at.asc())
        .select(SyncOperationRow::as_select())
        .load(&mut conn)
        .await?;
    Ok(rows.into_iter().map(Into::into).collect())
}

pub(super) async fn record_sync_batch_deduplication(
    store: &DieselStore,
    id: Uuid,
    count: u64,
) -> anyhow::Result<()> {
    let mut conn = store.get_connection().await?;
    diesel::update(sync_batches::table.find(id))
        .set((
            sync_batches::deduplicated_packages
                .eq(sync_batches::deduplicated_packages + to_i64(count)),
            sync_batches::updated_at.eq(now_utc()),
        ))
        .execute(&mut conn)
        .await?;
    drop(conn);
    refresh_sync_batch(store, id).await?;
    Ok(())
}

pub(super) async fn record_sync_batch_enqueue_failure(
    store: &DieselStore,
    id: Uuid,
    error: &str,
) -> anyhow::Result<()> {
    let mut conn = store.get_connection().await?;
    diesel::update(sync_batches::table.find(id))
        .set((
            sync_batches::enqueue_failed_packages.eq(sync_batches::enqueue_failed_packages + 1),
            sync_batches::error_message.eq(Some(error)),
            sync_batches::updated_at.eq(now_utc()),
        ))
        .execute(&mut conn)
        .await?;
    drop(conn);
    refresh_sync_batch(store, id).await?;
    Ok(())
}

pub(super) async fn refresh_sync_batch(
    store: &DieselStore,
    id: Uuid,
) -> anyhow::Result<Option<SyncBatch>> {
    let mut conn = store.get_connection().await?;
    Ok(conn
        .transaction::<Option<SyncBatch>, diesel::result::Error, _>(async |conn| {
            // Serialize aggregate refreshes for this batch. Without the row
            // lock two concurrently finishing operations could overwrite a
            // newer completed count with an older snapshot.
            let Some(batch_row) = sync_batches::table
                .find(id)
                .for_update()
                .select(SyncBatchRow::as_select())
                .first(conn)
                .await
                .optional()?
            else {
                return Ok(None);
            };
            let batch = SyncBatch::from(batch_row);
            let statuses = sync_operations::table
                .filter(sync_operations::batch_id.eq(id))
                .select(sync_operations::status)
                .load::<String>(conn)
                .await?;
            let succeeded = count_status(&statuses, &[SyncStatus::Succeeded]);
            let failed = count_status(&statuses, &[SyncStatus::Failed, SyncStatus::Interrupted])
                .saturating_add(batch.enqueue_failed_packages);
            let cancelled = count_status(&statuses, &[SyncStatus::Cancelled]);
            let completed = succeeded
                .saturating_add(failed)
                .saturating_add(cancelled)
                .saturating_add(batch.deduplicated_packages);
            let terminal = completed >= batch.total_packages;
            let status = if !terminal {
                SyncBatchStatus::Running
            } else if failed > 0 {
                SyncBatchStatus::Failed
            } else if cancelled > 0 {
                SyncBatchStatus::Cancelled
            } else {
                SyncBatchStatus::Succeeded
            };
            let now = now_utc();
            diesel::update(sync_batches::table.find(id))
                .set((
                    sync_batches::status.eq(status.as_str()),
                    sync_batches::completed_packages.eq(to_i64(completed)),
                    sync_batches::succeeded_packages.eq(to_i64(succeeded)),
                    sync_batches::failed_packages.eq(to_i64(failed)),
                    sync_batches::cancelled_packages.eq(to_i64(cancelled)),
                    sync_batches::updated_at.eq(now),
                    sync_batches::finished_at.eq(terminal.then_some(now)),
                ))
                .execute(conn)
                .await?;
            load_batch(conn, id).await
        })
        .await?)
}

pub(super) async fn refresh_active_sync_batches(store: &DieselStore) -> anyhow::Result<()> {
    let mut conn = store.get_connection().await?;
    let ids = sync_batches::table
        .filter(sync_batches::status.eq(SyncBatchStatus::Running.as_str()))
        .select(sync_batches::id)
        .load::<Uuid>(&mut conn)
        .await?;
    drop(conn);
    for id in ids {
        refresh_sync_batch(store, id).await?;
    }
    Ok(())
}

async fn load_batch(
    conn: &mut diesel_async::AsyncPgConnection,
    id: Uuid,
) -> Result<Option<SyncBatch>, diesel::result::Error> {
    Ok(sync_batches::table
        .find(id)
        .select(SyncBatchRow::as_select())
        .first(conn)
        .await
        .optional()?
        .map(Into::into))
}

fn count_status(statuses: &[String], wanted: &[SyncStatus]) -> u64 {
    statuses
        .iter()
        .filter(|status| {
            wanted
                .iter()
                .any(|wanted| status.as_str() == wanted.as_str())
        })
        .count() as u64
}

fn non_negative(value: i64) -> u64 {
    value.max(0) as u64
}

fn to_i64(value: u64) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}
