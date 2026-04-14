use synforge_core::model::UserPermission;

pub(crate) fn validate_user_handle(handle: &str) -> anyhow::Result<()> {
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

pub(crate) fn validate_display_name(display_name: &str) -> anyhow::Result<()> {
    if display_name.trim().is_empty() {
        anyhow::bail!("display_name must not be empty");
    }
    Ok(())
}

pub(crate) fn validate_password(password: &str) -> anyhow::Result<()> {
    if password.is_empty() {
        anyhow::bail!("password must not be empty");
    }
    Ok(())
}

pub(crate) fn validate_permissions(permissions: &[UserPermission]) -> anyhow::Result<()> {
    if permissions.is_empty() {
        anyhow::bail!("at least one permission is required");
    }
    Ok(())
}
