use crate::api::{ConfigFieldDescriptor, ConfigFieldType};
use serde_json::Value;

#[derive(Copy, Clone)]
pub(super) struct ConfigSection<'a> {
    pub(super) key: &'a str,
    pub(super) label: &'a str,
}

#[derive(Copy, Clone)]
pub(super) struct ConfigEditability {
    pub(super) in_setup: bool,
    pub(super) in_runtime: bool,
    pub(super) restart_required: bool,
}

pub(super) fn config_string_field(
    section: ConfigSection<'_>,
    key: &str,
    label: &str,
    description: &str,
    default_value: &str,
    editability: ConfigEditability,
) -> ConfigFieldDescriptor {
    ConfigFieldDescriptor {
        key: key.to_string(),
        label: label.to_string(),
        description: description.to_string(),
        section_key: section.key.to_string(),
        section_label: section.label.to_string(),
        field_type: ConfigFieldType::String,
        required: true,
        min_value: None,
        editable_in_setup: editability.in_setup,
        editable_in_runtime: editability.in_runtime,
        restart_required: editability.restart_required,
        default_value: Value::String(default_value.to_string()),
    }
}

pub(super) fn config_optional_string_field(
    section: ConfigSection<'_>,
    key: &str,
    label: &str,
    description: &str,
    editability: ConfigEditability,
) -> ConfigFieldDescriptor {
    ConfigFieldDescriptor {
        key: key.to_string(),
        label: label.to_string(),
        description: description.to_string(),
        section_key: section.key.to_string(),
        section_label: section.label.to_string(),
        field_type: ConfigFieldType::String,
        required: false,
        min_value: None,
        editable_in_setup: editability.in_setup,
        editable_in_runtime: editability.in_runtime,
        restart_required: editability.restart_required,
        default_value: Value::Null,
    }
}

pub(super) fn config_number_field(
    section: ConfigSection<'_>,
    key: &str,
    label: &str,
    description: &str,
    default_value: u64,
    editability: ConfigEditability,
) -> ConfigFieldDescriptor {
    ConfigFieldDescriptor {
        key: key.to_string(),
        label: label.to_string(),
        description: description.to_string(),
        section_key: section.key.to_string(),
        section_label: section.label.to_string(),
        field_type: ConfigFieldType::Number,
        required: true,
        min_value: Some(1),
        editable_in_setup: editability.in_setup,
        editable_in_runtime: editability.in_runtime,
        restart_required: editability.restart_required,
        default_value: Value::Number(default_value.into()),
    }
}

pub(super) fn config_bool_field(
    section: ConfigSection<'_>,
    key: &str,
    label: &str,
    description: &str,
    default_value: bool,
    editability: ConfigEditability,
) -> ConfigFieldDescriptor {
    ConfigFieldDescriptor {
        key: key.to_string(),
        label: label.to_string(),
        description: description.to_string(),
        section_key: section.key.to_string(),
        section_label: section.label.to_string(),
        field_type: ConfigFieldType::Boolean,
        required: true,
        min_value: None,
        editable_in_setup: editability.in_setup,
        editable_in_runtime: editability.in_runtime,
        restart_required: editability.restart_required,
        default_value: Value::Bool(default_value),
    }
}
