use bigdecimal::{BigDecimal, ToPrimitive};
use diesel::dsl::{count, sum};
use diesel_async::RunQueryDsl;

use super::*;

type TargetStatsRow = (
    String,
    i64,
    Option<BigDecimal>,
    Option<BigDecimal>,
    Option<BigDecimal>,
    Option<BigDecimal>,
    Option<BigDecimal>,
    Option<BigDecimal>,
);

type WorkspaceTargetStatsRow = (
    String,
    String,
    i64,
    Option<BigDecimal>,
    Option<BigDecimal>,
    Option<BigDecimal>,
    Option<BigDecimal>,
    Option<BigDecimal>,
    Option<BigDecimal>,
);

pub(in crate::db) async fn list_package_ccache_stats(
    store: &DieselStore,
    package_name: &str,
) -> anyhow::Result<Vec<PackageTargetCcacheStats>> {
    let mut conn = store.get_connection().await?;
    let rows = build_ccache_stats::table
        .inner_join(build_jobs::table.on(build_ccache_stats::job_id.eq(build_jobs::id)))
        .filter(build_jobs::package_name.eq(package_name))
        .group_by(build_jobs::mock_chroot)
        .order(build_jobs::mock_chroot.asc())
        .select((
            build_jobs::mock_chroot,
            count(build_ccache_stats::job_id),
            sum(build_ccache_stats::compiler_calls),
            sum(build_ccache_stats::direct_hits),
            sum(build_ccache_stats::preprocessed_hits),
            sum(build_ccache_stats::cache_misses),
            sum(build_ccache_stats::uncacheable_calls),
            sum(build_ccache_stats::error_calls),
        ))
        .load::<TargetStatsRow>(&mut conn)
        .await?;

    rows.into_iter()
        .map(
            |(
                mock_chroot,
                build_count,
                compiler_calls,
                direct_hits,
                preprocessed_hits,
                cache_misses,
                uncacheable_calls,
                error_calls,
            )| {
                Ok(PackageTargetCcacheStats {
                    mock_chroot,
                    build_count: u64::try_from(build_count)?,
                    stats: BuildCcacheStats {
                        compiler_calls: decimal_stat(compiler_calls, "compiler_calls")?,
                        direct_hits: decimal_stat(direct_hits, "direct_hits")?,
                        preprocessed_hits: decimal_stat(preprocessed_hits, "preprocessed_hits")?,
                        cache_misses: decimal_stat(cache_misses, "cache_misses")?,
                        uncacheable_calls: decimal_stat(uncacheable_calls, "uncacheable_calls")?,
                        error_calls: decimal_stat(error_calls, "error_calls")?,
                    },
                })
            },
        )
        .collect()
}

pub(in crate::db) async fn get_workspace_ccache_stats(
    store: &DieselStore,
) -> anyhow::Result<WorkspaceCcacheStats> {
    let mut conn = store.get_connection().await?;
    let rows = build_ccache_stats::table
        .inner_join(build_jobs::table.on(build_ccache_stats::job_id.eq(build_jobs::id)))
        .group_by((build_jobs::package_name, build_jobs::mock_chroot))
        .order((
            build_jobs::package_name.asc(),
            build_jobs::mock_chroot.asc(),
        ))
        .select((
            build_jobs::package_name,
            build_jobs::mock_chroot,
            count(build_ccache_stats::job_id),
            sum(build_ccache_stats::compiler_calls),
            sum(build_ccache_stats::direct_hits),
            sum(build_ccache_stats::preprocessed_hits),
            sum(build_ccache_stats::cache_misses),
            sum(build_ccache_stats::uncacheable_calls),
            sum(build_ccache_stats::error_calls),
        ))
        .load::<WorkspaceTargetStatsRow>(&mut conn)
        .await?;

    let targets = rows
        .into_iter()
        .map(
            |(
                package_name,
                mock_chroot,
                build_count,
                compiler_calls,
                direct_hits,
                preprocessed_hits,
                cache_misses,
                uncacheable_calls,
                error_calls,
            )| {
                Ok(WorkspaceCcacheTargetStats {
                    package_name,
                    mock_chroot,
                    build_count: u64::try_from(build_count)?,
                    stats: BuildCcacheStats {
                        compiler_calls: decimal_stat(compiler_calls, "compiler_calls")?,
                        direct_hits: decimal_stat(direct_hits, "direct_hits")?,
                        preprocessed_hits: decimal_stat(preprocessed_hits, "preprocessed_hits")?,
                        cache_misses: decimal_stat(cache_misses, "cache_misses")?,
                        uncacheable_calls: decimal_stat(uncacheable_calls, "uncacheable_calls")?,
                        error_calls: decimal_stat(error_calls, "error_calls")?,
                    },
                })
            },
        )
        .collect::<anyhow::Result<Vec<_>>>()?;

    aggregate_workspace_stats(targets)
}

