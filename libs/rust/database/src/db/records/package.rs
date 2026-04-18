use super::super::*;

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = packages)]
pub(crate) struct PackageRecord {
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) enabled: bool,
    pub(crate) repo_subdir: String,
    pub(crate) publish_srpm: bool,
    pub(crate) publish_debuginfo: bool,
    pub(crate) network_access: bool,
    pub(crate) mock_chroots_json: String,
    pub(crate) source_repo_url: String,
    pub(crate) source_spec_file: String,
    pub(crate) source_poll: bool,
    pub(crate) poll_interval_seconds: i64,
    pub(crate) build_timeout_seconds: i64,
    pub(crate) package_history_count: i64,
    pub(crate) cpu_limit_millicores: Option<i64>,
    pub(crate) memory_limit_mb: Option<i64>,
    pub(crate) ccache_enabled: bool,
    pub(crate) ccache_max_size_mb: Option<i64>,
    pub(crate) build_env_json: String,
    pub(crate) spec_file: String,
    pub(crate) version: String,
    pub(crate) release: String,
}

#[derive(Insertable)]
#[diesel(table_name = packages)]
pub(crate) struct NewPackageRecord<'a> {
    pub(crate) name: &'a str,
    pub(crate) description: &'a str,
    pub(crate) enabled: bool,
    pub(crate) repo_subdir: &'a str,
    pub(crate) publish_srpm: bool,
    pub(crate) publish_debuginfo: bool,
    pub(crate) network_access: bool,
    pub(crate) mock_chroots_json: &'a str,
    pub(crate) source_repo_url: &'a str,
    pub(crate) source_spec_file: &'a str,
    pub(crate) source_poll: bool,
    pub(crate) poll_interval_seconds: i64,
    pub(crate) build_timeout_seconds: i64,
    pub(crate) package_history_count: i64,
    pub(crate) cpu_limit_millicores: Option<i64>,
    pub(crate) memory_limit_mb: Option<i64>,
    pub(crate) ccache_enabled: bool,
    pub(crate) ccache_max_size_mb: Option<i64>,
    pub(crate) build_env_json: &'a str,
    pub(crate) spec_file: &'a str,
    pub(crate) version: &'a str,
    pub(crate) release: &'a str,
}

pub(crate) async fn package_response_from_record(
    conn: &mut AsyncPgConnection,
    record: PackageRecord,
) -> anyhow::Result<PackageResponse> {
    let package = PackageDefinition {
        name: record.name.clone(),
        description: record.description,
        enabled: record.enabled,
        repo_subdir: record.repo_subdir,
        publish_srpm: record.publish_srpm,
        publish_debuginfo: record.publish_debuginfo,
        network_access: record.network_access,
        mock_chroots: serde_json::from_str::<Vec<String>>(&record.mock_chroots_json)
            .unwrap_or_default(),
        source: SpecSource {
            repo_url: record.source_repo_url,
            spec_file: record.source_spec_file,
            poll: record.source_poll,
        },
        poll_interval_seconds: record.poll_interval_seconds as u64,
        build_timeout_seconds: record.build_timeout_seconds as u64,
        package_history_count: record.package_history_count as u64,
        cpu_limit_millicores: record
            .cpu_limit_millicores
            .and_then(|value| u64::try_from(value).ok())
            .filter(|value| *value > 0),
        memory_limit_mb: record
            .memory_limit_mb
            .and_then(|value| u64::try_from(value).ok())
            .filter(|value| *value > 0),
        ccache_enabled: record.ccache_enabled,
        ccache_max_size_mb: record
            .ccache_max_size_mb
            .and_then(|value| u64::try_from(value).ok())
            .filter(|value| *value > 0),
        build_env: serde_json::from_str::<Vec<BuildEnvVar>>(&record.build_env_json)
            .unwrap_or_default(),
        spec_file: PathBuf::from(record.spec_file),
        version: record.version,
        release: record.release,
    };
    let state = compute_package_state(conn, &record.name, &package.mock_chroots).await?;
    Ok(PackageResponse { package, state })
}

