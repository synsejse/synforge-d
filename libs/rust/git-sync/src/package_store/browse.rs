use std::time::Duration;

use synforge_core::api::BrowseRepositoryResponse;
use tracing::instrument;

use super::PackageSyncStore;
use super::git::{run_git, run_git_clone};

impl PackageSyncStore {
    #[instrument(skip(self), fields(repo_url = %repo_url))]
    pub async fn browse_repository(
        &self,
        repo_url: &str,
    ) -> anyhow::Result<BrowseRepositoryResponse> {
        let repo_url = repo_url.trim();
        if repo_url.is_empty() {
            anyhow::bail!("repository URL must not be empty");
        }

        let paths = self.config.runtime_paths();
        let clone_dir = paths.repo_browse_workspace_dir(uuid::Uuid::now_v7());
        if let Some(parent) = clone_dir.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let git_timeout = Duration::from_secs(self.config.git_operation_timeout_seconds);
        let clone_result: anyhow::Result<()> =
            match self.git_mirror_cache.ensure_mirror(repo_url).await {
                Ok(mirror_dir) => {
                    let mirror_path = mirror_dir.to_string_lossy().to_string();
                    let mirror_clone = run_git_clone(
                        None,
                        &[
                            "clone",
                            "--progress",
                            "--depth",
                            "1",
                            mirror_path.as_str(),
                            clone_dir.to_string_lossy().as_ref(),
                        ],
                        git_timeout,
                    )
                    .await;
                    if let Err(error) = mirror_clone {
                        tracing::warn!(
                            repo_url,
                            mirror_dir = %mirror_dir.display(),
                            error = %error,
                            "failed to clone from mirror cache; falling back to direct clone"
                        );
                        if let Err(cleanup_error) = tokio::fs::remove_dir_all(&clone_dir).await {
                            tracing::warn!(
                                repo_url,
                                path = %clone_dir.display(),
                                error = %cleanup_error,
                                "failed to cleanup failed mirror clone workspace before retry"
                            );
                        }
                        run_git_clone(
                            None,
                            &[
                                "clone",
                                "--progress",
                                "--depth",
                                "1",
                                repo_url,
                                clone_dir.to_string_lossy().as_ref(),
                            ],
                            git_timeout,
                        )
                        .await
                    } else {
                        Ok(())
                    }
                }
                Err(error) => {
                    tracing::warn!(
                        repo_url,
                        error = %error,
                        "failed to prepare mirror cache; falling back to direct clone"
                    );
                    run_git_clone(
                        None,
                        &[
                            "clone",
                            "--progress",
                            "--depth",
                            "1",
                            repo_url,
                            clone_dir.to_string_lossy().as_ref(),
                        ],
                        git_timeout,
                    )
                    .await
                }
            };

        let response: anyhow::Result<BrowseRepositoryResponse> = async {
            clone_result?;
            let head_commit =
                run_git(Some(&clone_dir), &["rev-parse", "HEAD"], git_timeout).await?;
            let files_output = run_git(Some(&clone_dir), &["ls-files"], git_timeout).await?;
            let mut files = files_output
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .map(ToOwned::to_owned)
                .collect::<Vec<_>>();
            files.sort();
            let spec_files = files
                .iter()
                .filter(|path| path.ends_with(".spec"))
                .cloned()
                .collect::<Vec<_>>();

            let response = BrowseRepositoryResponse {
                repo_url: repo_url.to_string(),
                head_commit: head_commit.trim().to_string(),
                files,
                spec_files,
            };
            Ok(response)
        }
        .await;

        if let Err(cleanup_error) = tokio::fs::remove_dir_all(&clone_dir).await {
            tracing::warn!(
                repo_url,
                path = %clone_dir.display(),
                error = %cleanup_error,
                "failed to cleanup repository browse workspace"
            );
        }
        response
    }
}
