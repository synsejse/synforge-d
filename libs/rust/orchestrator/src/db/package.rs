use super::*;

pub(super) async fn list_packages(
    store: &DieselStore,
    limit: usize,
    offset: usize,
) -> anyhow::Result<Vec<PackageResponse>> {
    store
        .with_connection(move |conn| {
            let rows = packages::table
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
            let spec_path = package.spec_path.to_string_lossy().to_string();
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
                source_spec_path: package.source.spec_path.as_str(),
                source_poll: package.source.poll,
                poll_interval_seconds: package.poll_interval_seconds as i64,
                build_timeout_seconds: package.build_timeout_seconds as i64,
                package_history_count: package.package_history_count as i64,
                build_env_json: build_env_json.as_str(),
                spec_path: spec_path.as_str(),
                version: package.version.as_str(),
                release: package.release.as_str(),
            };
            diesel::insert_into(packages::table)
                .values(&new_row)
                .on_conflict(packages::name)
                .do_update()
                .set((
                    packages::description.eq(excluded(packages::description)),
                    packages::enabled.eq(excluded(packages::enabled)),
                    packages::repo_subdir.eq(excluded(packages::repo_subdir)),
                    packages::publish_srpm.eq(excluded(packages::publish_srpm)),
                    packages::network_access.eq(excluded(packages::network_access)),
                    packages::mock_chroots_json.eq(excluded(packages::mock_chroots_json)),
                    packages::source_repo_url.eq(excluded(packages::source_repo_url)),
                    packages::source_spec_path.eq(excluded(packages::source_spec_path)),
                    packages::source_poll.eq(excluded(packages::source_poll)),
                    packages::poll_interval_seconds.eq(excluded(packages::poll_interval_seconds)),
                    packages::build_timeout_seconds.eq(excluded(packages::build_timeout_seconds)),
                    packages::package_history_count.eq(excluded(packages::package_history_count)),
                    packages::build_env_json.eq(excluded(packages::build_env_json)),
                    packages::spec_path.eq(excluded(packages::spec_path)),
                    packages::version.eq(excluded(packages::version)),
                    packages::release.eq(excluded(packages::release)),
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
