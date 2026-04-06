import { useEffect, useState, type FormEvent } from "react";
import { faSave, faServer } from "@fortawesome/free-solid-svg-icons";
import api from "../../lib/api";
import type { ConfigFieldDescriptor, DaemonConfig } from "../../lib/types";
import ErrorMessage from "../common/ErrorMessage";
import LoadingBlock from "../ui/LoadingBlock";
import FaIcon from "../ui/FaIcon";
import Button from "../ui/Button";
import PageHeader from "../ui/PageHeader";

export default function Settings() {
  const [config, setConfig] = useState<DaemonConfig | null>(null);
  const [schema, setSchema] = useState<ConfigFieldDescriptor[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [values, setValues] = useState<Record<string, string>>({});
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    async function load() {
      try {
        const [configRes, schemaRes] = await Promise.all([
          api.getConfig(),
          api.getConfigSchema(),
        ]);
        setConfig(configRes.config);
        setSchema(schemaRes.fields);
        setValues(buildFieldValues(configRes.config, schemaRes.fields, false));
      } catch (e) {
        setError(e instanceof Error ? e.message : "Failed to load config");
      } finally {
        setLoading(false);
      }
    }
    load();
  }, []);

  async function handleSave(event: FormEvent) {
    event.preventDefault();
    setSaving(true);
    try {
      const runtimeFields = schema.filter((field) => field.editable_in_runtime);
      const res = await api.updateRuntimeSettings({
        settings: buildSettingsPayload(runtimeFields, values),
      });
      setConfig(res.config);
      setValues(buildFieldValues(res.config, schema, false));
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to update settings");
    } finally {
      setSaving(false);
    }
  }

  if (loading) {
    return <LoadingBlock label="Loading config…" lines={4} />;
  }

  if (error || !config) {
    return <ErrorMessage message={error || "Failed to load"} />;
  }

  const groupedFields = groupConfigFields(schema);

  return (
    <div className="space-y-8">
      {/* Header */}
      <PageHeader
        eyebrow="DAEMON_SETTINGS"
        title="Configuration"
        description="Runtime settings and effective daemon values."
        color="purple"
        actions={[{ href: "/", label: "Overview", icon: faServer }]}
      />

      {/* Settings Form */}
      <form onSubmit={handleSave} className="space-y-6">
        {groupedFields.map((section) => {
          const runtimeFields = section.fields.filter((field) => field.editable_in_runtime);
          const readOnlyFields = section.fields.filter((field) => !field.editable_in_runtime);

          if (runtimeFields.length === 0 && readOnlyFields.length === 0) {
            return null;
          }

          return (
            <div key={section.key} className="border-2 border-white bg-black">
              <div className="border-b-2 border-zinc-800 bg-black px-6 py-5">
                <h2 className="font-mono text-lg font-bold uppercase text-white">
                  {section.label}
                </h2>
              </div>
              <div className="grid gap-6 p-6 xl:grid-cols-2">
                {runtimeFields.map((field) => (
                  <ConfigFieldInput
                    key={field.key}
                    field={field}
                    value={values[field.key] || ""}
                    onChange={(next) =>
                      setValues((current) => ({
                        ...current,
                        [field.key]: next,
                      }))
                    }
                  />
                ))}
                {readOnlyFields.map((field) => (
                  <ConfigFieldInput
                    key={field.key}
                    field={field}
                    value={values[field.key] || ""}
                    onChange={() => undefined}
                    disabled
                  />
                ))}
              </div>
            </div>
          );
        })}

        {/* Save Button */}
        <div className="flex justify-end">
          <Button type="submit" variant="primary" size="md" disabled={saving}>
            <FaIcon icon={faSave} className="mr-2" />
            {saving ? "Saving…" : "Save Settings"}
          </Button>
        </div>
      </form>
    </div>
  );
}

function ConfigFieldInput({
  field,
  value,
  onChange,
  disabled = false,
}: {
  field: ConfigFieldDescriptor;
  value: string;
  onChange: (next: string) => void;
  disabled?: boolean;
}) {
  return (
    <label className="block">
      <span className="mb-2 block font-mono text-xs font-bold uppercase tracking-[0.2em] text-zinc-400">
        {field.label}
      </span>
      <input
        type={field.type === "number" ? "number" : "text"}
        min={field.min_value}
        value={value}
        onChange={(event) => onChange(event.target.value)}
        className="w-full border-2 border-zinc-700 bg-black px-4 py-3 font-mono text-sm text-white outline-none transition duration-100 ease-linear focus:border-[var(--theme-accent-lime)] focus:ring-2 focus:ring-[var(--theme-accent-lime)] disabled:text-zinc-600 disabled:opacity-60"
        required={field.required}
        disabled={disabled}
      />
      <span className="mt-2 block text-xs text-zinc-600">{field.description}</span>
    </label>
  );
}

function groupConfigFields(schema: ConfigFieldDescriptor[]) {
  const groups = new Map<
    string,
    { key: string; label: string; fields: ConfigFieldDescriptor[] }
  >();
  for (const field of schema) {
    if (!groups.has(field.section_key)) {
      groups.set(field.section_key, {
        key: field.section_key,
        label: field.section_label,
        fields: [],
      });
    }
    groups.get(field.section_key)?.fields.push(field);
  }
  return Array.from(groups.values());
}

function buildFieldValues(
  config: DaemonConfig,
  schema: ConfigFieldDescriptor[],
  runtimeOnly: boolean,
): Record<string, string> {
  const source = config as Record<string, unknown>;
  return Object.fromEntries(
    schema
      .filter((field) => (runtimeOnly ? field.editable_in_runtime : true))
      .map((field) => [
        field.key,
        String(source[field.key] ?? field.default_value ?? ""),
      ]),
  );
}

function buildSettingsPayload(
  fields: ConfigFieldDescriptor[],
  values: Record<string, string>,
): Record<string, string | number> {
  return Object.fromEntries(
    fields.map((field) => [
      field.key,
      field.type === "number"
        ? Number(values[field.key] ?? field.default_value)
        : (values[field.key] ?? String(field.default_value)).trim(),
    ]),
  );
}
