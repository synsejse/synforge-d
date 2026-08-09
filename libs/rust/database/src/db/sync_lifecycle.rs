use diesel::prelude::*;
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
use synforge_core::{
    model::now_utc,
    sync::{SyncEventLevel, SyncOperation, SyncStage, SyncStatus},
};
use uuid::Uuid;

use super::{
    DieselStore,
    sync::SyncOperationRow,
    sync_events::insert_event,
    traits::{EnqueuedSyncRun, NewSyncRun, SyncRunCompletion},
};
use crate::schema::sync_operations;

#[derive(Insertable)]
#[diesel(table_name = sync_operations)]
struct QueuedSyncOperation<'a> {
    id: Uuid,
    package_name: &'a str,
    trigger_type: &'a str,
    status: &'a str,
    stage: &'a str,
    target_mock_chroot: Option<&'a str>,
    batch_id: Option<Uuid>,
    retry_of: Option<Uuid>,
    created_at: time::OffsetDateTime,
    updated_at: time::OffsetDateTime,
}

#[derive(AsChangeset)]
#[diesel(table_name = sync_operations, treat_none_as_null = true)]
struct CompletedSyncOperation<'a> {
    status: &'a str,
    stage: &'a str,
    revision: Option<&'a str>,
    previous_revision: Option<&'a str>,
    changed: Option<bool>,
    queued_targets: i64,
    skipped_targets: i64,
    blocked_targets: i64,
    error_message: Option<&'a str>,
    updated_at: time::OffsetDateTime,
    finished_at: Option<time::OffsetDateTime>,
}

pub(super) async fn enqueue_sync_run(
    store: &DieselStore,
    request: NewSyncRun,
) -> anyhow::Result<EnqueuedSyncRun> {
    let id = Uuid::now_v7();
    let now = now_utc();
    let trigger = request.trigger_type.as_str().to_string();
    let package_name = request.package_name;
    let target_mock_chroot = request.target_mock_chroot;
    let mut conn = store.get_connection().await?;
    let created = conn
        .transaction::<bool, diesel::result::Error, _>(async |conn| {
            let row = QueuedSyncOperation {
                id,
                package_name: &package_name,
                trigger_type: &trigger,
                status: SyncStatus::Queued.as_str(),
                stage: SyncStage::Queued.as_str(),
                target_mock_chroot: target_mock_chroot.as_deref(),
                batch_id: request.batch_id,
                retry_of: request.retry_of,
                created_at: now,
                updated_at: now,
            };
            let inserted = diesel::insert_into(sync_operations::table)
                .values(&row)
                .on_conflict_do_nothing()
                .execute(conn)
                .await?;
            if inserted == 1 {
                insert_event(
                    conn,
                    id,
                    SyncStage::Queued,
                    SyncEventLevel::Info,
                    "Sync queued",
                    now,
                )
                .await?;
            }
            Ok(inserted == 1)
        })
        .await?;

    let operation = if created {
        load_operation(&mut conn, id).await?
    } else {
        sync_operations::table
            .filter(sync_operations::package_name.eq(&package_name))
            .filter(
                sync_operations::status
                    .eq_any([SyncStatus::Queued.as_str(), SyncStatus::Running.as_str()]),
            )
            .order(sync_operations::created_at.desc())
            .select(SyncOperationRow::as_select())
            .first(&mut conn)
            .await
            .optional()?
            .map(SyncOperation::from)
    }
    .ok_or_else(|| anyhow::anyhow!("sync enqueue conflict did not leave an active operation"))?;

    Ok(EnqueuedSyncRun { operation, created })
}

pub(super) async fn get_sync_operation(
    store: &DieselStore,
    id: Uuid,
) -> anyhow::Result<Option<SyncOperation>> {
    let mut conn = store.get_connection().await?;
    load_operation(&mut conn, id).await.map_err(Into::into)
}

