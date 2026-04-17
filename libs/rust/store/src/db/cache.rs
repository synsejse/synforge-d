use diesel::dsl::max;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use super::*;
use crate::schema::git_mirror_cache_states;

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = git_mirror_cache_states)]
pub struct GitMirrorCacheStateRow {
    pub mirror_key: String,
    pub repo_url: String,
    pub last_fetched_at: i64,
    pub last_used_at: i64,
}

#[derive(Insertable)]
#[diesel(table_name = git_mirror_cache_states)]
pub struct NewGitMirrorCacheState<'a> {
    pub mirror_key: &'a str,
    pub repo_url: &'a str,
    pub last_fetched_at: i64,
    pub last_used_at: i64,
}

impl From<GitMirrorCacheStateRow> for traits::GitMirrorCacheState {
    fn from(row: GitMirrorCacheStateRow) -> Self {
        Self {
            mirror_key: row.mirror_key,
            repo_url: row.repo_url,
            last_fetched_at: row.last_fetched_at,
            last_used_at: row.last_used_at,
        }
    }
}

pub(super) async fn get_git_mirror_cache_state(
    store: &DieselStore,
    mirror_key: &str,
) -> anyhow::Result<Option<traits::GitMirrorCacheState>> {
    let mirror_key = mirror_key.to_string();
    let mut conn = store.get_connection().await?;
    let row = git_mirror_cache_states::table
        .find(mirror_key)
        .select(GitMirrorCacheStateRow::as_select())
        .first(&mut conn)
        .await
        .optional()?;
    Ok(row.map(traits::GitMirrorCacheState::from))
}

pub(super) async fn upsert_git_mirror_cache_state(
    store: &DieselStore,
    mirror_key: &str,
    repo_url: &str,
    last_fetched_at: i64,
    last_used_at: i64,
) -> anyhow::Result<()> {
    let mirror_key = mirror_key.to_string();
    let repo_url = repo_url.to_string();
    let mut conn = store.get_connection().await?;
    let new_state = NewGitMirrorCacheState {
        mirror_key: &mirror_key,
        repo_url: &repo_url,
        last_fetched_at,
        last_used_at,
    };
    diesel::insert_into(git_mirror_cache_states::table)
        .values(&new_state)
        .on_conflict(git_mirror_cache_states::mirror_key)
        .do_update()
        .set((
            git_mirror_cache_states::repo_url.eq(&repo_url),
            git_mirror_cache_states::last_fetched_at.eq(last_fetched_at),
            git_mirror_cache_states::last_used_at.eq(last_used_at),
        ))
        .execute(&mut conn)
        .await?;
    Ok(())
}

pub(super) async fn list_stale_git_mirror_cache_states(
    store: &DieselStore,
    last_used_before_or_equal: i64,
) -> anyhow::Result<Vec<traits::GitMirrorCacheState>> {
    let mut conn = store.get_connection().await?;
    let rows = git_mirror_cache_states::table
        .filter(git_mirror_cache_states::last_used_at.le(last_used_before_or_equal))
        .select(GitMirrorCacheStateRow::as_select())
        .load(&mut conn)
        .await?;
    Ok(rows
        .into_iter()
        .map(traits::GitMirrorCacheState::from)
        .collect())
}

pub(super) async fn delete_git_mirror_cache_state(
    store: &DieselStore,
    mirror_key: &str,
) -> anyhow::Result<()> {
    let mirror_key = mirror_key.to_string();
    let mut conn = store.get_connection().await?;
    diesel::delete(
        git_mirror_cache_states::table.filter(git_mirror_cache_states::mirror_key.eq(&mirror_key)),
    )
    .execute(&mut conn)
    .await?;
    Ok(())
}

pub(super) async fn count_git_mirror_cache_states(store: &DieselStore) -> anyhow::Result<u64> {
    let mut conn = store.get_connection().await?;
    let count = git_mirror_cache_states::table
        .count()
        .get_result::<i64>(&mut conn)
        .await?;
    Ok(count as u64)
}

pub(super) async fn count_stale_git_mirror_cache_states(
    store: &DieselStore,
    last_used_before_or_equal: i64,
) -> anyhow::Result<u64> {
    let mut conn = store.get_connection().await?;
    let count = git_mirror_cache_states::table
        .filter(git_mirror_cache_states::last_used_at.le(last_used_before_or_equal))
        .count()
        .get_result::<i64>(&mut conn)
        .await?;
    Ok(count as u64)
}

pub(super) async fn latest_git_mirror_cache_timestamps(
    store: &DieselStore,
) -> anyhow::Result<(Option<i64>, Option<i64>)> {
    let mut conn = store.get_connection().await?;
    let latest_fetched = git_mirror_cache_states::table
        .select(max(git_mirror_cache_states::last_fetched_at))
        .first::<Option<i64>>(&mut conn)
        .await?;
    let latest_used = git_mirror_cache_states::table
        .select(max(git_mirror_cache_states::last_used_at))
        .first::<Option<i64>>(&mut conn)
        .await?;
    Ok((latest_fetched, latest_used))
}
