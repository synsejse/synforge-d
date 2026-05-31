use super::*;
use diesel_async::{AsyncConnection, RunQueryDsl, scoped_futures::ScopedFutureExt};

pub(in crate::db) async fn create_user(
    store: &DieselStore,
    handle: &str,
    display_name: &str,
    password_hash: &str,
    active: bool,
    permissions: &[UserPermission],
) -> anyhow::Result<UserSummary> {
    let user_id = Uuid::now_v7();
    let handle = handle.to_string();
    let display_name = display_name.to_string();
    let password_hash = password_hash.to_string();
    let permissions = permissions.to_vec();
    let now = now_utc();
    let mut conn = store.get_connection().await?;
    conn.transaction::<UserSummary, anyhow::Error, _>(|conn| {
        async move {
            let user = NewUserRecord {
                id: user_id,
                handle: handle.as_str(),
                display_name: display_name.as_str(),
                password_hash: password_hash.as_str(),
                active,
                created_at: now,
                updated_at: now,
            };
            diesel::insert_into(users::table)
                .values(&user)
                .execute(conn)
                .await?;
            replace_permissions(conn, user_id, &permissions).await?;
            let metrics = NewUserRepoMetricsRecord {
                user_id,
                downloaded_bytes: 0,
                updated_at: now,
            };
            diesel::insert_into(user_repo_metrics::table)
                .values(&metrics)
                .execute(conn)
                .await?;
            load_user_summary(conn, user_id).await
        }
        .scope_boxed()
    })
    .await
}

pub(in crate::db) async fn update_user(
    store: &DieselStore,
    user_id: Uuid,
    handle: &str,
    display_name: &str,
    active: bool,
    permissions: &[UserPermission],
) -> anyhow::Result<Option<UserSummary>> {
    let handle = handle.to_string();
    let display_name = display_name.to_string();
    let permissions = permissions.to_vec();
    let now = now_utc();
    let mut conn = store.get_connection().await?;
    conn.transaction::<Option<UserSummary>, anyhow::Error, _>(|conn| {
        async move {
            let exists = users::table
                .find(user_id)
                .select(users::id)
                .first::<Uuid>(conn)
                .await
                .optional()?;
            if exists.is_none() {
                return Ok(None);
            }
            diesel::update(users::table.find(user_id))
                .set((
                    users::handle.eq(handle.as_str()),
                    users::display_name.eq(display_name.as_str()),
                    users::active.eq(active),
                    users::updated_at.eq(now),
                ))
                .execute(conn)
                .await?;
            replace_permissions(conn, user_id, &permissions).await?;
            load_user_summary(conn, user_id).await.map(Some)
        }
        .scope_boxed()
    })
    .await
}

pub(in crate::db) async fn update_user_password(
    store: &DieselStore,
    user_id: Uuid,
    password_hash: &str,
) -> anyhow::Result<bool> {
    let password_hash = password_hash.to_string();
    let now = now_utc();
    let mut conn = store.get_connection().await?;
    Ok(diesel::update(users::table.find(user_id))
        .set((
            users::password_hash.eq(password_hash.as_str()),
            users::updated_at.eq(now),
        ))
        .execute(&mut conn)
        .await?
        > 0)
}

pub(in crate::db) async fn delete_user(
    store: &DieselStore,
    user_id: Uuid,
) -> anyhow::Result<Option<UserSummary>> {
    let mut conn = store.get_connection().await?;
    conn.transaction::<Option<UserSummary>, anyhow::Error, _>(|conn| {
        async move {
            let exists = users::table
                .find(user_id)
                .select(users::id)
                .first::<Uuid>(conn)
                .await
                .optional()?;
            let Some(_) = exists else {
                return Ok(None);
            };
            let existing = load_user_summary(conn, user_id).await?;
            diesel::delete(user_permissions::table.filter(user_permissions::user_id.eq(user_id)))
                .execute(conn)
                .await?;
            diesel::delete(user_repo_metrics::table.filter(user_repo_metrics::user_id.eq(user_id)))
                .execute(conn)
                .await?;
            diesel::delete(users::table.find(user_id))
                .execute(conn)
                .await?;
            Ok(Some(existing))
        }
        .scope_boxed()
    })
    .await
}

pub(in crate::db) async fn increment_user_download_bytes(
    store: &DieselStore,
    user_id: Uuid,
    bytes: u64,
) -> anyhow::Result<()> {
    let bytes = (bytes.min(i64::MAX as u64)) as i64;
    let now = now_utc();
    let mut conn = store.get_connection().await?;
    let row = NewUserRepoMetricsRecord {
        user_id,
        downloaded_bytes: bytes,
        updated_at: now,
    };
    diesel::insert_into(user_repo_metrics::table)
        .values(&row)
        .on_conflict(user_repo_metrics::user_id)
        .do_update()
        .set((
            user_repo_metrics::downloaded_bytes.eq(user_repo_metrics::downloaded_bytes + bytes),
            user_repo_metrics::updated_at.eq(now),
        ))
        .execute(&mut conn)
        .await?;
    Ok(())
}
