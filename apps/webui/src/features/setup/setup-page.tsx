import { useEffect, useMemo, useRef, useState, type FormEvent } from "react";
import { useMutation, useQuery } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import api, { ApiClientError } from "../../lib/api";
import { configQueries } from "../../lib/queries";
import { sessionQueries } from "../../lib/queries/session";
import type {
  ConfigFieldDescriptor,
  SetupInitializeRequest,
} from "../../lib/types";

type Step = "config" | "signing" | "admin";
type SigningMode = "generate" | "import";

const STEP_LABELS: Record<Step, string> = {
  config: "Step 1 of 3 · Configuration",
  signing: "Step 2 of 3 · Signing",
  admin: "Step 3 of 3 · First account",
};

const STEP_DESCRIPTIONS: Record<Step, string> = {
  config: "Configure daemon settings for first run.",
  signing: "Choose whether to enable managed repository signing.",
  admin: "Create the first admin account.",
};

const inputClass =
  "w-full border-2 border-zinc-700 bg-black px-4 py-3 font-mono text-sm text-zinc-100 outline-none transition duration-100 ease-linear focus:border-[var(--theme-accent-lime)] focus:ring-2 focus:ring-[var(--theme-accent-lime)]";

const labelTitleClass =
  "mb-2 block font-mono text-xs font-bold uppercase tracking-[0.16em] text-zinc-300";

const primaryButtonClass =
  "border-2 border-[var(--theme-accent-lime)] bg-[var(--theme-accent-lime)] px-4 py-3 font-mono text-xs font-bold uppercase tracking-[0.16em] text-black transition duration-100 ease-linear hover:-translate-x-[1px] hover:-translate-y-[1px] hover:bg-[#d8ff72]";

const secondaryButtonClass =
  "border-2 border-zinc-700 bg-black px-4 py-3 font-mono text-xs font-bold uppercase tracking-[0.16em] text-zinc-200 transition duration-100 ease-linear hover:-translate-x-[1px] hover:-translate-y-[1px] hover:border-white hover:bg-zinc-950";

interface AdminForm {
  handle: string;
  displayName: string;
  password: string;
  passwordConfirm: string;
}

interface SigningState {
  enabled: boolean;
  mode: SigningMode;
  privateKey: string;
  filename: string;
}

const EMPTY_ADMIN: AdminForm = {
  handle: "admin",
  displayName: "Administrator",
  password: "",
  passwordConfirm: "",
};

const DEFAULT_SIGNING: SigningState = {
  enabled: true,
  mode: "generate",
  privateKey: "",
  filename: "",
};

function defaultFieldValue(field: ConfigFieldDescriptor): string {
  if (field.key === "public_base_url" && typeof window !== "undefined") {
    return window.location.origin;
  }
  return String(field.default_value ?? "");
}

function groupBySection(fields: ConfigFieldDescriptor[]) {
  const sections = new Map<string, { label: string; fields: ConfigFieldDescriptor[] }>();
  for (const field of fields) {
    if (!sections.has(field.section_key)) {
      sections.set(field.section_key, { label: field.section_label, fields: [] });
    }
    sections.get(field.section_key)!.fields.push(field);
  }
  return Array.from(sections.values());
}

function validateConfig(
  fields: ConfigFieldDescriptor[],
  values: Record<string, string>,
): string | null {
  for (const field of fields) {
    const raw = (values[field.key] ?? "").trim();
    if (field.required && raw.length === 0) {
      return `${field.label} is required.`;
    }
    if (field.type === "number" && raw.length > 0 && Number.isNaN(Number(raw))) {
      return `${field.label} must be a valid number.`;
    }
  }
  return null;
}

function buildSettings(
  fields: ConfigFieldDescriptor[],
  values: Record<string, string>,
): SetupInitializeRequest["settings"] {
  const settings: SetupInitializeRequest["settings"] = {};
  for (const field of fields) {
    const raw = values[field.key] ?? String(field.default_value ?? "");
    settings[field.key] = field.type === "number" ? Number(raw) : raw.trim();
  }
  return settings;
}

