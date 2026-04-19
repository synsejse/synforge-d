use super::*;
use diesel_async::{AsyncConnection, RunQueryDsl, scoped_futures::ScopedFutureExt};

pub(super) async fn list_packages(
    store: &DieselStore,
    limit: usize,
    offset: usize,
    search: Option<String>,
    enabled: Option<bool>,
) -> anyhow::Result<Vec<PackageResponse>> {
    let search = search.map(|value| format!("%{}%", value));
    let mut conn = store.get_connection().await?;
    let mut query = packages::table.into_boxed();
    if let Some(search) = search.as_deref() {
        query = query.filter(
            packages::name
                .like(search)
                .or(packages::description.like(search)),
        );
    }
    if let Some(enabled) = enabled {
        query = query.filter(packages::enabled.eq(enabled));
    }
    let rows = query
        .order(packages::name.asc())
        .limit(limit as i64)
        .offset(offset as i64)
        .select(PackageRecord::as_select())
        .load(&mut conn)
        .await?;
    let mut responses = Vec::with_capacity(rows.len());
    for record in rows {
        responses.push(package_response_from_record(&mut conn, record).await?);
    }
    Ok(responses)
}

pub(super) async fn count_packages(
    store: &DieselStore,
    search: Option<String>,
    enabled: Option<bool>,
) -> anyhow::Result<u64> {
    let search = search.map(|value| format!("%{}%", value));
    let mut conn = store.get_connection().await?;
    let mut query = packages::table.into_boxed();
    if let Some(search) = search.as_deref() {
        query = query.filter(
            packages::name
                .like(search)
                .or(packages::description.like(search)),
        );
    }
    if let Some(enabled) = enabled {
        query = query.filter(packages::enabled.eq(enabled));
    }
    let count = query.count().get_result::<i64>(&mut conn).await?;
    Ok(count as u64)
}

pub(super) async fn get_package(
    store: &DieselStore,
    package_name: &str,
) -> anyhow::Result<Option<PackageResponse>> {
    let package_name = package_name.to_string();
    let mut conn = store.get_connection().await?;
    let row = packages::table
        .find(package_name)
        .select(PackageRecord::as_select())
        .first(&mut conn)
        .await
        .optional()?;
    match row {
        Some(record) => Ok(Some(package_response_from_record(&mut conn, record).await?)),
        None => Ok(None),
    }
}

pub(super) async fn upsert_package(
    store: &DieselStore,
    package: &PackageDefinition,
) -> anyhow::Result<()> {
    let package = package.clone();
    let mut conn = store.get_connection().await?;
    let package_name = package.name.clone();
    let description = package.description.clone();
    let repo_subdir = package.repo_subdir.clone();
    let source_repo_url = package.source.repo_url.clone();
    let source_spec_file = package.source.spec_file.clone();
    let spec_file = package.spec_file.to_string_lossy().to_string();
    let version = package.version.clone();
    let release = package.release.clone();
    let source_poll = package.source.poll;
    let enabled = package.enabled;
    let publish_srpm = package.publish_srpm;
    let publish_debuginfo = package.publish_debuginfo;
    let network_access = package.network_access;
    let poll_interval_seconds = package.poll_interval_seconds as i64;
    let build_timeout_seconds = package.build_timeout_seconds as i64;
    let package_history_count = package.package_history_count as i64;
    let ccache_enabled = package.ccache_enabled;
    let build_env_json = serde_json::to_value(&package.build_env)?;
    let cpu_limit_millicores = package
        .cpu_limit_millicores
        .map(i64::try_from)
        .transpose()
        .context("cpu_limit_millicores exceeds database range")?;
    let memory_limit_mb = package
        .memory_limit_mb
        .map(i64::try_from)
        .transpose()
        .context("memory_limit_mb exceeds database range")?;
    let ccache_max_size_mb = package
        .ccache_max_size_mb
        .map(i64::try_from)
        .transpose()
        .context("ccache_max_size_mb exceeds database range")?;
    let mut mock_chroots = package.mock_chroots;
    mock_chroots.sort_unstable();
    mock_chroots.dedup();
    conn.transaction::<(), anyhow::Error, _>(|conn| {
        async move {
            let new_row = NewPackageRecord {
                name: package_name.as_str(),
                description: description.as_str(),
                enabled,
                repo_subdir: repo_subdir.as_str(),
                publish_srpm,
                publish_debuginfo,
                network_access,
                source_repo_url: source_repo_url.as_str(),
                source_spec_file: source_spec_file.as_str(),
                source_poll,
                poll_interval_seconds,
                build_timeout_seconds,
                package_history_count,
                cpu_limit_millicores,
                memory_limit_mb,
                ccache_enabled,
                ccache_max_size_mb,
                build_env_json: build_env_json.clone(),
                spec_file: spec_file.as_str(),
                version: version.as_str(),
                release: release.as_str(),
            };
            diesel::insert_into(packages::table)
                .values(&new_row)
                .on_conflict(packages::name)
                .do_update()
                .set((
                    packages::description.eq(new_row.description),
                    packages::enabled.eq(new_row.enabled),
                    packages::repo_subdir.eq(new_row.repo_subdir),
                    packages::publish_srpm.eq(new_row.publish_srpm),
                    packages::publish_debuginfo.eq(new_row.publish_debuginfo),
                    packages::network_access.eq(new_row.network_access),
                    packages::source_repo_url.eq(new_row.source_repo_url),
                    packages::source_spec_file.eq(new_row.source_spec_file),
                    packages::source_poll.eq(new_row.source_poll),
                    packages::poll_interval_seconds.eq(new_row.poll_interval_seconds),
                    packages::build_timeout_seconds.eq(new_row.build_timeout_seconds),
                    packages::package_history_count.eq(new_row.package_history_count),
                    packages::cpu_limit_millicores.eq(new_row.cpu_limit_millicores),
                    packages::memory_limit_mb.eq(new_row.memory_limit_mb),
                    packages::ccache_enabled.eq(new_row.ccache_enabled),
                    packages::ccache_max_size_mb.eq(new_row.ccache_max_size_mb),
                    packages::build_env_json.eq(new_row.build_env_json.clone()),
                    packages::spec_file.eq(new_row.spec_file),
                    packages::version.eq(new_row.version),
                    packages::release.eq(new_row.release),
                ))
                .execute(conn)
                .await?;

            diesel::delete(
                package_mock_chroots::table
                    .filter(package_mock_chroots::package_name.eq(package_name.as_str())),
            )
            .execute(conn)
            .await?;

            if !mock_chroots.is_empty() {
                let rows = mock_chroots
                    .iter()
                    .map(|mock_chroot| NewPackageMockChrootRecord {
                        package_name: package_name.as_str(),
                        mock_chroot: mock_chroot.as_str(),
                    })
                    .collect::<Vec<_>>();
                diesel::insert_into(package_mock_chroots::table)
                    .values(&rows)
                    .execute(conn)
                    .await?;
            }

            Ok(())
        }
        .scope_boxed()
    })
    .await?;
    Ok(())
}

pub(super) async fn remove_package(store: &DieselStore, package_name: &str) -> anyhow::Result<()> {
    let package_name = package_name.to_string();
    let mut conn = store.get_connection().await?;
    diesel::delete(packages::table.find(package_name))
        .execute(&mut conn)
        .await?;
    Ok(())
}