fn aggregate_workspace_stats(
    targets: Vec<WorkspaceCcacheTargetStats>,
) -> anyhow::Result<WorkspaceCcacheStats> {
    let mut workspace = WorkspaceCcacheStats {
        targets,
        ..WorkspaceCcacheStats::default()
    };
    for target in &workspace.targets {
        workspace.build_count =
            checked_sum(workspace.build_count, target.build_count, "build_count")?;
        workspace.stats.compiler_calls = checked_sum(
            workspace.stats.compiler_calls,
            target.stats.compiler_calls,
            "compiler_calls",
        )?;
        workspace.stats.direct_hits = checked_sum(
            workspace.stats.direct_hits,
            target.stats.direct_hits,
            "direct_hits",
        )?;
        workspace.stats.preprocessed_hits = checked_sum(
            workspace.stats.preprocessed_hits,
            target.stats.preprocessed_hits,
            "preprocessed_hits",
        )?;
        workspace.stats.cache_misses = checked_sum(
            workspace.stats.cache_misses,
            target.stats.cache_misses,
            "cache_misses",
        )?;
        workspace.stats.uncacheable_calls = checked_sum(
            workspace.stats.uncacheable_calls,
            target.stats.uncacheable_calls,
            "uncacheable_calls",
        )?;
        workspace.stats.error_calls = checked_sum(
            workspace.stats.error_calls,
            target.stats.error_calls,
            "error_calls",
        )?;
    }
    Ok(workspace)
}

fn checked_sum(left: u64, right: u64, field: &str) -> anyhow::Result<u64> {
    left.checked_add(right)
        .ok_or_else(|| anyhow::anyhow!("workspace ccache aggregate overflow for {field}"))
}

fn decimal_stat(value: Option<BigDecimal>, field: &str) -> anyhow::Result<u64> {
    let value = value.unwrap_or_default();
    value
        .to_u64()
        .ok_or_else(|| anyhow::anyhow!("invalid aggregate ccache statistic {field}: {value}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aggregates_workspace_targets() {
        let workspace = aggregate_workspace_stats(vec![
            target("mesa-git", "fedora-44-x86_64", 3, 100, 60, 10, 30),
            target("qemu", "fedora-44-x86_64", 2, 80, 20, 5, 55),
        ])
        .expect("aggregate workspace stats");

        assert_eq!(workspace.build_count, 5);
        assert_eq!(workspace.stats.compiler_calls, 180);
        assert_eq!(workspace.stats.direct_hits, 80);
        assert_eq!(workspace.stats.preprocessed_hits, 15);
        assert_eq!(workspace.stats.cache_misses, 85);
    }

    fn target(
        package_name: &str,
        mock_chroot: &str,
        build_count: u64,
        compiler_calls: u64,
        direct_hits: u64,
        preprocessed_hits: u64,
        cache_misses: u64,
    ) -> WorkspaceCcacheTargetStats {
        WorkspaceCcacheTargetStats {
            package_name: package_name.to_string(),
            mock_chroot: mock_chroot.to_string(),
            build_count,
            stats: BuildCcacheStats {
                compiler_calls,
                direct_hits,
                preprocessed_hits,
                cache_misses,
                uncacheable_calls: 0,
                error_calls: 0,
            },
        }
    }
}
