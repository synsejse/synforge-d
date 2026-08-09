use crate::{
    error::SynforgeError,
    package::{is_dns_label, is_safe_path_segment, parse_mock_chroot},
};

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
        if !is_safe_path_segment(value) {
            return Err(SynforgeError::Spec(format!(
                "package name {value:?} must be a single safe path segment"
            )));
        }
        Ok(Self(value.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::PackageName;

    #[test]
    fn package_name_preserves_alias_independent_of_rpm_name() {
        let alias = PackageName::new("XYZ").expect("valid package alias");
        assert_eq!(alias.as_str(), "XYZ");
    }

    #[test]
    fn package_name_rejects_path_traversal() {
        for value in ["../XYZ", "/XYZ", "nested/XYZ", "..", "XYZ:ro"] {
            assert!(PackageName::new(value).is_err(), "accepted {value:?}");
        }
    }
}
