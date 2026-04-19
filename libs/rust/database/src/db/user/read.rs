use super::*;
use diesel_async::RunQueryDsl;

pub(in crate::db) async fn user_count(store: &DieselStore) -> anyhow::Result<u64> {
    let mut conn = store.get_connection().await?;
    Ok(users::table.count().get_result::<i64>(&mut conn).await? as u64)
}

pub(in crate::db) async fn list_users(store: &DieselStore) -> anyhow::Result<Vec<UserSummary>> {
    let mut conn = store.get_connection().await?;
    let rows = users::table
        .order(users::handle.asc())
        .select(UserRecord::as_select())
        .load(&mut conn)
        .await?;
    build_user_summaries(&mut conn, rows).await
}

pub(in crate::db) async fn get_user(
    store: &DieselStore,
    user_id: Uuid,
) -> anyhow::Result<Option<UserSummary>> {
    let mut conn = store.get_connection().await?;
    let row = users::table
        .find(user_id)
        .select(UserRecord::as_select())
        .first(&mut conn)
        .await
        .optional()?;
    match row {
        Some(row) => {
            let mut users = build_user_summaries(&mut conn, vec![row]).await?;
            Ok(users.pop())
        }
        None => Ok(None),
    }
}

pub(in crate::db) async fn get_user_by_handle(
    store: &DieselStore,
    handle: &str,
) -> anyhow::Result<Option<UserSummary>> {
    let handle = handle.to_string();
    let mut conn = store.get_connection().await?;
    let row = users::table
        .filter(users::handle.eq(handle.as_str()))
        .select(UserRecord::as_select())
        .first(&mut conn)
        .await
        .optional()?;
    match row {
        Some(row) => {
            let mut users = build_user_summaries(&mut conn, vec![row]).await?;
            Ok(users.pop())
        }
        None => Ok(None),
    }
}

pub(in crate::db) async fn get_user_auth_by_handle(
    store: &DieselStore,
    handle: &str,
) -> anyhow::Result<Option<UserAuthRecord>> {
    let handle = handle.to_string();
    let mut conn = store.get_connection().await?;
    let row = users::table
        .filter(users::handle.eq(handle.as_str()))
        .select(UserRecord::as_select())
        .first(&mut conn)
        .await
        .optional()?;
    let Some(row) = row else {
        return Ok(None);
    };
    let permissions = load_permissions_map(&mut conn, std::slice::from_ref(&row.id))
        .await?
        .remove(&row.id)
        .unwrap_or_default();
    Ok(Some(UserAuthRecord {
        password_hash: row.password_hash.clone(),
        user: user_from_record(row, permissions)?,
    }))
}