export default function SetupPage() {
  const navigate = useNavigate();

  const statusQuery = useQuery({
    ...sessionQueries.setupStatus(),
    retry: false,
  });
  const schemaQuery = useQuery(configQueries.schema());

  const editableFields = useMemo(
    () =>
      (schemaQuery.data?.fields ?? []).filter((field) => field.editable_in_setup),
    [schemaQuery.data],
  );

  const [step, setStep] = useState<Step>("config");
  const [error, setError] = useState<string | null>(null);
  const [configValues, setConfigValues] = useState<Record<string, string>>({});
  const [configValuesInitialized, setConfigValuesInitialized] = useState(false);
  const [signing, setSigning] = useState<SigningState>(DEFAULT_SIGNING);
  const [admin, setAdmin] = useState<AdminForm>(EMPTY_ADMIN);
  const fileInputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (!configValuesInitialized && editableFields.length > 0) {
      setConfigValues(
        Object.fromEntries(
          editableFields.map((field) => [field.key, defaultFieldValue(field)]),
        ),
      );
      setConfigValuesInitialized(true);
    }
  }, [configValuesInitialized, editableFields]);

  useEffect(() => {
    if (statusQuery.data?.initialized) {
      navigate({
        to: "/login",
        search: {
          message: "Synforge is already initialized. Sign in to continue.",
        },
      });
    }
  }, [statusQuery.data?.initialized, navigate]);

  const initializeMutation = useMutation({
    mutationFn: (payload: SetupInitializeRequest) => api.initializeSetup(payload),
    onSuccess: () => {
      navigate({
        to: "/login",
        search: {
          message:
            "Setup complete. Sign in with the admin account you just created.",
        },
      });
    },
    onError: (err) => {
      setError(err instanceof ApiClientError ? err.error.message : "Setup failed.");
    },
  });

  function goNext() {
    setError(null);
    if (step === "config") {
      const validationError = validateConfig(editableFields, configValues);
      if (validationError) {
        setError(validationError);
        return;
      }
      setStep("signing");
      return;
    }
    if (step === "signing") {
      if (signing.enabled && signing.mode === "import" && !signing.privateKey.trim()) {
        setError("Import mode selected but no key file has been loaded.");
        return;
      }
      setStep("admin");
      return;
    }
  }

  function goBack() {
    setError(null);
    setStep((current) => (current === "admin" ? "signing" : "config"));
  }

  function handleSubmit(event: FormEvent) {
    event.preventDefault();
    setError(null);
    if (!admin.handle.trim() || !admin.displayName.trim() || !admin.password) {
      setError("Admin handle, display name, and password are required.");
      return;
    }
    if (admin.password !== admin.passwordConfirm) {
      setError("Password confirmation does not match.");
      return;
    }
    initializeMutation.mutate({
      settings: buildSettings(editableFields, configValues),
      enable_signing: signing.enabled,
      signing_armored_private_key:
        signing.enabled && signing.mode === "import"
          ? signing.privateKey.trim()
          : null,
      admin: {
        handle: admin.handle.trim(),
        display_name: admin.displayName.trim(),
        password: admin.password,
      },
    });
  }

  async function handleFileChange(file: File) {
    setError(null);
    try {
      const text = await file.text();
      setSigning((prev) => ({
        ...prev,
        mode: "import",
        privateKey: text,
        filename: file.name,
      }));
    } catch {
      setSigning((prev) => ({ ...prev, privateKey: "", filename: "" }));
      setError("Failed to read signing key file.");
    }
  }

  if (statusQuery.isPending || schemaQuery.isPending) {
    return null;
  }

  if (statusQuery.error || schemaQuery.error) {
    return (
      <div className="flex min-h-full items-center justify-center px-3 py-12">
        <p className="border-2 border-[var(--theme-error-red)] bg-black px-4 py-3 font-mono text-sm text-zinc-200">
          Failed to load daemon configuration.
        </p>
      </div>
    );
  }

  return (
    <div className="flex min-h-full items-start justify-center px-3 py-3 sm:px-6 sm:py-6 lg:items-center lg:py-12">
      <div className="flex max-h-[calc(100dvh-1.5rem)] w-full max-w-2xl flex-col overflow-hidden border-4 border-white bg-black shadow-[6px_6px_0_rgba(255,255,255,0.25)] sm:max-h-[calc(100dvh-3rem)] lg:max-h-[calc(100dvh-6rem)] xl:max-w-3xl">
        <header className="mb-6 shrink-0 border-b-2 border-zinc-800 px-4 pb-5 pt-5 sm:mb-8 sm:px-8 sm:pb-6 sm:pt-8">
          <p className="font-mono text-xs font-bold uppercase tracking-[0.3em] text-[var(--theme-accent-lime)]">
            Synforge
          </p>
          <h1 className="mt-3 font-mono text-3xl font-bold uppercase text-zinc-50">
            First Run Setup
          </h1>
          <p className="mt-3 text-sm leading-6 text-zinc-400">
            {STEP_DESCRIPTIONS[step]}
          </p>
          <p className="mt-3 font-mono text-xs font-bold uppercase tracking-[0.22em] text-zinc-500">
            {STEP_LABELS[step]}
          </p>
        </header>

        <form
          onSubmit={handleSubmit}
          className="min-h-0 flex-1 overflow-y-auto px-4 pb-5 pt-0 sm:px-8 sm:pb-8"
        >
          {step === "config" ? (
            <ConfigStep
              sections={groupBySection(editableFields)}
              values={configValues}
              onChange={(key, value) =>
                setConfigValues((prev) => ({ ...prev, [key]: value }))
              }
            />
          ) : null}

          {step === "signing" ? (
            <SigningStep
              signing={signing}
              onToggle={() =>
                setSigning((prev) => ({ ...prev, enabled: !prev.enabled }))
              }
              onSelectGenerate={() =>
                setSigning((prev) => ({
                  ...prev,
                  mode: "generate",
                  privateKey: "",
                  filename: "",
                }))
              }
              onSelectImport={() => {
                setSigning((prev) => ({ ...prev, mode: "import" }));
                fileInputRef.current?.click();
              }}
              fileInputRef={fileInputRef}
              onFileChange={handleFileChange}
            />
          ) : null}

          {step === "admin" ? (
            <AdminStep admin={admin} onChange={setAdmin} />
          ) : null}

          {error ? (
            <p className="mt-4 border-2 border-[var(--theme-error-red)] bg-black px-3 py-2 text-sm text-zinc-200">
              {error}
            </p>
          ) : null}

          <div className="mt-4 flex items-center justify-between gap-3">
            {step !== "config" ? (
              <button type="button" onClick={goBack} className={secondaryButtonClass}>
                Back
              </button>
            ) : null}
            <div className="ml-auto flex items-center gap-3">
              {step !== "admin" ? (
                <button type="button" onClick={goNext} className={primaryButtonClass}>
                  Continue
                </button>
              ) : (
                <button
                  type="submit"
                  disabled={initializeMutation.isPending}
                  className={`${primaryButtonClass} disabled:opacity-60`}
                >
                  {initializeMutation.isPending
                    ? "Initializing…"
                    : "Initialize Synforge"}
                </button>
              )}
            </div>
          </div>
        </form>
      </div>
    </div>
  );
}