pub(super) async fn claim_sync_run(store: &DieselStore, id: Uuid) -> anyhow::Result<bool> {
    let now = now_utc();
    let mut conn = store.get_connection().await?;
    let claimed = conn
        .transaction::<bool, diesel::result::Error, _>(async |conn| {
            let updated = diesel::update(
                sync_operations::table
                    .find(id)
                    .filter(sync_operations::status.eq(SyncStatus::Queued.as_str())),
            )
            .set((
                sync_operations::status.eq(SyncStatus::Running.as_str()),
                sync_operations::stage.eq(SyncStage::InspectingSource.as_str()),
                sync_operations::started_at.eq(Some(now)),
                sync_operations::updated_at.eq(now),
            ))
            .execute(conn)
            .await?;
            if updated == 1 {
                insert_event(
                    conn,
                    id,
                    SyncStage::InspectingSource,
                    SyncEventLevel::Info,
                    "Inspecting package source",
                    now,
                )
                .await?;
            }
            Ok(updated == 1)
        })
        .await?;
    Ok(claimed)
}

pub(super) async fn advance_sync_run(
    store: &DieselStore,
    id: Uuid,
    stage: SyncStage,
    message: &str,
) -> anyhow::Result<bool> {
    let now = now_utc();
    let message = message.to_string();
    let mut conn = store.get_connection().await?;
    Ok(conn
        .transaction::<bool, diesel::result::Error, _>(async |conn| {
            let updated = diesel::update(
                sync_operations::table
                    .find(id)
                    .filter(sync_operations::status.eq(SyncStatus::Running.as_str()))
                    .filter(sync_operations::cancellation_requested.eq(false)),
            )
            .set((
                sync_operations::stage.eq(stage.as_str()),
                sync_operations::updated_at.eq(now),
            ))
            .execute(conn)
            .await?;
            if updated == 1 {
                insert_event(conn, id, stage, SyncEventLevel::Info, &message, now).await?;
            }
            Ok(updated == 1)
        })
        .await?)
}

pub(super) async fn finish_sync_run(
    store: &DieselStore,
    id: Uuid,
    completion: SyncRunCompletion,
) -> anyhow::Result<bool> {
    if !completion.status.is_terminal() {
        anyhow::bail!("sync completion status must be terminal");
    }
    let now = now_utc();
    let status = completion.status.as_str().to_string();
    let allow_requested_cancellation = completion.status == SyncStatus::Cancelled;
    let message = completion_message(&completion);
    let level = if completion.status == SyncStatus::Succeeded {
        SyncEventLevel::Info
    } else if completion.status == SyncStatus::Cancelled {
        SyncEventLevel::Warning
    } else {
        SyncEventLevel::Error
    };
    let mut conn = store.get_connection().await?;
    Ok(conn
        .transaction::<bool, diesel::result::Error, _>(async |conn| {
            let changes = CompletedSyncOperation {
                status: &status,
                stage: SyncStage::Completed.as_str(),
                revision: completion.revision.as_deref(),
                previous_revision: completion.previous_revision.as_deref(),
                changed: completion.changed,
                queued_targets: to_i64(completion.queued_targets),
                skipped_targets: to_i64(completion.skipped_targets),
                blocked_targets: to_i64(completion.blocked_targets),
                error_message: completion.error_message.as_deref(),
                updated_at: now,
                finished_at: Some(now),
            };
            let updated = if allow_requested_cancellation {
                diesel::update(
                    sync_operations::table.find(id).filter(
                        sync_operations::status
                            .eq_any([SyncStatus::Queued.as_str(), SyncStatus::Running.as_str()]),
                    ),
                )
                .set(changes)
                .execute(conn)
                .await?
            } else {
                diesel::update(
                    sync_operations::table
                        .find(id)
                        .filter(
                            sync_operations::status.eq_any([
                                SyncStatus::Queued.as_str(),
                                SyncStatus::Running.as_str(),
                            ]),
                        )
                        .filter(sync_operations::cancellation_requested.eq(false)),
                )
                .set(changes)
                .execute(conn)
                .await?
            };
            if updated == 1 {
                insert_event(conn, id, SyncStage::Completed, level, &message, now).await?;
            }
            Ok(updated == 1)
        })
        .await?)
}

