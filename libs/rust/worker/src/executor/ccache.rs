use std::path::{Path, PathBuf};

use synforge_core::model::BuildCcacheStats;
use uuid::Uuid;

const MAX_STATS_LOG_BYTES: u64 = 16 * 1024 * 1024;
const STATS_DIR: &str = ".synforge/stats";

pub(super) async fn prepare_stats_log(ccache_dir: &Path, job_id: Uuid) -> anyhow::Result<PathBuf> {
    let path = host_stats_log_path(ccache_dir, job_id);
    let parent = path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("ccache stats path has no parent"))?;
    tokio::fs::create_dir_all(parent).await?;
    tokio::fs::write(&path, []).await?;
    Ok(path)
}

pub(super) fn chroot_stats_log_path(job_id: Uuid) -> String {
    format!("/var/tmp/ccache/{STATS_DIR}/{job_id}.log")
}

pub(super) async fn collect_stats(path: &Path) -> anyhow::Result<BuildCcacheStats> {
    let metadata = tokio::fs::metadata(path).await?;
    if metadata.len() > MAX_STATS_LOG_BYTES {
        anyhow::bail!(
            "ccache stats log is {} bytes; maximum accepted size is {} bytes",
            metadata.len(),
            MAX_STATS_LOG_BYTES
        );
    }
    let contents = tokio::fs::read_to_string(path).await?;
    Ok(parse_stats_log(&contents))
}

pub(super) async fn remove_stats_log(path: &Path) {
    if let Err(error) = tokio::fs::remove_file(path).await
        && error.kind() != std::io::ErrorKind::NotFound
    {
        tracing::warn!(path = %path.display(), %error, "failed to remove ingested ccache stats log");
    }
}

fn host_stats_log_path(ccache_dir: &Path, job_id: Uuid) -> PathBuf {
    ccache_dir.join(STATS_DIR).join(format!("{job_id}.log"))
}

fn parse_stats_log(contents: &str) -> BuildCcacheStats {
    let mut stats = BuildCcacheStats::default();
    let mut logged_calls = 0_u64;

    for line in contents.lines() {
        if line.starts_with("# ") {
            logged_calls = logged_calls.saturating_add(1);
            continue;
        }
        match line {
            "direct_cache_hit" => stats.direct_hits = stats.direct_hits.saturating_add(1),
            "preprocessed_cache_hit" => {
                stats.preprocessed_hits = stats.preprocessed_hits.saturating_add(1);
            }
            "cache_miss" => stats.cache_misses = stats.cache_misses.saturating_add(1),
            value if is_error(value) => {
                stats.error_calls = stats.error_calls.saturating_add(1);
            }
            value if is_uncacheable(value) => {
                stats.uncacheable_calls = stats.uncacheable_calls.saturating_add(1);
            }
            _ => {}
        }
    }

    let classified = stats
        .direct_hits
        .saturating_add(stats.preprocessed_hits)
        .saturating_add(stats.cache_misses)
        .saturating_add(stats.uncacheable_calls)
        .saturating_add(stats.error_calls);
    if logged_calls > classified {
        stats.uncacheable_calls = stats
            .uncacheable_calls
            .saturating_add(logged_calls - classified);
    }
    stats.compiler_calls = logged_calls.max(classified);
    stats
}

fn is_error(value: &str) -> bool {
    matches!(
        value,
        "bad_input_file"
            | "bad_output_file"
            | "compiler_check_failed"
            | "could_not_find_compiler"
            | "error_hashing_extra_file"
            | "internal_error"
            | "missing_cache_file"
            | "modified_input_file"
    )
}

fn is_uncacheable(value: &str) -> bool {
    matches!(
        value,
        "autoconf_test"
            | "bad_compiler_arguments"
            | "called_for_link"
            | "called_for_preprocessing"
            | "compile_failed"
            | "compiler_produced_no_output"
            | "compiler_produced_empty_output"
            | "compiler_produced_stdout"
            | "could_not_use_modules"
            | "could_not_use_precompiled_header"
            | "disabled"
            | "multiple_source_files"
            | "no_input_file"
            | "output_to_stdout"
            | "preprocessor_error"
            | "recache"
            | "unsupported_code_directive"
            | "unsupported_compiler_option"
            | "unsupported_environment_variable"
            | "unsupported_source_encoding"
            | "unsupported_source_language"
    )
}

#[cfg(test)]
mod tests {
    use super::parse_stats_log;

    #[test]
    fn parses_hits_misses_and_non_cacheable_calls() {
        let stats = parse_stats_log(
            "# first.c\ncache_miss\ndirect_cache_miss\npreprocessed_cache_miss\n\
             # second.c\ndirect_cache_hit\n# link.c\ncalled_for_link\n\
             # broken.c\nbad_input_file\n",
        );

        assert_eq!(stats.compiler_calls, 4);
        assert_eq!(stats.direct_hits, 1);
        assert_eq!(stats.preprocessed_hits, 0);
        assert_eq!(stats.cache_misses, 1);
        assert_eq!(stats.uncacheable_calls, 1);
        assert_eq!(stats.error_calls, 1);
    }

    #[test]
    fn treats_unknown_terminal_results_as_uncacheable() {
        let stats = parse_stats_log("# future.c\nfuture_terminal_result\n");

        assert_eq!(stats.compiler_calls, 1);
        assert_eq!(stats.uncacheable_calls, 1);
    }
}
