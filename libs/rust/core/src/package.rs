use std::fmt;
use std::path::{Component, Path, PathBuf};

use idna::domain_to_ascii_strict;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct PackageDefinition {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    pub repo_subdir: String,
    #[serde(default = "default_publish_srpm")]
    pub publish_srpm: bool,
    #[serde(default = "default_publish_debuginfo")]
    pub publish_debuginfo: bool,
    #[serde(default)]
    pub network_access: bool,
    #[serde(default)]
    pub mock_chroots: Vec<String>,
    pub source: SpecSource,
    #[serde(default = "default_poll_interval_seconds")]
    pub poll_interval_seconds: u64,
    #[serde(default = "default_build_timeout_seconds")]
    pub build_timeout_seconds: u64,
    #[serde(default = "default_package_history_count")]
    pub package_history_count: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cpu_limit_millicores: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memory_limit_mb: Option<u64>,
    #[serde(default)]
    pub ccache_enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ccache_max_size_mb: Option<u64>,
    #[serde(default)]
    pub build_env: Vec<BuildEnvVar>,
    #[schema(value_type = String)]
    pub spec_file: PathBuf,
    pub version: String,
    pub release: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct BuildEnvVar {
    pub key: String,
    pub value: String,
}

fn default_enabled() -> bool {
    true
}

fn default_publish_srpm() -> bool {
    true
}

fn default_publish_debuginfo() -> bool {
    true
}

fn default_poll_interval_seconds() -> u64 {
    900
}

fn default_build_timeout_seconds() -> u64 {
    7200
}

fn default_package_history_count() -> u64 {
    3
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct BuildTarget {
    pub distribution: String,
    pub release: String,
    pub arch: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, ToSchema)]
pub struct RepoTarget {
    pub distribution: String,
    pub release: String,
}

impl BuildTarget {
    pub fn repo_target(&self) -> RepoTarget {
        RepoTarget {
            distribution: self.distribution.clone(),
            release: self.release.clone(),
        }
    }
}

impl RepoTarget {
    pub fn repo_subdir(&self) -> PathBuf {
        PathBuf::from(&self.distribution).join(&self.release)
    }

    pub fn from_mock_chroot(value: &str) -> Option<Self> {
        parse_mock_chroot(value).map(|target| target.repo_target())
    }

    pub fn from_repo_relative_path(path: &Path) -> Option<Self> {
        let mut parts = path.components().filter_map(|component| match component {
            Component::Normal(part) => part.to_str(),
            _ => None,
        });
        let distribution = parts.next()?;
        let release = parts.next()?;
        Some(Self {
            distribution: distribution.to_string(),
            release: release.to_string(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct SpecSource {
    pub repo_url: String,
    pub spec_file: String,
    #[serde(default = "default_true")]
    pub poll: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct ParsedSpec {
    pub name: String,
    pub version: String,
    pub release: String,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct SpecRevision {
    pub version: String,
    pub release: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_digest: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_commit: Option<String>,
}

fn default_true() -> bool {
    true
}

impl SpecRevision {
    pub fn new(
        version: impl Into<String>,
        release: impl Into<String>,
        content_digest: Option<String>,
        source_commit: Option<String>,
    ) -> Self {
        Self {
            version: version.into(),
            release: release.into(),
            content_digest,
            source_commit,
        }
    }

    pub fn comparison_key(&self) -> String {
        match &self.content_digest {
            Some(digest) => format!("{}-{}@{}", self.version.trim(), self.release.trim(), digest),
            None => format!("{}-{}", self.version.trim(), self.release.trim()),
        }
    }
}

pub fn parse_mock_chroot(value: &str) -> Option<BuildTarget> {
    let value = value.trim();
    if value.is_empty() || value.ends_with(".cfg") {
        return None;
    }
    let mut parts = value.rsplitn(3, '-');
    let arch = parts.next()?;
    let release = parts.next()?;
    let distribution = parts.next()?;
    if distribution.is_empty() || release.is_empty() || arch.is_empty() {
        return None;
    }
    Some(BuildTarget {
        distribution: distribution.to_string(),
        release: release.to_string(),
        arch: arch.to_string(),
    })
}

impl fmt::Display for SpecRevision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.comparison_key())
    }
}

pub fn is_dns_label(value: &str) -> bool {
    if value.is_empty() {
        return false;
    }

    let Ok(ascii) = domain_to_ascii_strict(value) else {
        return false;
    };

    ascii == value && !ascii.contains('.') && ascii.len() <= 63
}