function ConfigStep({
  sections,
  values,
  onChange,
}: {
  sections: { label: string; fields: ConfigFieldDescriptor[] }[];
  values: Record<string, string>;
  onChange: (key: string, value: string) => void;
}) {
  return (
    <div className="space-y-4">
      <div className="grid gap-4 xl:grid-cols-2">
        {sections.map((section) => (
          <section
            key={section.label}
            className="xl:col-span-2 border-2 border-zinc-700 bg-black p-5"
          >
            <h2 className="font-mono text-lg font-bold uppercase text-white">
              {section.label}
            </h2>
            <div className="mt-4 grid gap-4 xl:grid-cols-2">
              {section.fields.map((field) => (
                <label key={field.key} className="block">
                  <span className={labelTitleClass}>{field.label}</span>
                  <input
                    type={field.type === "number" ? "number" : "text"}
                    required={field.required}
                    min={field.min_value !== undefined ? String(field.min_value) : undefined}
                    value={values[field.key] ?? ""}
                    onChange={(e) => onChange(field.key, e.target.value)}
                    className={inputClass}
                  />
                  <span className="mt-2 block text-xs text-zinc-500">
                    {field.description}
                  </span>
                </label>
              ))}
            </div>
          </section>
        ))}
      </div>
    </div>
  );
}

