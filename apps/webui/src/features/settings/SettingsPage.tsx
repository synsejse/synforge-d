import { useEffect, useState, type SyntheticEvent } from "react";
import { useMutation, useQuery } from "@tanstack/react-query";
import { faSave, faServer } from "@fortawesome/free-solid-svg-icons";
import api from "../../lib/api";
import { configQueries } from "../../lib/queries";
import type { ConfigFieldDescriptor, DaemonConfig } from "../../lib/types";
import ErrorMessage from "../../components/common/ErrorMessage";
import LoadingBlock from "../../components/ui/LoadingBlock";
import FaIcon from "../../components/ui/FaIcon";
import Button from "../../components/ui/Button";
import PageHeader from "../../components/ui/PageHeader";

function Settings() {
  const configQuery = useQuery(configQueries.effective());
  const schemaQuery = useQuery(configQueries.schema());

  const [values, setValues] = useState<Record<string, string>>({});
  const [valuesInitialized, setValuesInitialized] = useState(false);

  useEffect(() => {
    if (
      !valuesInitialized &&
      configQuery.data &&
      schemaQuery.data
    ) {
      setValues(
        buildFieldValues(configQuery.data.config, schemaQuery.data.fields, false),
      );
      setValuesInitialized(true);
    }
  }, [configQuery.data, schemaQuery.data, valuesInitialized]);

  const saveMutation = useMutation({
    mutationFn: () => {
      const runtimeFields = (schemaQuery.data?.fields ?? []).filter(
        (field) => field.editable_in_runtime,
      );
      return api.updateRuntimeSettings({
        settings: buildSettingsPayload(runtimeFields, values),
      });
    },
    onSuccess: (response) => {
      if (schemaQuery.data) {
        setValues(buildFieldValues(response.config, schemaQuery.data.fields, false));
      }
      configQuery.refetch();
    },
  });

  function handleSave(event: SyntheticEvent) {
    event.preventDefault();
    saveMutation.mutate();
  }

  if (configQuery.isPending || schemaQuery.isPending) {
    return <LoadingBlock label="Loading config…" lines={4} />;
  }

  const loadError = configQuery.error ?? schemaQuery.error;
  if (loadError || !configQuery.data || !schemaQuery.data) {
    return (
      <ErrorMessage
        message={loadError instanceof Error ? loadError.message : "Failed to load"}
      />
    );
  }

  const groupedFields = groupConfigFields(schemaQuery.data.fields);

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

      {saveMutation.error ? (
        <ErrorMessage
          message={
            saveMutation.error instanceof Error
              ? saveMutation.error.message
              : "Failed to update settings"
          }
        />
      ) : null}

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
          <Button type="submit" variant="primary" size="md" disabled={saveMutation.isPending}>
            <FaIcon icon={faSave} className="mr-2" />
            {saveMutation.isPending ? "Saving…" : "Save Settings"}
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
        min={field.min_value ?? undefined}
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

export default function SettingsPage() {
  return <Settings />;
}
