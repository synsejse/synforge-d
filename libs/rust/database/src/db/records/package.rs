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
    pub(crate) build_env_json: serde_json::Value,
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
    pub(crate) build_env_json: serde_json::Value,
    pub(crate) spec_file: &'a str,
    pub(crate) version: &'a str,
    pub(crate) release: &'a str,
}

#[derive(Insertable)]
#[diesel(table_name = package_mock_chroots)]
pub(crate) struct NewPackageMockChrootRecord<'a> {
    pub(crate) package_name: &'a str,
    pub(crate) mock_chroot: &'a str,
}

pub(crate) async fn package_response_from_record(
    conn: &mut AsyncPgConnection,
    record: PackageRecord,
) -> anyhow::Result<PackageResponse> {
    let mut responses = package_responses_from_records(conn, vec![record]).await?;
    Ok(responses.pop().expect("input record produces one response"))
}

/// Aggregated runtime state for a batch of packages, keyed by name.
/// Loaded in one pass so the response builder is a pure in-memory join.
struct PackageRuntimeAggregate {
    chroots_by_pkg: HashMap<String, Vec<String>>,
    target_success: HashMap<(String, String), (Uuid, String)>,
    target_active: HashMap<(String, String), (Uuid, BuildStatus)>,
    backoff: HashMap<(String, String), OffsetDateTime>,
    pkg_success: HashMap<String, (Uuid, String)>,
    pkg_active: HashMap<String, Uuid>,
}

pub(crate) async fn package_responses_from_records(
    conn: &mut AsyncPgConnection,
    records: Vec<PackageRecord>,
) -> anyhow::Result<Vec<PackageResponse>> {
    if records.is_empty() {
        return Ok(Vec::new());
    }
    let names: Vec<String> = records.iter().map(|r| r.name.clone()).collect();
    let mut aggregate = load_package_runtime_aggregate(conn, &names).await?;

    let now = now_utc();
    let mut responses = Vec::with_capacity(records.len());
    for record in records {
        let chroots = aggregate.chroots_by_pkg.remove(&record.name).unwrap_or_default();
        let targets = chroots
            .iter()
            .map(|chroot| build_target_runtime_state(&record.name, chroot, &aggregate, now))
            .collect();
        let pkg_last = aggregate.pkg_success.get(&record.name);
        let state = PackageRuntimeState {
            last_revision: pkg_last.map(|(_, revision)| revision.clone()),
            last_successful_build_id: pkg_last.map(|(id, _)| *id),
            active_job_id: aggregate.pkg_active.get(&record.name).copied(),
            targets,
        };
        let package = build_package_definition(record, chroots);
        responses.push(PackageResponse { package, state });
    }
    Ok(responses)
}

