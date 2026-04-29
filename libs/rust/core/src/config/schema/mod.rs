mod builders;
mod sections;

use crate::api::ConfigFieldDescriptor;

pub fn editable_config_fields() -> Vec<ConfigFieldDescriptor> {
    let mut fields = Vec::new();
    fields.extend(sections::server_fields());
    fields.extend(sections::worker_fields());
    fields.extend(sections::signing_fields());
    fields.extend(sections::build_fields());
    fields.extend(sections::database_fields());
    fields.extend(sections::scheduler_fields());
    fields.extend(sections::git_fields());
    fields.extend(sections::cache_fields());
    fields
}
