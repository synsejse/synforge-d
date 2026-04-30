import type { ReactNode } from "react";

// Shared input class for consistent styling
const inputClass =
  "w-full border-2 border-zinc-700 bg-zinc-950 px-4 py-3 font-mono text-sm text-white placeholder:text-zinc-600 outline-none transition duration-100 ease-linear focus:border-[var(--theme-accent-lime)]";

interface TextFieldProps {
  label: string;
  value: string;
  onChange: (value: string) => void;
  type?: string;
  placeholder?: string;
  required?: boolean;
  className?: string;
}

export function TextField({
  label,
  value,
  onChange,
  type = "text",
  placeholder,
  required,
  className = "",
}: TextFieldProps) {
  return (
    <label className={`block ${className}`}>
      <span className="mb-2 block font-mono text-xs font-bold uppercase tracking-[0.15em] text-zinc-400">
        {label}
      </span>
      <input
        type={type}
        value={value}
        onChange={(event) => onChange(event.target.value)}
        placeholder={placeholder}
        required={required}
        className={inputClass}
      />
    </label>
  );
}

interface NumberFieldProps {
  label: string;
  value: string;
  onChange: (value: string) => void;
  min?: number;
  step?: number;
  required?: boolean;
  className?: string;
}

export function NumberField({
  label,
  value,
  onChange,
  min = 1,
  step = 1,
  required,
  className = "",
}: NumberFieldProps) {
  return (
    <label className={`block ${className}`}>
      <span className="mb-2 block font-mono text-xs font-bold uppercase tracking-[0.15em] text-zinc-400">
        {label}
      </span>
      <input
        type="number"
        min={min}
        step={step}
        value={value}
        onChange={(event) => onChange(event.target.value)}
        required={required}
        className={inputClass}
      />
    </label>
  );
}

interface TextAreaFieldProps {
  label: string;
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  rows?: number;
  hint?: string;
  className?: string;
}

export function TextAreaField({
  label,
  value,
  onChange,
  placeholder,
  rows = 6,
  hint,
  className = "",
}: TextAreaFieldProps) {
  return (
    <label className={`block ${className}`}>
      <span className="mb-2 block font-mono text-xs font-bold uppercase tracking-[0.15em] text-zinc-400">
        {label}
      </span>
      <textarea
        value={value}
        onChange={(event) => onChange(event.target.value)}
        rows={rows}
        placeholder={placeholder}
        className={inputClass}
      />
      {hint && (
        <span className="mt-2 block text-xs text-zinc-500">{hint}</span>
      )}
    </label>
  );
}

interface ToggleFieldProps {
  label: string;
  description?: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
  className?: string;
}

export function ToggleField({
  label,
  description,
  checked,
  onChange,
  className = "",
}: ToggleFieldProps) {
  return (
    <label
      className={`flex items-start justify-between gap-3 border-2 border-zinc-700 bg-zinc-950 px-4 py-3 ${className}`}
    >
      <span className="min-w-0">
        <span className="block font-mono text-xs font-bold uppercase tracking-[0.1em] text-white">
          {label}
        </span>
        {description && (
          <span className="mt-1 block text-xs text-zinc-500">{description}</span>
        )}
      </span>
      <input
        type="checkbox"
        checked={checked}
        onChange={(event) => onChange(event.target.checked)}
      />
    </label>
  );
}

interface FieldGroupProps {
  label: string;
  description?: string;
  action?: ReactNode;
  children: ReactNode;
  className?: string;
}

export function FieldGroup({
  label,
  description,
  action,
  children,
  className = "",
}: FieldGroupProps) {
  return (
    <div className={`border-2 border-zinc-700 bg-zinc-950 p-4 ${className}`}>
      <div className="flex flex-col gap-3 md:flex-row md:flex-wrap md:items-center md:justify-between">
        <div className="min-w-0">
          <span className="block font-mono text-xs font-bold uppercase tracking-[0.15em] text-zinc-400">
            {label}
          </span>
          {description && (
            <span className="mt-1 block text-xs text-zinc-500">{description}</span>
          )}
        </div>
        {action}
      </div>
      <div className="mt-4">{children}</div>
    </div>
  );
}

interface DisplayBoxProps {
  children: ReactNode;
}

export function DisplayBox({ children }: DisplayBoxProps) {
  return (
    <div className="border-2 border-zinc-700 bg-black px-4 py-3 font-mono text-sm text-zinc-200 break-all">
      {children}
    </div>
  );
}
