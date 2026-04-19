use super::*;

impl DieselStore {
    pub async fn health_check(&self) -> anyhow::Result<()> {
        let mut conn = self.get_connection().await?;
        packages::table.count().get_result::<i64>(&mut conn).await?;
        Ok(())
    }

    pub async fn list_runtime_settings(
        &self,
    ) -> anyhow::Result<BTreeMap<String, serde_json::Value>> {
        let mut conn = self.get_connection().await?;
        let rows = runtime_settings::table
            .select((runtime_settings::key, runtime_settings::value_json))
            .load::<(String, serde_json::Value)>(&mut conn)
            .await?;
        let mut settings = BTreeMap::new();
        for (key, value_json) in rows {
            settings.insert(key, value_json);
        }
        Ok(settings)
    }

    pub async fn upsert_runtime_settings(
        &self,
        settings: BTreeMap<String, serde_json::Value>,
    ) -> anyhow::Result<()> {
        let mut conn = self.get_connection().await?;
        let updated_at = now_utc();
        for (key, value) in settings {
            let existing = runtime_settings::table
                .find(key.as_str())
                .select(runtime_settings::key)
                .first::<String>(&mut conn)
                .await
                .optional()?;
            if existing.is_some() {
                diesel::update(runtime_settings::table.find(key.as_str()))
                    .set((
                        runtime_settings::value_json.eq(value.clone()),
                        runtime_settings::updated_at.eq(updated_at),
                    ))
                    .execute(&mut conn)
                    .await?;
            } else {
                let row = NewRuntimeSettingRecord {
                    key: key.as_str(),
                    value_json: value,
                    updated_at,
                };
                diesel::insert_into(runtime_settings::table)
                    .values(&row)
                    .execute(&mut conn)
                    .await?;
            }
        }
        Ok(())
    }

    pub async fn update_build_artifact_metadata(
        &self,
        artifact_id: Uuid,
        sha256: String,
        size_bytes: u64,
    ) -> anyhow::Result<()> {
        let mut conn = self.get_connection().await?;
        diesel::update(build_artifacts::table.find(artifact_id))
            .set((
                build_artifacts::sha256.eq(sha256.as_str()),
                build_artifacts::size_bytes.eq(size_bytes as i64),
            ))
            .execute(&mut conn)
            .await?;
        Ok(())
    }

    pub async fn upsert_artifact_signatures(
        &self,
        signatures: Vec<ArtifactSignature>,
    ) -> anyhow::Result<()> {
        let mut conn = self.get_connection().await?;
        let updated_at = now_utc();
        for signature in signatures {
            let existing = artifact_signatures::table
                .find(signature.artifact_id)
                .select(artifact_signatures::artifact_id)
                .first::<Uuid>(&mut conn)
                .await
                .optional()?;
            if existing.is_some() {
                diesel::update(artifact_signatures::table.find(signature.artifact_id))
                    .set((
                        artifact_signatures::status.eq(signature.status),
                        artifact_signatures::signed_at.eq(signature.signed_at),
                        artifact_signatures::key_id.eq(signature.key_id.as_deref()),
                        artifact_signatures::fingerprint.eq(signature.fingerprint.as_deref()),
                        artifact_signatures::error_message.eq(signature.error_message.as_deref()),
                        artifact_signatures::updated_at.eq(updated_at),
                    ))
                    .execute(&mut conn)
                    .await?;
            } else {
                let row = NewArtifactSignatureRecord {
                    artifact_id: signature.artifact_id,
                    status: signature.status,
                    signed_at: signature.signed_at,
                    key_id: signature.key_id,
                    fingerprint: signature.fingerprint,
                    error_message: signature.error_message,
                    updated_at,
                };
                diesel::insert_into(artifact_signatures::table)
                    .values(&row)
                    .execute(&mut conn)
                    .await?;
            }
        }
        Ok(())
    }
}
