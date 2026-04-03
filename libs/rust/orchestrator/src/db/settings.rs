use super::*;

pub(super) async fn get_public_base_url_override(
    store: &DieselStore,
) -> anyhow::Result<Option<String>> {
    store
        .with_connection(|conn| {
            let value = daemon_runtime_settings::table
                .find(1)
                .select(daemon_runtime_settings::public_base_url)
                .first::<Option<String>>(conn)
                .optional()?
                .flatten()
                .filter(|value| !value.trim().is_empty());
            Ok(value)
        })
        .await
}

pub(super) async fn set_public_base_url_override(
    store: &DieselStore,
    value: &str,
) -> anyhow::Result<()> {
    let value = value.trim().to_string();
    store
        .with_connection(move |conn| {
            let row = RuntimeSettingsRecord {
                id: 1,
                public_base_url: Some(value.as_str()),
            };
            diesel::insert_into(daemon_runtime_settings::table)
                .values(&row)
                .on_conflict(daemon_runtime_settings::id)
                .do_update()
                .set(daemon_runtime_settings::public_base_url.eq(row.public_base_url))
                .execute(conn)?;
            Ok(())
        })
        .await
}
