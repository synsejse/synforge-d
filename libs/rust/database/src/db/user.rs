#[path = "user/read.rs"]
mod read;
#[path = "user/write.rs"]
mod write;

pub(super) use read::*;
pub(super) use write::*;

use super::*;
use diesel_async::AsyncPgConnection;

async fn build_user_summaries(
    conn: &mut AsyncPgConnection,
    rows: Vec<UserRecord>,
) -> anyhow::Result<Vec<UserSummary>> {
    let ids = rows.iter().map(|row| row.id).collect::<Vec<_>>();
    let mut permissions = load_permissions_map(conn, &ids).await?;
    let mut metrics = load_metrics_map(conn, &ids).await?;

    rows.into_iter()
        .map(|row| {
            let user_id = row.id;
            let fallback_updated_at = row.updated_at;
            let user = user_from_record(row, permissions.remove(&user_id).unwrap_or_default())?;
            let metrics = metrics.remove(&user_id).unwrap_or(UserRepoMetrics {
                user_id: user.id,
                downloaded_bytes: 0,
                updated_at: fallback_updated_at,
            });
            Ok(UserSummary { user, metrics })
        })
        .collect()
}

async fn load_user_summary(
    conn: &mut AsyncPgConnection,
    user_id: Uuid,
) -> anyhow::Result<UserSummary> {
    let row = users::table
        .find(user_id)
        .select(UserRecord::as_select())
        .first(conn)
        .await?;
    let mut users = build_user_summaries(conn, vec![row]).await?;
    users
        .pop()
        .context("user summary missing for fetched user record")
}

async fn load_permissions_map(
    conn: &mut AsyncPgConnection,
    user_ids: &[Uuid],
) -> anyhow::Result<HashMap<Uuid, Vec<UserPermission>>> {
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
    let mut map = HashMap::<Uuid, Vec<UserPermission>>::new();
    for row in rows {
        map.entry(row.user_id).or_default().push(row.permission);
    }
    Ok(map)
}

async fn load_metrics_map(
    conn: &mut AsyncPgConnection,
    user_ids: &[Uuid],
) -> anyhow::Result<HashMap<Uuid, UserRepoMetrics>> {
    if user_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let rows = user_repo_metrics::table
        .filter(user_repo_metrics::user_id.eq_any(user_ids))
        .select(UserRepoMetricsRecord::as_select())
        .load::<UserRepoMetricsRecord>(conn)
        .await?;
    let mut map = HashMap::<Uuid, UserRepoMetrics>::new();
    for row in rows {
        map.insert(row.user_id.clone(), user_metrics_from_record(row)?);
    }
    Ok(map)
}

async fn replace_permissions(
    conn: &mut AsyncPgConnection,
    user_id: Uuid,
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