pub(super) async fn request_sync_cancellation(
    store: &DieselStore,
    id: Uuid,
) -> anyhow::Result<Option<SyncOperation>> {
    let now = now_utc();
    let mut conn = store.get_connection().await?;
    conn.transaction::<(), diesel::result::Error, _>(async |conn| {
        let operation_state = sync_operations::table
            .find(id)
            .for_update()
            .select((sync_operations::status, sync_operations::stage))
            .first::<(String, String)>(conn)
            .await
            .optional()?;
        match operation_state.as_ref() {
            Some((status, _)) if status == SyncStatus::Queued.as_str() => {
                diesel::update(sync_operations::table.find(id))
                    .set((
                        sync_operations::status.eq(SyncStatus::Cancelled.as_str()),
                        sync_operations::stage.eq(SyncStage::Completed.as_str()),
                        sync_operations::cancellation_requested.eq(true),
                        sync_operations::updated_at.eq(now),
                        sync_operations::finished_at.eq(Some(now)),
                    ))
                    .execute(conn)
                    .await?;
                insert_event(
                    conn,
                    id,
                    SyncStage::Completed,
                    SyncEventLevel::Warning,
                    "Sync cancelled before it started",
                    now,
                )
                .await?;
            }
            Some((status, stage)) if status == SyncStatus::Running.as_str() => {
                diesel::update(sync_operations::table.find(id))
                    .set((
                        sync_operations::cancellation_requested.eq(true),
                        sync_operations::updated_at.eq(now),
                    ))
                    .execute(conn)
                    .await?;
                insert_event(
                    conn,
                    id,
                    stage.parse().unwrap_or(SyncStage::InspectingSource),
                    SyncEventLevel::Warning,
                    "Cancellation requested",
                    now,
                )
                .await?;
            }
            _ => {}
        }
        Ok(())
    })
    .await?;
    load_operation(&mut conn, id).await.map_err(Into::into)
}

pub(super) async fn list_queued_sync_run_ids(store: &DieselStore) -> anyhow::Result<Vec<Uuid>> {
    let mut conn = store.get_connection().await?;
    Ok(sync_operations::table
        .filter(sync_operations::status.eq(SyncStatus::Queued.as_str()))
        .order(sync_operations::created_at.asc())
        .select(sync_operations::id)
        .load(&mut conn)
        .await?)
}

pub(super) async fn interrupt_running_sync_runs(
    store: &DieselStore,
    message: &str,
) -> anyhow::Result<u64> {
    let now = now_utc();
    let message = message.to_string();
    let mut conn = store.get_connection().await?;
    let count = conn
        .transaction::<usize, diesel::result::Error, _>(async |conn| {
            let ids = sync_operations::table
                .filter(sync_operations::status.eq(SyncStatus::Running.as_str()))
                .select(sync_operations::id)
                .load::<Uuid>(conn)
                .await?;
            if ids.is_empty() {
                return Ok(0);
            }
            diesel::update(sync_operations::table.filter(sync_operations::id.eq_any(&ids)))
                .set((
                    sync_operations::status.eq(SyncStatus::Interrupted.as_str()),
                    sync_operations::stage.eq(SyncStage::Completed.as_str()),
                    sync_operations::error_message.eq(Some(message.as_str())),
                    sync_operations::updated_at.eq(now),
                    sync_operations::finished_at.eq(Some(now)),
                ))
                .execute(conn)
                .await?;
            for id in &ids {
                insert_event(
                    conn,
                    *id,
                    SyncStage::Completed,
                    SyncEventLevel::Error,
                    &message,
                    now,
                )
                .await?;
            }
            Ok(ids.len())
        })
        .await?;
    Ok(count as u64)
}

async fn load_operation(
    conn: &mut AsyncPgConnection,
    id: Uuid,
) -> Result<Option<SyncOperation>, diesel::result::Error> {
    Ok(sync_operations::table
        .find(id)
        .select(SyncOperationRow::as_select())
        .first(conn)
        .await
        .optional()?
        .map(Into::into))
}

fn completion_message(completion: &SyncRunCompletion) -> String {
    match (&completion.status, &completion.error_message) {
        (_, Some(error)) => error.clone(),
        (SyncStatus::Succeeded, None) => format!(
            "Sync completed: {} queued, {} skipped, {} blocked",
            completion.queued_targets, completion.skipped_targets, completion.blocked_targets
        ),
        (status, None) => format!("Sync {}", status.as_str()),
    }
}

fn to_i64(value: u64) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}
