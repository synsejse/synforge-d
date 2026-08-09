import { useEffect, useId, useState, type SyntheticEvent } from "react";
import { useMutation, useQuery } from "@tanstack/react-query";
import {
  faRotateLeft,
  faSave,
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
import useUnsavedChangesWarning from "../../components/common/use-unsaved-changes-warning";

function Settings() {
  const configQuery = useQuery(configQueries.effective());
  const schemaQuery = useQuery(configQueries.schema());

  const [values, setValues] = useState<Record<string, string>>({});
  const [pristineValues, setPristineValues] = useState<Record<string, string>>(
    {},
  );
  const [valuesInitialized, setValuesInitialized] = useState(false);

  useEffect(() => {
    if (
      !valuesInitialized &&
      configQuery.data &&
      schemaQuery.data
    ) {
      const initialValues = buildFieldValues(
        configQuery.data.config,
        schemaQuery.data.fields,
        configQuery.data.pending_restart_settings,
      );
      setValues(initialValues);
      setPristineValues(initialValues);
      setValuesInitialized(true);
    }
  }, [configQuery.data, schemaQuery.data, valuesInitialized]);

  const runtimeFields = (schemaQuery.data?.fields ?? []).filter(
    (field) => field.editable_in_runtime,
  );
  const isDirty = runtimeFields.some(
    (field) => values[field.key] !== pristineValues[field.key],
  );

  useUnsavedChangesWarning(isDirty);

  const saveMutation = useMutation({
    mutationFn: () => {
      return api.updateRuntimeSettings({
        settings: buildSettingsPayload(runtimeFields, values),
      });
    },
    onSuccess: (response) => {
      if (schemaQuery.data) {
        const savedValues = buildFieldValues(
          response.config,
          schemaQuery.data.fields,
          response.pending_restart_settings,
        );
        setValues(savedValues);
        setPristineValues(savedValues);
      }
      void configQuery.refetch();
    },
  });

  function handleSave(event: SyntheticEvent) {
    event.preventDefault();
    if (!isDirty || saveMutation.isPending) return;
    saveMutation.mutate();
  }

  function discardChanges() {
    setValues(pristineValues);
    saveMutation.reset();
  }

  if (configQuery.isPending || schemaQuery.isPending) {
    return (
      <div className="space-y-6">
        <PageHeader
          title="Settings"
          description="Runtime settings and effective daemon values."
          color="purple"
        />
        <LoadingBlock label="Loading config…" lines={4} />
      </div>
    );
  }

  const loadError = configQuery.error ?? schemaQuery.error;
  if (loadError || !configQuery.data || !schemaQuery.data) {
    return (
      <ErrorMessage
        message={loadError instanceof Error ? loadError.message : "Failed to load"}
        onRetry={() => {
          void configQuery.refetch();
          void schemaQuery.refetch();
        }}
        retrying={configQuery.isFetching || schemaQuery.isFetching}
      />
    );
  }

  const groupedFields = groupConfigFields(schemaQuery.data.fields);

  return (
    <div className="space-y-6">
      <PageHeader
        title="Settings"
        description="Runtime settings and effective daemon values."
        color="purple"
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

      {configQuery.data.restart_required ? (
        <div className="border border-accent-cyan bg-surface p-5 sm:p-6">
          <p className="font-mono text-xs font-bold uppercase tracking-[0.2em] text-accent-cyan">
            Daemon restart required
          </p>
          <p className="mt-2 text-sm text-muted">
            Saved values marked below are pending. The effective values remain unchanged until
            the daemon restarts.
          </p>
        </div>
      ) : null}

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
                <div className="grid gap-x-5 gap-y-6 md:grid-cols-2">
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

        <div className="sticky bottom-0 z-20 flex flex-col gap-3 border border-edge-strong bg-black/95 p-4 backdrop-blur-sm sm:flex-row sm:items-center sm:justify-between">
          <div className="flex items-center gap-2 font-mono text-xs font-bold uppercase tracking-[0.14em]">
            <span
              aria-hidden="true"
              className={`h-2 w-2 ${isDirty ? "animate-pulse bg-accent-violet" : "bg-success"}`}
            />
            <span className={isDirty ? "text-accent-violet" : "text-soft"}>
              {isDirty ? "Unsaved changes" : "All changes saved"}
            </span>
          </div>
          <div className="flex flex-col gap-2 sm:flex-row">
            {isDirty ? (
              <Button
                variant="ghost"
                size="md"
                onClick={discardChanges}
                disabled={saveMutation.isPending}
              >
                Discard
              </Button>
            ) : null}
            <Button
              type="submit"
              variant="primary"
              size="md"
              disabled={!isDirty}
              loading={saveMutation.isPending}
            >
              {saveMutation.isPending ? null : <FaIcon icon={faSave} />}
              {saveMutation.isPending ? "Saving…" : "Save settings"}
            </Button>
          </div>
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
  const inputId = useId();
  const defaultStr =
    field.default_value != null ? String(field.default_value) : "";
  const isAtDefault = value.trim() === defaultStr.trim();
  const canReset = !disabled && defaultStr !== "" && !isAtDefault;

  return (
    <div className="block">
      {/* min-h matches the Reset button height (border-2 + py-0.5 +
          10px text) so rows with and without the button line up at the
          same input baseline when sitting side-by-side in a grid. */}
      <div className="mb-2 flex min-h-[1.5rem] items-center justify-between gap-2">
        <label
          htmlFor={inputId}
          className="font-mono text-xs font-bold uppercase tracking-[0.2em] text-muted"
        >
          {field.label}
        </label>
        <span className="flex items-center gap-2">
          {field.restart_required ? (
            <span className="font-mono text-xs uppercase tracking-[0.15em] text-accent-cyan">
              Restart required
            </span>
          ) : null}
          {canReset ? (
            <Tooltip content={`Reset to default: ${defaultStr}`} side="top">
              <Button
                variant="subtle"
                size="xs"
                onClick={() => onChange(defaultStr)}
                aria-label={`Reset ${field.label} to default`}
                className="border-edge text-soft"
              >
                <FaIcon icon={faRotateLeft} className="text-[0.85em]" />
                Reset
              </Button>
            </Tooltip>
          ) : null}
        </span>
      </div>
      <input
        id={inputId}
        type={field.type === "number" ? "number" : "text"}
        min={field.min_value ?? undefined}
        value={value}
        onChange={(event) => onChange(event.target.value)}
        className="w-full border border-edge bg-black px-4 py-3 font-mono text-sm text-white outline-none transition duration-100 ease-linear focus:border-accent-lime focus:ring-2 focus:ring-accent-lime disabled:text-soft disabled:opacity-60"
        required={field.required}
        disabled={disabled}
      />
      <div className="mt-2 flex flex-wrap items-baseline justify-between gap-x-3 gap-y-1">
        <span className="text-xs text-soft">{field.description}</span>
        {defaultStr !== "" ? (
          <span className="font-mono text-xs uppercase tracking-[0.15em] text-soft">
            Default: <span className="text-muted">{defaultStr}</span>
          </span>
        ) : null}
      </div>
    </div>
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
  pendingRestartSettings: Record<string, unknown> = {},
): Record<string, string> {
  const source = config as Record<string, unknown>;
  return Object.fromEntries(
    schema
      .map((field) => [
        field.key,
        String(
          pendingRestartSettings[field.key] ??
            source[field.key] ??
            field.default_value ??
            "",
        ),
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