function SigningStep({
  signing,
  onToggle,
  onSelectGenerate,
  onSelectImport,
  fileInputRef,
  onFileChange,
}: {
  signing: SigningState;
  onToggle: () => void;
  onSelectGenerate: () => void;
  onSelectImport: () => void;
  fileInputRef: React.RefObject<HTMLInputElement | null>;
  onFileChange: (file: File) => void;
}) {
  const buttonBase =
    "border-2 px-4 py-3 font-mono text-xs font-bold uppercase tracking-[0.16em] transition duration-100 ease-linear hover:-translate-x-[1px] hover:-translate-y-[1px]";
  const activeClass =
    "border-[var(--theme-accent-lime)] bg-[var(--theme-accent-lime)] text-black hover:bg-[#d8ff72]";
  const idleClass =
    "border-zinc-700 bg-black text-zinc-200 hover:border-white hover:bg-zinc-950";

  const keyNote = !signing.enabled
    ? "Key actions are disabled while signing is disabled."
    : signing.mode === "import"
      ? signing.filename
        ? `Import selected: ${signing.filename}`
        : "Import mode selected. Choose a private key file."
      : "Managed key generation is selected.";

  return (
    <div className="space-y-4">
      <section className="border-2 border-zinc-700 bg-black p-5">
        <h2 className="font-mono text-lg font-bold uppercase text-white">Signing</h2>
        <p className="mt-3 text-sm leading-6 text-zinc-400">
          Configure repository signing before initialization. You can keep signing
          disabled, generate a managed key, or import an existing key file.
        </p>

        <div className="mt-5 space-y-5">
          <div>
            <p className="mb-2 font-mono text-xs font-bold uppercase tracking-[0.16em] text-zinc-400">
              Signing State
            </p>
            <button
              type="button"
              onClick={onToggle}
              className={`${buttonBase} ${signing.enabled ? activeClass : idleClass}`}
            >
              {signing.enabled ? "Disable Signing" : "Enable Signing"}
            </button>
            <p className="mt-2 font-mono text-xs text-zinc-500">
              {signing.enabled
                ? "Signing will be enabled after initialization."
                : "Signing will stay disabled after initialization."}
            </p>
          </div>

          <div>
            <p className="mb-2 font-mono text-xs font-bold uppercase tracking-[0.16em] text-zinc-400">
              Key Lifecycle
            </p>
            <div className="flex flex-wrap gap-3">
              <button
                type="button"
                disabled={!signing.enabled}
                onClick={onSelectGenerate}
                className={`${buttonBase} ${
                  signing.enabled && signing.mode === "generate" ? activeClass : idleClass
                }`}
              >
                Generate Key
              </button>
              <button
                type="button"
                disabled={!signing.enabled}
                onClick={onSelectImport}
                className={`${buttonBase} ${
                  signing.enabled && signing.mode === "import" ? activeClass : idleClass
                }`}
              >
                Import Key File
              </button>
            </div>
            <input
              ref={fileInputRef}
              type="file"
              accept=".asc,.key,.pgp,.gpg,text/plain"
              className="hidden"
              onChange={(e) => {
                const file = e.target.files?.[0];
                if (file) onFileChange(file);
              }}
            />
            <p className="mt-2 font-mono text-xs text-zinc-500">{keyNote}</p>
          </div>
        </div>
      </section>
    </div>
  );
}

function AdminStep({
  admin,
  onChange,
}: {
  admin: AdminForm;
  onChange: (next: AdminForm) => void;
}) {
  const update = (patch: Partial<AdminForm>) => onChange({ ...admin, ...patch });
  return (
    <div className="space-y-4">
      <div className="grid gap-4 xl:grid-cols-2">
        <label className="block">
          <span className={labelTitleClass}>Admin handle</span>
          <input
            type="text"
            required
            value={admin.handle}
            onChange={(e) => update({ handle: e.target.value })}
            className={inputClass}
          />
        </label>
        <label className="block">
          <span className={labelTitleClass}>Admin display name</span>
          <input
            type="text"
            required
            value={admin.displayName}
            onChange={(e) => update({ displayName: e.target.value })}
            className={inputClass}
          />
        </label>
        <label className="block xl:col-span-2">
          <span className={labelTitleClass}>Admin password</span>
          <input
            type="password"
            required
            placeholder="Choose a strong password"
            value={admin.password}
            onChange={(e) => update({ password: e.target.value })}
            className={inputClass}
          />
        </label>
        <label className="block xl:col-span-2">
          <span className={labelTitleClass}>Confirm password</span>
          <input
            type="password"
            required
            placeholder="Re-enter admin password"
            value={admin.passwordConfirm}
            onChange={(e) => update({ passwordConfirm: e.target.value })}
            className={inputClass}
          />
        </label>
      </div>
    </div>
  );
}
