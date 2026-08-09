use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use synforge_core::{
    model::{format_timestamp, now_utc},
    sync::{SyncEventLevel, SyncOperationEvent, SyncStage},
};
use uuid::Uuid;

use super::DieselStore;
use crate::schema::sync_operation_events;

#[derive(Insertable)]
#[diesel(table_name = sync_operation_events)]
struct NewSyncOperationEvent<'a> {
    id: Uuid,
    sync_operation_id: Uuid,
    stage: &'a str,
    level: &'a str,
    message: &'a str,
    created_at: time::OffsetDateTime,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = sync_operation_events)]
struct SyncOperationEventRow {
    id: Uuid,
    sync_operation_id: Uuid,
    stage: String,
    level: String,
    message: String,
    created_at: time::OffsetDateTime,
}

impl From<SyncOperationEventRow> for SyncOperationEvent {
    fn from(row: SyncOperationEventRow) -> Self {
        Self {
            id: row.id.to_string(),
            sync_operation_id: row.sync_operation_id.to_string(),
            stage: row.stage.parse().unwrap_or(SyncStage::Completed),
            level: row.level.parse().unwrap_or(SyncEventLevel::Info),
            message: row.message,
            created_at: format_timestamp(row.created_at),
        }
    }
}

pub(super) async fn list_sync_operation_events(
    store: &DieselStore,
    id: Uuid,
) -> anyhow::Result<Vec<SyncOperationEvent>> {
    let mut conn = store.get_connection().await?;
    let rows = sync_operation_events::table
        .filter(sync_operation_events::sync_operation_id.eq(id))
        .order(sync_operation_events::created_at.asc())
        .select(SyncOperationEventRow::as_select())
        .load(&mut conn)
        .await?;
    Ok(rows.into_iter().map(Into::into).collect())
}

pub(super) async fn append_sync_operation_event(
    store: &DieselStore,
    id: Uuid,
    stage: SyncStage,
    level: SyncEventLevel,
    message: &str,
) -> anyhow::Result<()> {
    let mut conn = store.get_connection().await?;
    insert_event(&mut conn, id, stage, level, message, now_utc()).await?;
    Ok(())
}

pub(super) async fn insert_event(
    conn: &mut AsyncPgConnection,
    operation_id: Uuid,
    stage: SyncStage,
    level: SyncEventLevel,
    message: &str,
    created_at: time::OffsetDateTime,
) -> Result<(), diesel::result::Error> {
    diesel::insert_into(sync_operation_events::table)
        .values(NewSyncOperationEvent {
            id: Uuid::now_v7(),
            sync_operation_id: operation_id,
            stage: stage.as_str(),
            level: level.as_str(),
            message,
            created_at,
        })
        .execute(conn)
        .await?;
    Ok(())
}
