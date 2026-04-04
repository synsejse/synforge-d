use super::*;

pub(super) async fn list_packages(
    store: &DieselStore,
    limit: usize,
    offset: usize,
    search: Option<String>,
    enabled: Option<bool>,
) -> anyhow::Result<Vec<PackageResponse>> {
    let search = search.map(|value| format!("%{}%", value));
    store
        .with_connection(move |conn| {
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
                .load(conn)?;
            rows.into_iter()
                .map(|record| package_response_from_record(conn, record))
                .collect()
        })
        .await
}

pub(super) async fn count_packages(
    store: &DieselStore,
    search: Option<String>,
    enabled: Option<bool>,
) -> anyhow::Result<u64> {
    let search = search.map(|value| format!("%{}%", value));
    store
        .with_connection(move |conn| {
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
            let count = query.count().get_result::<i64>(conn)?;
            Ok(count as u64)
        })
        .await
}

pub(super) async fn get_package(
    store: &DieselStore,
    package_name: &str,
) -> anyhow::Result<Option<PackageResponse>> {
    let package_name = package_name.to_string();
    store
        .with_connection(move |conn| {
            let row = packages::table
                .find(package_name)
                .select(PackageRecord::as_select())
                .first(conn)
                .optional()?;
            row.map(|record| package_response_from_record(conn, record))
                .transpose()
        })
        .await
}

pub(super) async fn upsert_package(
    store: &DieselStore,
    package: &PackageDefinition,
) -> anyhow::Result<()> {
    let package = package.clone();
    store
        .with_connection(move |conn| {
            let spec_file = package.spec_file.to_string_lossy().to_string();
            let build_env_json = serde_json::to_string(&package.build_env)?;
            let mock_chroots_json = serde_json::to_string(&package.mock_chroots)?;
            let new_row = NewPackageRecord {
                name: package.name.as_str(),
                description: package.description.as_str(),
                enabled: package.enabled,
                repo_subdir: package.repo_subdir.as_str(),
                publish_srpm: package.publish_srpm,
                network_access: package.network_access,
                mock_chroots_json: mock_chroots_json.as_str(),
                source_repo_url: package.source.repo_url.as_str(),
                source_spec_file: package.source.spec_file.as_str(),
                source_poll: package.source.poll,
                poll_interval_seconds: package.poll_interval_seconds as i64,
                build_timeout_seconds: package.build_timeout_seconds as i64,
                package_history_count: package.package_history_count as i64,
                build_env_json: build_env_json.as_str(),
                spec_file: spec_file.as_str(),
                version: package.version.as_str(),
                release: package.release.as_str(),
            };
            diesel::insert_into(packages::table)
                .values(&new_row)
                .on_conflict(diesel::dsl::DuplicatedKeys)
                .do_update()
                .set((
                    packages::description.eq(new_row.description),
                    packages::enabled.eq(new_row.enabled),
                    packages::repo_subdir.eq(new_row.repo_subdir),
                    packages::publish_srpm.eq(new_row.publish_srpm),
                    packages::network_access.eq(new_row.network_access),
                    packages::mock_chroots_json.eq(new_row.mock_chroots_json),
                    packages::source_repo_url.eq(new_row.source_repo_url),
                    packages::source_spec_file.eq(new_row.source_spec_file),
                    packages::source_poll.eq(new_row.source_poll),
                    packages::poll_interval_seconds.eq(new_row.poll_interval_seconds),
                    packages::build_timeout_seconds.eq(new_row.build_timeout_seconds),
                    packages::package_history_count.eq(new_row.package_history_count),
                    packages::build_env_json.eq(new_row.build_env_json),
                    packages::spec_file.eq(new_row.spec_file),
                    packages::version.eq(new_row.version),
                    packages::release.eq(new_row.release),
                ))
                .execute(conn)?;
            Ok(())
        })
        .await
}

pub(super) async fn remove_package(store: &DieselStore, package_name: &str) -> anyhow::Result<()> {
    let package_name = package_name.to_string();
    store
        .with_connection(move |conn| {
            diesel::delete(packages::table.find(package_name)).execute(conn)?;
            Ok(())
        })
        .await
}
