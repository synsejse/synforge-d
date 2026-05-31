import type {
  ConfigFieldDescriptor,
  SetupInitializeRequest,
} from "../../lib/types";

export type Step = "config" | "signing" | "admin";
export type SigningMode = "generate" | "import";

export const STEP_LABELS: Record<Step, string> = {
  config: "Step 1 of 3 · Configuration",
  signing: "Step 2 of 3 · Signing",
  admin: "Step 3 of 3 · First account",
};

export const STEP_DESCRIPTIONS: Record<Step, string> = {
  config: "Configure daemon settings for first run.",
  signing: "Choose whether to enable managed repository signing.",
  admin: "Create the first admin account.",
};

export const inputClass =
  "w-full border-2 border-edge-strong bg-black px-4 py-3 font-mono text-sm text-strong outline-none transition duration-100 ease-linear focus:border-accent-lime focus:ring-2 focus:ring-accent-lime";

export const labelTitleClass =
  "mb-2 block font-mono text-xs font-bold uppercase tracking-[0.16em] text-muted";

export interface AdminForm {
  handle: string;
  displayName: string;
  password: string;
  passwordConfirm: string;
}

export interface SigningState {
  enabled: boolean;
  mode: SigningMode;
  privateKey: string;
  filename: string;
}

export const EMPTY_ADMIN: AdminForm = {
  handle: "admin",
  displayName: "Administrator",
  password: "",
  passwordConfirm: "",
};

export const DEFAULT_SIGNING: SigningState = {
  enabled: true,
  mode: "generate",
  privateKey: "",
  filename: "",
};

export interface ConfigSection {
  label: string;
  fields: ConfigFieldDescriptor[];
}

export function defaultFieldValue(field: ConfigFieldDescriptor): string {
  if (field.key === "public_base_url" && typeof window !== "undefined") {
    return window.location.origin;
  }
  return String(field.default_value ?? "");
}

export function groupBySection(fields: ConfigFieldDescriptor[]): ConfigSection[] {
  const sections = new Map<string, ConfigSection>();
  for (const field of fields) {
    if (!sections.has(field.section_key)) {
      sections.set(field.section_key, { label: field.section_label, fields: [] });
    }
    sections.get(field.section_key)!.fields.push(field);
  }
  return Array.from(sections.values());
}

export function validateConfig(
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

export function buildSettings(
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
