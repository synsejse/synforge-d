use super::*;
use diesel_async::{AsyncConnection, RunQueryDsl, scoped_futures::ScopedFutureExt};

pub(super) async fn user_count(store: &DieselStore) -> anyhow::Result<u64> {
    let mut conn = store.get_connection().await?;
    Ok(users::table.count().get_result::<i64>(&mut conn).await? as u64)
}

pub(super) async fn list_users(store: &DieselStore) -> anyhow::Result<Vec<UserSummary>> {
    let mut conn = store.get_connection().await?;
    let rows = users::table
        .order(users::handle.asc())
        .select(UserRecord::as_select())
        .load(&mut conn)
        .await?;
    build_user_summaries(&mut conn, rows).await
}

pub(super) async fn get_user(
    store: &DieselStore,
    user_id: Uuid,
) -> anyhow::Result<Option<UserSummary>> {
    let user_id = user_id.to_string();
    let mut conn = store.get_connection().await?;
    let row = users::table
        .find(user_id.as_str())
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

pub(super) async fn get_user_by_handle(
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

pub(super) async fn get_user_auth_by_handle(
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

pub(super) async fn create_user(
    store: &DieselStore,
    handle: &str,
    display_name: &str,
    password_hash: &str,
    active: bool,
    permissions: &[UserPermission],
) -> anyhow::Result<UserSummary> {
    let user_id = Uuid::now_v7().to_string();
    let handle = handle.to_string();
    let display_name = display_name.to_string();
    let password_hash = password_hash.to_string();
    let permissions = permissions.to_vec();
    let now = format_timestamp(now_utc());
    let mut conn = store.get_connection().await?;
    conn.transaction::<UserSummary, anyhow::Error, _>(|conn| {
        async move {
            let user = NewUserRecord {
                id: user_id.as_str(),
                handle: handle.as_str(),
                display_name: display_name.as_str(),
                password_hash: password_hash.as_str(),
                active,
                created_at: now.as_str(),
                updated_at: now.as_str(),
            };
            diesel::insert_into(users::table)
                .values(&user)
                .execute(conn)
                .await?;
            replace_permissions(conn, user_id.as_str(), &permissions).await?;
            let metrics = NewUserRepoMetricsRecord {
                user_id: user_id.as_str(),
                downloaded_bytes: 0,
                updated_at: now.as_str(),
            };
            diesel::insert_into(user_repo_metrics::table)
                .values(&metrics)
                .execute(conn)
                .await?;
            load_user_summary(conn, user_id.as_str()).await
        }
        .scope_boxed()
    })
    .await
}

pub(super) async fn update_user(
    store: &DieselStore,
    user_id: Uuid,
    handle: &str,
    display_name: &str,
    active: bool,
    permissions: &[UserPermission],
) -> anyhow::Result<Option<UserSummary>> {
    let user_id = user_id.to_string();
    let handle = handle.to_string();
    let display_name = display_name.to_string();
    let permissions = permissions.to_vec();
    let now = format_timestamp(now_utc());
    let mut conn = store.get_connection().await?;
    conn.transaction::<Option<UserSummary>, anyhow::Error, _>(|conn| {
        async move {
            let exists = users::table
                .find(user_id.as_str())
                .select(users::id)
                .first::<String>(conn)
                .await
                .optional()?;
            if exists.is_none() {
                return Ok(None);
            }
            diesel::update(users::table.find(user_id.as_str()))
                .set((
                    users::handle.eq(handle.as_str()),
                    users::display_name.eq(display_name.as_str()),
                    users::active.eq(active),
                    users::updated_at.eq(now.as_str()),
                ))
                .execute(conn)
                .await?;
            replace_permissions(conn, user_id.as_str(), &permissions).await?;
            load_user_summary(conn, user_id.as_str()).await.map(Some)
        }
        .scope_boxed()
    })
    .await
}

pub(super) async fn update_user_password(
    store: &DieselStore,
    user_id: Uuid,
    password_hash: &str,
) -> anyhow::Result<bool> {
    let user_id = user_id.to_string();
    let password_hash = password_hash.to_string();
    let now = format_timestamp(now_utc());
    let mut conn = store.get_connection().await?;
    Ok(diesel::update(users::table.find(user_id.as_str()))
        .set((
            users::password_hash.eq(password_hash.as_str()),
            users::updated_at.eq(now.as_str()),
        ))
        .execute(&mut conn)
        .await?
        > 0)
}

pub(super) async fn delete_user(
    store: &DieselStore,
    user_id: Uuid,
) -> anyhow::Result<Option<UserSummary>> {
    let user_id = user_id.to_string();
    let mut conn = store.get_connection().await?;
    conn.transaction::<Option<UserSummary>, anyhow::Error, _>(|conn| {
        async move {
            let exists = users::table
                .find(user_id.as_str())
                .select(users::id)
                .first::<String>(conn)
                .await
                .optional()?;
            let Some(_) = exists else {
                return Ok(None);
            };
            let existing = load_user_summary(conn, user_id.as_str()).await?;
            diesel::delete(
                user_permissions::table.filter(user_permissions::user_id.eq(user_id.as_str())),
            )
            .execute(conn)
            .await?;
            diesel::delete(
                user_repo_metrics::table.filter(user_repo_metrics::user_id.eq(user_id.as_str())),
            )
            .execute(conn)
            .await?;
            diesel::delete(users::table.find(user_id.as_str()))
                .execute(conn)
                .await?;
            Ok(Some(existing))
        }
        .scope_boxed()
    })
    .await
}

pub(super) async fn increment_user_download_bytes(
    store: &DieselStore,
    user_id: Uuid,
    bytes: u64,
) -> anyhow::Result<()> {
    let user_id = user_id.to_string();
    let bytes = (bytes.min(i64::MAX as u64)) as i64;
    let now = format_timestamp(now_utc());
    let mut conn = store.get_connection().await?;
    let row = NewUserRepoMetricsRecord {
        user_id: user_id.as_str(),
        downloaded_bytes: bytes,
        updated_at: now.as_str(),
    };
    diesel::insert_into(user_repo_metrics::table)
        .values(&row)
        .on_conflict(diesel::dsl::DuplicatedKeys)
        .do_update()
        .set((
            user_repo_metrics::downloaded_bytes.eq(user_repo_metrics::downloaded_bytes + bytes),
            user_repo_metrics::updated_at.eq(now.as_str()),
        ))
        .execute(&mut conn)
        .await?;
    Ok(())
}

async fn build_user_summaries(
    conn: &mut AsyncMysqlConnection,
    rows: Vec<UserRecord>,
) -> anyhow::Result<Vec<UserSummary>> {
    let ids = rows.iter().map(|row| row.id.clone()).collect::<Vec<_>>();
    let mut permissions = load_permissions_map(conn, &ids).await?;
    let mut metrics = load_metrics_map(conn, &ids).await?;

    rows.into_iter()
        .map(|row| {
            let user_id = row.id.clone();
            let fallback_updated_at = row.updated_at.clone();
            let user = user_from_record(row, permissions.remove(&user_id).unwrap_or_default())?;
            let metrics = metrics.remove(&user_id).unwrap_or(UserRepoMetrics {
                user_id: user.id,
                downloaded_bytes: 0,
                updated_at: parse_timestamp(&fallback_updated_at)?,
            });
            Ok(UserSummary { user, metrics })
        })
        .collect()
}

async fn load_user_summary(
    conn: &mut AsyncMysqlConnection,
    user_id: &str,
) -> anyhow::Result<UserSummary> {
    let row = users::table
        .find(user_id)
        .select(UserRecord::as_select())
        .first(conn)
        .await?;
    let mut users = build_user_summaries(conn, vec![row]).await?;
    Ok(users.pop().expect("single user summary"))
}

async fn load_permissions_map(
    conn: &mut AsyncMysqlConnection,
    user_ids: &[String],
) -> anyhow::Result<HashMap<String, Vec<UserPermission>>> {
    if user_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let rows = user_permissions::table
        .filter(user_permissions::user_id.eq_any(user_ids))
        .order((
            user_permissions::user_id.asc(),
            user_permissions::permission.asc(),
        ))
        .select(UserPermissionRecord::as_select())
        .load::<UserPermissionRecord>(conn)
        .await?;
    let mut map = HashMap::<String, Vec<UserPermission>>::new();
    for row in rows {
        map.entry(row.user_id).or_default().push(row.permission);
    }
    Ok(map)
}

async fn load_metrics_map(
    conn: &mut AsyncMysqlConnection,
    user_ids: &[String],
) -> anyhow::Result<HashMap<String, UserRepoMetrics>> {
    if user_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let rows = user_repo_metrics::table
        .filter(user_repo_metrics::user_id.eq_any(user_ids))
        .select(UserRepoMetricsRecord::as_select())
        .load::<UserRepoMetricsRecord>(conn)
        .await?;
    let mut map = HashMap::<String, UserRepoMetrics>::new();
    for row in rows {
        map.insert(row.user_id.clone(), user_metrics_from_record(row)?);
    }
    Ok(map)
}

async fn replace_permissions(
    conn: &mut AsyncMysqlConnection,
    user_id: &str,
    permissions: &[UserPermission],
) -> anyhow::Result<()> {
    diesel::delete(user_permissions::table.filter(user_permissions::user_id.eq(user_id)))
        .execute(conn)
        .await?;
    let mut values = permissions.to_vec();
    values.sort_unstable();
    values.dedup();
    if values.is_empty() {
        return Ok(());
    }
    let rows = values
        .iter()
        .map(|permission| NewUserPermissionRecord {
            user_id,
            permission: *permission,
        })
        .collect::<Vec<_>>();
    diesel::insert_into(user_permissions::table)
        .values(&rows)
        .execute(conn)
        .await?;
    Ok(())
}