pub(crate) async fn compute_package_state(
    conn: &mut AsyncPgConnection,
    package_name: &str,
    mock_chroots: &[String],
) -> anyhow::Result<PackageRuntimeState> {
    let last_success = build_jobs::table
        .filter(build_jobs::package_name.eq(package_name))
        .filter(build_jobs::status.eq(BuildStatus::Succeeded))
        .order(build_jobs::finished_at.desc())
        .select((build_jobs::id, build_jobs::revision))
        .first::<(String, String)>(conn)
        .await
        .optional()?;

    let active_job = build_jobs::table
        .filter(build_jobs::package_name.eq(package_name))
        .filter(
            build_jobs::status
                .eq(BuildStatus::Pending)
                .or(build_jobs::status.eq(BuildStatus::Running)),
        )
        .order(build_jobs::created_at.desc())
        .select(build_jobs::id)
        .first::<String>(conn)
        .await
        .optional()?;

    let mut targets = Vec::with_capacity(mock_chroots.len());
    for mock_chroot in mock_chroots {
        let last_success = build_jobs::table
            .filter(build_jobs::package_name.eq(package_name))
            .filter(build_jobs::mock_chroot.eq(mock_chroot))
            .filter(build_jobs::status.eq(BuildStatus::Succeeded))
            .order(build_jobs::finished_at.desc())
            .select((build_jobs::id, build_jobs::revision))
            .first::<(String, String)>(conn)
            .await
            .optional()?;

        let active_job = build_jobs::table
            .filter(build_jobs::package_name.eq(package_name))
            .filter(build_jobs::mock_chroot.eq(mock_chroot))
            .filter(
                build_jobs::status
                    .eq(BuildStatus::Pending)
                    .or(build_jobs::status.eq(BuildStatus::Running)),
            )
            .order(build_jobs::created_at.desc())
            .select((build_jobs::id, build_jobs::status))
            .first::<(String, BuildStatus)>(conn)
            .await
            .optional()?;

        let backoff = build_failure_backoff::table
            .find((package_name, mock_chroot.as_str()))
            .select((
                build_failure_backoff::consecutive_failures,
                build_failure_backoff::next_eligible_at,
            ))
            .first::<(i32, String)>(conn)
            .await
            .optional()?;
        let backoff_until = backoff
            .as_ref()
            .map(|(_, next_eligible_at)| next_eligible_at.clone());
        let backoff_remaining_seconds = backoff
            .as_ref()
            .and_then(|(_, next_eligible_at)| parse_timestamp(next_eligible_at).ok())
            .and_then(|next_eligible_at| {
                let wait_seconds = (next_eligible_at - now_utc()).whole_seconds();
                if wait_seconds > 0 {
                    Some(wait_seconds as u64)
                } else {
                    None
                }
            });

        targets.push(PackageTargetRuntimeState {
            mock_chroot: mock_chroot.clone(),
            last_revision: last_success.as_ref().map(|(_, revision)| revision.clone()),
            last_successful_build_id: last_success
                .as_ref()
                .map(|(id, _)| Uuid::parse_str(id))
                .transpose()?,
            active_job_id: active_job
                .as_ref()
                .map(|(id, _)| Uuid::parse_str(id))
                .transpose()?,
            active_status: active_job.as_ref().map(|(_, status)| *status),
            backoff_until,
            backoff_remaining_seconds,
        });
    }

    Ok(PackageRuntimeState {
        last_revision: last_success.as_ref().map(|(_, revision)| revision.clone()),
        last_successful_build_id: last_success
            .as_ref()
            .map(|(id, _)| Uuid::parse_str(id))
            .transpose()?,
        active_job_id: active_job.as_deref().map(Uuid::parse_str).transpose()?,
        targets,
    })
}
