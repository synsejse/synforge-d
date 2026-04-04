use utoipa::OpenApi;

fn main() -> anyhow::Result<()> {
    let doc = synforge_serve::openapi::ApiDoc::openapi();
    println!("{}", serde_json::to_string_pretty(&doc)?);
    Ok(())
}
