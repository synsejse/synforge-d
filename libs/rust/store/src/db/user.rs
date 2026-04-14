use super::*;

pub(super) async fn user_count(store: &DieselStore) -> anyhow::Result<u64> {
    store
        .with_connection(|conn| Ok(users::table.count().get_result::<i64>(conn)? as u64))
        .await
}

pub(super) async fn list_users(store: &DieselStore) -> anyhow::Result<Vec<UserSummary>> {
    store
        .with_connection(|conn| {
            let rows = users::table
                .order(users::handle.asc())
                .select(UserRecord::as_select())
                .load(conn)?;
            build_user_summaries(conn, rows)
        })
        .await
}

pub(super) async fn get_user(
    store: &DieselStore,
    user_id: Uuid,
) -> anyhow::Result<Option<UserSummary>> {
    let user_id = user_id.to_string();
    store
        .with_connection(move |conn| {
            let row = users::table
                .find(user_id.as_str())
                .select(UserRecord::as_select())
                .first(conn)
                .optional()?;
            Ok(row
                .map(|row| build_user_summaries(conn, vec![row]))
                .transpose()?
                .map(|mut users| users.pop().expect("single user summary")))
        })
        .await
}

pub(super) async fn get_user_by_handle(
    store: &DieselStore,
    handle: &str,
) -> anyhow::Result<Option<UserSummary>> {
    let handle = handle.to_string();
    store
        .with_connection(move |conn| {
            let row = users::table
                .filter(users::handle.eq(handle.as_str()))
                .select(UserRecord::as_select())
                .first(conn)
                .optional()?;
            Ok(row
                .map(|row| build_user_summaries(conn, vec![row]))
                .transpose()?
                .map(|mut users| users.pop().expect("single user summary")))
        })
        .await
}

pub(super) async fn get_user_auth_by_handle(
    store: &DieselStore,
    handle: &str,
) -> anyhow::Result<Option<UserAuthRecord>> {
    let handle = handle.to_string();
    store
        .with_connection(move |conn| {
            let row = users::table
                .filter(users::handle.eq(handle.as_str()))
                .select(UserRecord::as_select())
                .first(conn)
                .optional()?;
            let Some(row) = row else {
                return Ok(None);
            };
            let permissions = load_permissions_map(conn, std::slice::from_ref(&row.id))?
                .remove(&row.id)
                .unwrap_or_default();
            Ok(Some(UserAuthRecord {
                password_hash: row.password_hash.clone(),
                user: user_from_record(row, permissions)?,
            }))
        })
        .await
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
    store
        .with_connection(move |conn| {
            conn.transaction::<UserSummary, anyhow::Error, _>(|conn| {
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
                    .execute(conn)?;
                replace_permissions(conn, user_id.as_str(), &permissions)?;
                let metrics = NewUserRepoMetricsRecord {
                    user_id: user_id.as_str(),
                    downloaded_bytes: 0,
                    updated_at: now.as_str(),
                };
                diesel::insert_into(user_repo_metrics::table)
                    .values(&metrics)
                    .execute(conn)?;
                load_user_summary(conn, user_id.as_str())
            })
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
    store
        .with_connection(move |conn| {
            conn.transaction::<Option<UserSummary>, anyhow::Error, _>(|conn| {
                let exists = users::table
                    .find(user_id.as_str())
                    .select(users::id)
                    .first::<String>(conn)
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
                    .execute(conn)?;
                replace_permissions(conn, user_id.as_str(), &permissions)?;
                load_user_summary(conn, user_id.as_str()).map(Some)
            })
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
    store
        .with_connection(move |conn| {
            Ok(diesel::update(users::table.find(user_id.as_str()))
                .set((
                    users::password_hash.eq(password_hash.as_str()),
                    users::updated_at.eq(now.as_str()),
                ))
                .execute(conn)?
                > 0)
        })
        .await
}

pub(super) async fn delete_user(
    store: &DieselStore,
    user_id: Uuid,
) -> anyhow::Result<Option<UserSummary>> {
    let user_id = user_id.to_string();
    store
        .with_connection(move |conn| {
            conn.transaction::<Option<UserSummary>, anyhow::Error, _>(|conn| {
                let exists = users::table
                    .find(user_id.as_str())
                    .select(users::id)
                    .first::<String>(conn)
                    .optional()?;
                let Some(_) = exists else {
                    return Ok(None);
                };
                let existing = load_user_summary(conn, user_id.as_str())?;
                diesel::delete(
                    user_permissions::table.filter(user_permissions::user_id.eq(user_id.as_str())),
                )
                .execute(conn)?;
                diesel::delete(
                    user_repo_metrics::table
                        .filter(user_repo_metrics::user_id.eq(user_id.as_str())),
                )
                .execute(conn)?;
                diesel::delete(users::table.find(user_id.as_str())).execute(conn)?;
                Ok(Some(existing))
            })
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
    store
        .with_connection(move |conn| {
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
                    user_repo_metrics::downloaded_bytes
                        .eq(user_repo_metrics::downloaded_bytes + bytes),
                    user_repo_metrics::updated_at.eq(now.as_str()),
                ))
                .execute(conn)?;
            Ok(())
        })
        .await
}

fn build_user_summaries(
    conn: &mut MysqlConnection,
    rows: Vec<UserRecord>,
) -> anyhow::Result<Vec<UserSummary>> {
    let ids = rows.iter().map(|row| row.id.clone()).collect::<Vec<_>>();
    let mut permissions = load_permissions_map(conn, &ids)?;
    let mut metrics = load_metrics_map(conn, &ids)?;

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

fn load_user_summary(conn: &mut MysqlConnection, user_id: &str) -> anyhow::Result<UserSummary> {
    let row = users::table
        .find(user_id)
        .select(UserRecord::as_select())
        .first(conn)?;
    let mut users = build_user_summaries(conn, vec![row])?;
    Ok(users.pop().expect("single user summary"))
}

fn load_permissions_map(
    conn: &mut MysqlConnection,
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
        .load::<UserPermissionRecord>(conn)?;
    let mut map = HashMap::<String, Vec<UserPermission>>::new();
    for row in rows {
        map.entry(row.user_id).or_default().push(row.permission);
    }
    Ok(map)
}

fn load_metrics_map(
    conn: &mut MysqlConnection,
    user_ids: &[String],
) -> anyhow::Result<HashMap<String, UserRepoMetrics>> {
    if user_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let rows = user_repo_metrics::table
        .filter(user_repo_metrics::user_id.eq_any(user_ids))
        .select(UserRepoMetricsRecord::as_select())
        .load::<UserRepoMetricsRecord>(conn)?;
    let mut map = HashMap::<String, UserRepoMetrics>::new();
    for row in rows {
        map.insert(row.user_id.clone(), user_metrics_from_record(row)?);
    }
    Ok(map)
}

fn replace_permissions(
    conn: &mut MysqlConnection,
    user_id: &str,
    permissions: &[UserPermission],
) -> Result<(), diesel::result::Error> {
    diesel::delete(user_permissions::table.filter(user_permissions::user_id.eq(user_id)))
        .execute(conn)?;
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
        .execute(conn)?;
    Ok(())
}
