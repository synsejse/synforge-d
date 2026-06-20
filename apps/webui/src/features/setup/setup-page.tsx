import { useEffect, useMemo, useRef, useState, type FormEvent } from "react";
import { useMutation, useQuery } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import api, { ApiClientError } from "../../lib/api";
import { configQueries } from "../../lib/queries";
import { sessionQueries } from "../../lib/queries/session";
import type { SetupInitializeRequest } from "../../lib/types";
import AdminStep from "./components/admin-step";
import ConfigStep from "./components/config-step";
import SetupNav from "./components/setup-nav";
import SigningStep from "./components/signing-step";
import {
  DEFAULT_SIGNING,
  EMPTY_ADMIN,
  STEP_DESCRIPTIONS,
  STEP_LABELS,
  buildSettings,
  defaultFieldValue,
  groupBySection,
  validateConfig,
  type AdminForm,
  type SigningState,
  type Step,
} from "./model";

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
        <p className="border-2 border-error bg-black px-4 py-3 font-mono text-sm text-strong">
          Failed to load daemon configuration.
        </p>
      </div>
    );
  }

  return (
    <div className="flex min-h-full items-start justify-center px-3 py-3 sm:px-6 sm:py-6 lg:items-center lg:py-12">
      <div className="flex max-h-[calc(100dvh-1.5rem)] w-full max-w-2xl flex-col overflow-hidden border border-edge-strong bg-black shadow-card-md sm:max-h-[calc(100dvh-3rem)] lg:max-h-[calc(100dvh-6rem)] xl:max-w-3xl">
        <header className="mb-6 shrink-0 border-b border-edge px-4 pb-5 pt-5 sm:mb-8 sm:px-8 sm:pb-6 sm:pt-8">
          <p className="font-mono text-xs font-bold uppercase tracking-[0.3em] text-accent-lime">
            Synforge
          </p>
          <h1 className="mt-3 font-mono text-3xl font-bold uppercase text-strong">
            First Run Setup
          </h1>
          <p className="mt-3 text-sm leading-6 text-muted">
            {STEP_DESCRIPTIONS[step]}
          </p>
          <p className="mt-3 font-mono text-xs font-bold uppercase tracking-[0.22em] text-soft">
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
            <p className="mt-4 border-2 border-error bg-black px-3 py-2 text-sm text-strong">
              {error}
            </p>
          ) : null}

          <SetupNav
            step={step}
            submitting={initializeMutation.isPending}
            onBack={goBack}
            onNext={goNext}
          />
        </form>
      </div>
    </div>
  );
}
