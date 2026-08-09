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

fn decimal_stat(value: Option<BigDecimal>, field: &str) -> anyhow::Result<u64> {
    let value = value.unwrap_or_default();
    value
        .to_u64()
        .ok_or_else(|| anyhow::anyhow!("invalid aggregate ccache statistic {field}: {value}"))
}
