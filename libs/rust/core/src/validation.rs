use std::path::{Component, Path, PathBuf};

use crate::{
    error::SynforgeError,
    model::UserPermission,
    package::{BuildEnvVar, PackageDefinition, SpecSource},
    validated::{MockChroot, PackageName, RepoUrl},
};
use path_clean::PathClean;

pub trait Validator<T> {
    fn validate(&self, value: &T) -> Result<(), SynforgeError>;
}

pub struct PackageDefinitionValidator;
pub struct BuildEnvVarValidator;
pub struct SpecSourceValidator;

pub fn validate_user_handle(handle: &str) -> anyhow::Result<()> {
    if handle.is_empty() {
        anyhow::bail!("user handle must not be empty");
    }
    if !handle
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-'))
    {
        anyhow::bail!("user handle may only contain letters, digits, '.', '_' and '-'");
    }
    Ok(())
}

pub fn validate_display_name(display_name: &str) -> anyhow::Result<()> {
    if display_name.trim().is_empty() {
        anyhow::bail!("display_name must not be empty");
    }
    Ok(())
}

pub fn validate_password(password: &str) -> anyhow::Result<()> {
    if password.is_empty() {
        anyhow::bail!("password must not be empty");
    }
    Ok(())
}

pub fn validate_permissions(permissions: &[UserPermission]) -> anyhow::Result<()> {
    if permissions.is_empty() {
        anyhow::bail!("at least one permission is required");
    }
    Ok(())
}

impl Validator<PackageDefinition> for PackageDefinitionValidator {
    fn validate(&self, value: &PackageDefinition) -> Result<(), SynforgeError> {
        PackageName::new(&value.name)?;
        if value.mock_chroots.is_empty() {
            return Err(SynforgeError::Spec(
                "at least one mock chroot must be selected".to_string(),
            ));
        }
        for chroot in &value.mock_chroots {
            MockChroot::new(chroot)?;
        }
        if value.repo_subdir.trim().is_empty() {
            return Err(SynforgeError::Spec(
                "repo_subdir must not be empty".to_string(),
            ));
        }
        if value.version.trim().is_empty() {
            return Err(SynforgeError::Spec(
                "spec version must not be empty".to_string(),
            ));
        }
        if value.poll_interval_seconds == 0 {
            return Err(SynforgeError::Spec(
                "poll_interval_seconds must be greater than zero".to_string(),
            ));
        }
        if value.build_timeout_seconds == 0 {
            return Err(SynforgeError::Spec(
                "build_timeout_seconds must be greater than zero".to_string(),
            ));
        }
        if value.package_history_count == 0 {
            return Err(SynforgeError::Spec(
                "package_history_count must be greater than zero".to_string(),
            ));
        }
        if let Some(cpu_limit_millicores) = value.cpu_limit_millicores
            && cpu_limit_millicores == 0
        {
            return Err(SynforgeError::Spec(
                "cpu_limit_millicores must be greater than zero when set".to_string(),
            ));
        }
        if let Some(memory_limit_mb) = value.memory_limit_mb
            && memory_limit_mb == 0
        {
            return Err(SynforgeError::Spec(
                "memory_limit_mb must be greater than zero when set".to_string(),
            ));
        }
        if let Some(ccache_max_size_mb) = value.ccache_max_size_mb
            && ccache_max_size_mb == 0
        {
            return Err(SynforgeError::Spec(
                "ccache_max_size_mb must be greater than zero when set".to_string(),
            ));
        }
        for entry in &value.build_env {
            BuildEnvVarValidator.validate(entry)?;
        }
        if value.release.trim().is_empty() {
            return Err(SynforgeError::Spec(
                "spec release must not be empty".to_string(),
            ));
        }
        if value.spec_file.extension().and_then(|part| part.to_str()) != Some("spec") {
            return Err(SynforgeError::Spec(format!(
                "spec path {} must end with .spec",
                value.spec_file.display()
            )));
        }
        if value.spec_file.is_absolute() {
            return Err(SynforgeError::Spec(
                "spec path must be relative to the git repository root".to_string(),
            ));
        }
        validate_repository_relative_path(&value.spec_file, "spec path")?;
        SpecSourceValidator.validate(&value.source)?;
        Ok(())
    }
}

impl Validator<BuildEnvVar> for BuildEnvVarValidator {
    fn validate(&self, value: &BuildEnvVar) -> Result<(), SynforgeError> {
        let key = value.key.trim();
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
                value.key
            )));
        }
        if !chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_') {
            return Err(SynforgeError::Spec(format!(
                "build environment variable key {} contains invalid characters",
                value.key
            )));
        }
        Ok(())
    }
}

impl Validator<SpecSource> for SpecSourceValidator {
    fn validate(&self, value: &SpecSource) -> Result<(), SynforgeError> {
        let _ = RepoUrl::new(&value.repo_url)?;
        if value.spec_file.trim().is_empty() {
            return Err(SynforgeError::Spec(
                "spec path must not be empty".to_string(),
            ));
        }
        let spec_file = PathBuf::from(value.spec_file.trim());
        if spec_file.is_absolute() {
            return Err(SynforgeError::Spec(
                "spec path must be relative to the repository root".to_string(),
            ));
        }
        validate_repository_relative_path(&spec_file, "spec path")?;
        if spec_file.extension().and_then(|part| part.to_str()) != Some("spec") {
            return Err(SynforgeError::Spec(
                "spec path must point to a .spec file".to_string(),
            ));
        }
        Ok(())
    }
}

fn validate_repository_relative_path(path: &Path, label: &str) -> Result<(), SynforgeError> {
    if path.as_os_str().is_empty()
        || path.is_absolute()
        || path.clean() != path
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(SynforgeError::Spec(format!(
            "{label} must stay within the git repository root"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::validate_repository_relative_path;

    #[test]
    fn repository_relative_path_accepts_nested_spec() {
        assert!(validate_repository_relative_path(Path::new("rpm/package.spec"), "spec").is_ok());
    }

    #[test]
    fn repository_relative_path_rejects_non_canonical_or_escaping_paths() {
        for path in [
            "",
            "/tmp/package.spec",
            "../package.spec",
            "rpm/../../package.spec",
            "./package.spec",
        ] {
            assert!(
                validate_repository_relative_path(Path::new(path), "spec").is_err(),
                "accepted {path:?}"
            );
        }
    }
}
