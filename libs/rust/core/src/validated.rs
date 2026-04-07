use crate::{
    error::SynforgeError,
    package::{is_dns_label, parse_mock_chroot},
};
use slug::slugify;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DnsLabel(String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoUrl(String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthToken(String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MockChroot(String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageName(String);

impl DnsLabel {
    pub fn new(value: &str) -> Result<Self, SynforgeError> {
        let value = value.trim();
        if !is_dns_label(value) {
            return Err(SynforgeError::Spec(format!(
                "value {} is not DNS-label safe",
                value
            )));
        }
        Ok(Self(value.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl RepoUrl {
    pub fn new(value: &str) -> Result<Self, SynforgeError> {
        let value = value.trim();
        if value.is_empty() {
            return Err(SynforgeError::Spec(
                "git repository URL must not be empty".to_string(),
            ));
        }
        let valid_prefix = value.starts_with("http://")
            || value.starts_with("https://")
            || value.starts_with("ssh://")
            || value.starts_with("git@");
        if !valid_prefix {
            return Err(SynforgeError::Spec(format!(
                "git repository URL {} must use http(s), ssh, or git@ syntax",
                value
            )));
        }
        Ok(Self(value.to_string()))
    }
}

impl AuthToken {
    pub fn new(value: &str) -> Result<Self, SynforgeError> {
        let value = value.trim();
        if value.is_empty() {
            return Err(SynforgeError::Unauthorized);
        }
        Ok(Self(value.to_string()))
    }

    pub fn secret(&self) -> &str {
        &self.0
    }
}

impl MockChroot {
    pub fn new(value: &str) -> Result<Self, SynforgeError> {
        let value = value.trim();
        if parse_mock_chroot(value).is_none() {
            return Err(SynforgeError::Spec(format!(
                "mock chroot {} is not a valid mock target name",
                value
            )));
        }
        Ok(Self(value.to_string()))
    }
}

impl PackageName {
    pub fn new(value: &str) -> Result<Self, SynforgeError> {
        let normalized = slugify(value);
        if normalized.is_empty() {
            return Err(SynforgeError::Spec(
                "package name must not be empty".to_string(),
            ));
        }
        DnsLabel::new(&normalized).map_err(|_| {
            SynforgeError::Spec(format!("package name {} is not DNS-label safe", value))
        })?;
        Ok(Self(normalized))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::PackageName;

    #[test]
    fn package_name_is_normalized_with_slug_rules() {
        assert_eq!(
            PackageName::new("Hello.World+RPM")
                .expect("package name should normalize")
                .as_str(),
            "hello-world-rpm"
        );
        assert_eq!(
            PackageName::new("Æúű--cool?")
                .expect("unicode package name should normalize")
                .as_str(),
            "aeuu-cool"
        );
    }

    #[test]
    fn package_name_rejects_empty_normalization() {
        assert!(PackageName::new("$$$").is_err());
    }
}
