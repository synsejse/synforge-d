import { useEffect, useState, type SyntheticEvent } from "react";
import { useMutation, useQuery } from "@tanstack/react-query";
import {
  faRotateLeft,
  faSave,
  faServer,
} from "@fortawesome/free-solid-svg-icons";
import api from "../../lib/api";
import { configQueries } from "../../lib/queries";
import type { ConfigFieldDescriptor, DaemonConfig } from "../../lib/types";
import ErrorMessage from "../../components/common/error-message";
import LoadingBlock from "../../components/ui/loading-block";
import FaIcon from "../../components/ui/fa-icon";
import Button from "../../components/ui/button";
import PageHeader from "../../components/ui/page-header";
import Tooltip from "../../components/ui/tooltip";
import { DisclosureGroup, Disclosure } from "../../components/ui/disclosure";

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
        title="Configuration"
        description="Runtime settings and effective daemon values."
        color="purple"
        actions={[{ to: "/", label: "Overview", icon: faServer }]}
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
        <DisclosureGroup
          defaultValue={groupedFields
            .filter((s) => s.fields.length > 0)
            .slice(0, 1)
            .map((s) => s.key)}
        >
          {groupedFields.map((section) => {
            const runtimeFields = section.fields.filter(
              (field) => field.editable_in_runtime,
            );
            const readOnlyFields = section.fields.filter(
              (field) => !field.editable_in_runtime,
            );

            if (runtimeFields.length === 0 && readOnlyFields.length === 0) {
              return null;
            }

            const editableCount = runtimeFields.length;
            const readOnlyCount = readOnlyFields.length;
            const trailingCount =
              [
                editableCount > 0 ? `${editableCount} editable` : null,
                readOnlyCount > 0 ? `${readOnlyCount} read-only` : null,
              ]
                .filter(Boolean)
                .join(" · ");

            return (
              <Disclosure
                key={section.key}
                value={section.key}
                title={section.label}
                description={trailingCount || undefined}
              >
                <div className="grid gap-6 xl:grid-cols-2">
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
              </Disclosure>
            );
          })}
        </DisclosureGroup>

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
  const defaultStr =
    field.default_value != null ? String(field.default_value) : "";
  const isAtDefault = value.trim() === defaultStr.trim();
  const canReset = !disabled && defaultStr !== "" && !isAtDefault;

  return (
    <label className="block">
      <div className="mb-2 flex items-center justify-between gap-2">
        <span className="font-mono text-xs font-bold uppercase tracking-[0.2em] text-muted">
          {field.label}
        </span>
        {canReset ? (
          <Tooltip content={`Reset to default: ${defaultStr}`} side="top">
            <button
              type="button"
              onClick={() => onChange(defaultStr)}
              aria-label={`Reset ${field.label} to default`}
              className="inline-flex items-center gap-1 border-2 border-edge-strong bg-black px-2 py-0.5 font-mono text-[10px] uppercase tracking-[0.15em] text-soft transition hover:border-white hover:text-white focus:outline-none focus:ring-2 focus:ring-accent-lime"
            >
              <FaIcon icon={faRotateLeft} className="text-[0.85em]" />
              Reset
            </button>
          </Tooltip>
        ) : null}
      </div>
      <input
        type={field.type === "number" ? "number" : "text"}
        min={field.min_value ?? undefined}
        value={value}
        onChange={(event) => onChange(event.target.value)}
        className="w-full border-2 border-edge-strong bg-black px-4 py-3 font-mono text-sm text-white outline-none transition duration-100 ease-linear focus:border-accent-lime focus:ring-2 focus:ring-accent-lime disabled:text-soft disabled:opacity-60"
        required={field.required}
        disabled={disabled}
      />
      <div className="mt-2 flex flex-wrap items-baseline justify-between gap-x-3 gap-y-1">
        <span className="text-xs text-soft">{field.description}</span>
        {defaultStr !== "" ? (
          <span className="font-mono text-[10px] uppercase tracking-[0.15em] text-soft">
            Default: <span className="text-muted">{defaultStr}</span>
          </span>
        ) : null}
      </div>
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
