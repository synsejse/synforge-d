use std::fmt;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::SynforgeError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
    pub spec_path: PathBuf,
    pub version: String,
    pub release: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BuildTarget {
    pub distribution: String,
    pub release: String,
    pub arch: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SpecSource {
    pub repo_url: String,
    pub spec_path: String,
    #[serde(default = "default_true")]
    pub poll: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ParsedSpec {
    pub name: String,
    pub version: String,
    pub release: String,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
        if !is_dns_label(&self.name) {
            return Err(SynforgeError::Spec(format!(
                "package name {} is not DNS-label safe",
                self.name
            )));
        }
        if self.mock_chroots.is_empty() {
            return Err(SynforgeError::Spec(
                "at least one mock chroot must be selected".to_string(),
            ));
        }
        for chroot in &self.mock_chroots {
            parse_mock_chroot(chroot).ok_or_else(|| {
                SynforgeError::Spec(format!("mock chroot {} is not a valid mock target name", chroot))
            })?;
        }
        if self.repo_subdir.trim().is_empty() {
            return Err(SynforgeError::Spec(
                "repo_subdir must not be empty".to_string(),
            ));
        }
        if self.version.trim().is_empty() {
            return Err(SynforgeError::Spec(
                "spec version must not be empty".to_string(),
            ));
        }
        if self.poll_interval_seconds == 0 {
            return Err(SynforgeError::Spec(
                "poll_interval_seconds must be greater than zero".to_string(),
            ));
        }
        if self.build_timeout_seconds == 0 {
            return Err(SynforgeError::Spec(
                "build_timeout_seconds must be greater than zero".to_string(),
            ));
        }
        if self.package_history_count == 0 {
            return Err(SynforgeError::Spec(
                "package_history_count must be greater than zero".to_string(),
            ));
        }
        for entry in &self.build_env {
            entry.validate()?;
        }
        if self.release.trim().is_empty() {
            return Err(SynforgeError::Spec(
                "spec release must not be empty".to_string(),
            ));
        }
        if self.spec_path.extension().and_then(|value| value.to_str()) != Some("spec") {
            return Err(SynforgeError::Spec(format!(
                "spec path {} must end with .spec",
                self.spec_path.display()
            )));
        }
        if self.spec_path.is_absolute() {
            return Err(SynforgeError::Spec(
                "spec path must be relative to the git repository root".to_string(),
            ));
        }
        self.source.validate()?;
        Ok(())
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
        let key = self.key.trim();
        if key.is_empty() {
            return Err(SynforgeError::Spec(
                "build environment variable key must not be empty".to_string(),
            ));
        }
        let mut chars = key.chars();
        let Some(first) = chars.next() else {
            return Err(SynforgeError::Spec(
                "build environment variable key must not be empty".to_string(),
            ));
        };
        if !(first.is_ascii_alphabetic() || first == '_') {
            return Err(SynforgeError::Spec(format!(
                "build environment variable key {} must start with a letter or underscore",
                self.key
            )));
        }
        if !chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_') {
            return Err(SynforgeError::Spec(format!(
                "build environment variable key {} contains invalid characters",
                self.key
            )));
        }
        Ok(())
    }
}

impl SpecSource {
    pub fn validate(&self) -> Result<(), SynforgeError> {
        if self.repo_url.trim().is_empty() {
            return Err(SynforgeError::Spec(
                "git repository URL must not be empty".to_string(),
            ));
        }
        if self.spec_path.trim().is_empty() {
            return Err(SynforgeError::Spec(
                "spec path must not be empty".to_string(),
            ));
        }
        let spec_path = PathBuf::from(self.spec_path.trim());
        if spec_path.is_absolute() {
            return Err(SynforgeError::Spec(
                "spec path must be relative to the repository root".to_string(),
            ));
        }
        if spec_path.extension().and_then(|value| value.to_str()) != Some("spec") {
            return Err(SynforgeError::Spec(
                "spec path must point to a .spec file".to_string(),
            ));
        }
        Ok(())
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
