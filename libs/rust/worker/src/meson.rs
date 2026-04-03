use std::path::Path;

use ini::Ini;

use crate::logging::BuildLogger;

pub(crate) async fn apply_spec_compat_fixes(spec_path: &Path) -> anyhow::Result<()> {
    let mut contents = tokio::fs::read_to_string(spec_path).await?;

    if contents.contains("crate(rustc-hash)")
        && contents.contains("%define rewrite_wrap_file()")
        && !contents.contains("%rewrite_wrap_file rustc-hash")
    {
        let needle = "%rewrite_wrap_file unicode-ident\n";
        if contents.contains(needle) {
            contents = contents.replace(
                needle,
                "%rewrite_wrap_file unicode-ident\n%rewrite_wrap_file rustc-hash\n",
            );
            tokio::fs::write(spec_path, contents).await?;
        }
    }

    Ok(())
}

pub(crate) async fn rewrite_meson_rust_wraps(
    package_dir: &Path,
    logger: &BuildLogger,
) -> anyhow::Result<()> {
    let subprojects_dir = package_dir.join("subprojects");
    if !tokio::fs::try_exists(&subprojects_dir).await? {
        logger.line("No subprojects directory detected").await?;
        return Ok(());
    }

    let registry_dir = Path::new("/usr/share/cargo/registry");
    if !tokio::fs::try_exists(registry_dir).await? {
        logger.line("Cargo registry cache not available").await?;
        return Ok(());
    }

    let mut registry_entries = Vec::new();
    let mut registry_read_dir = tokio::fs::read_dir(registry_dir).await?;
    while let Some(entry) = registry_read_dir.next_entry().await? {
        if entry.file_type().await?.is_dir() {
            if let Some(name) = entry.file_name().to_str() {
                registry_entries.push(name.to_string());
            }
        }
    }
    registry_entries.sort();

    let mut read_dir = tokio::fs::read_dir(&subprojects_dir).await?;
    while let Some(entry) = read_dir.next_entry().await? {
        let path = entry.path();
        if !entry.file_type().await?.is_file() {
            continue;
        }
        let Some(file_name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        if !file_name.ends_with("-rs.wrap") {
            continue;
        }

        let Some(stem) = file_name.strip_suffix(".wrap") else {
            continue;
        };
        let Some(crate_prefix) = stem.strip_suffix("-rs") else {
            continue;
        };
        let Some(directory_name) = find_registry_directory(&registry_entries, crate_prefix) else {
            continue;
        };

        let contents = tokio::fs::read_to_string(&path).await?;
        let rewritten = rewrite_wrap_contents(&contents, directory_name)?;

        if rewritten != contents {
            logger
                .line(format!(
                    "Rewriting {} to use directory = {}",
                    path.display(),
                    directory_name
                ))
                .await?;
            tokio::fs::write(&path, rewritten).await?;
        }
    }

    Ok(())
}

pub(crate) fn rewrite_wrap_contents(
    contents: &str,
    directory_name: &str,
) -> anyhow::Result<String> {
    let mut ini = Ini::load_from_str(contents)
        .map_err(|error| anyhow::anyhow!("failed to parse Meson wrap file: {}", error))?;
    let sections = ini
        .iter()
        .filter_map(|(section, _)| section.as_ref().map(ToString::to_string))
        .collect::<Vec<_>>();

    for section in sections {
        if let Some(properties) = ini.section_mut(Some(section.clone())) {
            let source_keys = properties
                .iter()
                .map(|(key, _)| key.to_string())
                .filter(|key| key.starts_with("source_"))
                .collect::<Vec<_>>();
            for key in source_keys {
                properties.remove(&key);
            }
            properties.insert("directory".to_string(), directory_name.to_string());
        }
    }

    let mut rendered = Vec::new();
    ini.write_to(&mut rendered)?;
    Ok(String::from_utf8(rendered)?)
}

fn find_registry_directory<'a>(
    registry_entries: &'a [String],
    crate_prefix: &str,
) -> Option<&'a str> {
    let prefix = format!("{crate_prefix}.");
    registry_entries
        .iter()
        .find(|entry| entry.starts_with(&prefix))
        .map(String::as_str)
}
