//! Stateful coordination types for repository workflows.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoSigningKeyIdentity {
    pub key_id: String,
    pub fingerprint: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RepoSigningSettingsUpdate {
    pub signing_enabled: Option<bool>,
    pub signing_key_id: Option<Option<String>>,
    pub armored_private_key: Option<Option<String>>,
}
