use std::fmt;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::error::SynforgeError;
use crate::validation::{BuildEnvVarValidator, PackageDefinitionValidator, SpecSourceValidator, Validator};

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
    #[serde(default)]
    pub build_env: Vec<BuildEnvVar>,
    #[schema(value_type = String)]
    pub spec_path: PathBuf,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct SpecSource {
    pub repo_url: String,
    pub spec_path: String,
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
}

fn default_true() -> bool {
    true
}

impl PackageDefinition {
    pub fn validate(&self) -> Result<(), SynforgeError> {
        PackageDefinitionValidator.validate(self)
    }

    pub fn revision(&self) -> SpecRevision {
        SpecRevision {
            version: self.version.clone(),
            release: self.release.clone(),
            content_digest: None,
        }
    }
}

impl BuildEnvVar {
    pub fn validate(&self) -> Result<(), SynforgeError> {
        BuildEnvVarValidator.validate(self)
    }
}

impl SpecSource {
    pub fn validate(&self) -> Result<(), SynforgeError> {
        SpecSourceValidator.validate(self)
    }

    pub fn polling_enabled(&self) -> bool {
        self.poll
    }
}

impl ParsedSpec {
    pub fn revision(&self) -> SpecRevision {
        SpecRevision {
            version: self.version.clone(),
            release: self.release.clone(),
            content_digest: None,
        }
    }
}

impl SpecRevision {
    pub fn new(
        version: impl Into<String>,
        release: impl Into<String>,
        content_digest: Option<String>,
    ) -> Self {
        Self {
            version: version.into(),
            release: release.into(),
            content_digest,
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
    if value.is_empty() || value.len() > 63 {
        return false;
    }
    let bytes = value.as_bytes();
    if !bytes[0].is_ascii_lowercase() && !bytes[0].is_ascii_digit() {
        return false;
    }
    if !bytes[bytes.len() - 1].is_ascii_lowercase() && !bytes[bytes.len() - 1].is_ascii_digit() {
        return false;
    }
    value
        .bytes()
        .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

pub fn normalize_package_name(value: &str) -> String {
    let mut normalized = String::with_capacity(value.len());
    let mut last_was_dash = false;

    for ch in value.chars() {
        let mapped = match ch {
            'a'..='z' | '0'..='9' => Some(ch),
            'A'..='Z' => Some(ch.to_ascii_lowercase()),
            '-' | '_' | '.' | '+' => Some('-'),
            _ => None,
        };

        let Some(mapped) = mapped else {
            continue;
        };

        if mapped == '-' {
            if normalized.is_empty() || last_was_dash {
                continue;
            }
            last_was_dash = true;
        } else {
            last_was_dash = false;
        }

        normalized.push(mapped);
    }

    while normalized.ends_with('-') {
        normalized.pop();
    }

    normalized
}