async fn load_package_runtime_aggregate(
    conn: &mut AsyncPgConnection,
    names: &[String],
) -> anyhow::Result<PackageRuntimeAggregate> {
    let chroot_rows: Vec<(String, String)> = package_mock_chroots::table
        .filter(package_mock_chroots::package_name.eq_any(names))
        .order((
            package_mock_chroots::package_name.asc(),
            package_mock_chroots::mock_chroot.asc(),
        ))
        .select((
            package_mock_chroots::package_name,
            package_mock_chroots::mock_chroot,
        ))
        .load(conn)
        .await?;
    let mut chroots_by_pkg: HashMap<String, Vec<String>> = HashMap::new();
    for (pkg, chroot) in chroot_rows {
        chroots_by_pkg.entry(pkg).or_default().push(chroot);
    }

    let target_success_rows: Vec<(String, String, Uuid, String)> = build_jobs::table
        .filter(build_jobs::package_name.eq_any(names))
        .filter(build_jobs::status.eq(BuildStatus::Succeeded))
        .filter(build_jobs::deleted_at.is_null())
        .order((
            build_jobs::package_name.asc(),
            build_jobs::mock_chroot.asc(),
            build_jobs::finished_at.desc(),
        ))
        .distinct_on((build_jobs::package_name, build_jobs::mock_chroot))
        .select((
            build_jobs::package_name,
            build_jobs::mock_chroot,
            build_jobs::id,
            build_jobs::revision,
        ))
        .load(conn)
        .await?;
    let target_success = target_success_rows
        .into_iter()
        .map(|(pkg, chroot, id, revision)| ((pkg, chroot), (id, revision)))
        .collect();

    let target_active_rows: Vec<(String, String, Uuid, BuildStatus)> = build_jobs::table
        .filter(build_jobs::package_name.eq_any(names))
        .filter(
            build_jobs::status
                .eq(BuildStatus::Pending)
                .or(build_jobs::status.eq(BuildStatus::Running)),
        )
        .order((
            build_jobs::package_name.asc(),
            build_jobs::mock_chroot.asc(),
            build_jobs::created_at.desc(),
        ))
        .distinct_on((build_jobs::package_name, build_jobs::mock_chroot))
        .select((
            build_jobs::package_name,
            build_jobs::mock_chroot,
            build_jobs::id,
            build_jobs::status,
        ))
        .load(conn)
        .await?;
    let target_active = target_active_rows
        .into_iter()
        .map(|(pkg, chroot, id, status)| ((pkg, chroot), (id, status)))
        .collect();

    let backoff_rows: Vec<(String, String, OffsetDateTime)> = build_failure_backoff::table
        .filter(build_failure_backoff::package_name.eq_any(names))
        .select((
            build_failure_backoff::package_name,
            build_failure_backoff::mock_chroot,
            build_failure_backoff::next_eligible_at,
        ))
        .load(conn)
        .await?;
    let backoff = backoff_rows
        .into_iter()
        .map(|(pkg, chroot, next)| ((pkg, chroot), next))
        .collect();

    let pkg_success_rows: Vec<(String, Uuid, String)> = build_jobs::table
        .filter(build_jobs::package_name.eq_any(names))
        .filter(build_jobs::status.eq(BuildStatus::Succeeded))
        .filter(build_jobs::deleted_at.is_null())
        .order((
            build_jobs::package_name.asc(),
            build_jobs::finished_at.desc(),
        ))
        .distinct_on(build_jobs::package_name)
        .select((build_jobs::package_name, build_jobs::id, build_jobs::revision))
        .load(conn)
        .await?;
    let pkg_success = pkg_success_rows
        .into_iter()
        .map(|(pkg, id, revision)| (pkg, (id, revision)))
        .collect();

    let pkg_active_rows: Vec<(String, Uuid)> = build_jobs::table
        .filter(build_jobs::package_name.eq_any(names))
        .filter(
            build_jobs::status
                .eq(BuildStatus::Pending)
                .or(build_jobs::status.eq(BuildStatus::Running)),
        )
        .order((
            build_jobs::package_name.asc(),
            build_jobs::created_at.desc(),
        ))
        .distinct_on(build_jobs::package_name)
        .select((build_jobs::package_name, build_jobs::id))
        .load(conn)
        .await?;
    let pkg_active = pkg_active_rows.into_iter().collect();

    Ok(PackageRuntimeAggregate {
        chroots_by_pkg,
        target_success,
        target_active,
        backoff,
        pkg_success,
        pkg_active,
    })
}

fn build_target_runtime_state(
    package_name: &str,
    chroot: &str,
    aggregate: &PackageRuntimeAggregate,
    now: OffsetDateTime,
) -> PackageTargetRuntimeState {
    let key = (package_name.to_string(), chroot.to_string());
    let last = aggregate.target_success.get(&key);
    let active = aggregate.target_active.get(&key);
    let next_eligible = aggregate.backoff.get(&key).copied();
    let backoff_remaining_seconds = next_eligible.and_then(|next| {
        let wait = (next - now).whole_seconds();
        (wait > 0).then_some(wait as u64)
    });
    PackageTargetRuntimeState {
        mock_chroot: chroot.to_string(),
        last_revision: last.map(|(_, revision)| revision.clone()),
        last_successful_build_id: last.map(|(id, _)| *id),
        active_job_id: active.map(|(id, _)| *id),
        active_status: active.map(|(_, status)| *status),
        backoff_until: next_eligible.map(format_timestamp),
        backoff_remaining_seconds,
    }
}

fn build_package_definition(record: PackageRecord, mock_chroots: Vec<String>) -> PackageDefinition {
    PackageDefinition {
        name: record.name,
        description: record.description,
        enabled: record.enabled,
        repo_subdir: record.repo_subdir,
        publish_srpm: record.publish_srpm,
        publish_debuginfo: record.publish_debuginfo,
        network_access: record.network_access,
        mock_chroots,
        source: SpecSource {
            repo_url: record.source_repo_url,
            spec_file: record.source_spec_file,
            poll: record.source_poll,
        },
        poll_interval_seconds: record.poll_interval_seconds as u64,
        build_timeout_seconds: record.build_timeout_seconds as u64,
        package_history_count: record.package_history_count as u64,
        cpu_limit_millicores: positive_u64(record.cpu_limit_millicores),
        memory_limit_mb: positive_u64(record.memory_limit_mb),
        ccache_enabled: record.ccache_enabled,
        ccache_max_size_mb: positive_u64(record.ccache_max_size_mb),
        build_env: serde_json::from_value::<Vec<BuildEnvVar>>(record.build_env_json)
            .unwrap_or_default(),
        spec_file: PathBuf::from(record.spec_file),
        version: record.version,
        release: record.release,
    }
}

fn positive_u64(value: Option<i64>) -> Option<u64> {
    value
        .and_then(|v| u64::try_from(v).ok())
        .filter(|v| *v > 0)
}
