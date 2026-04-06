import type { UserPermission } from "../../lib/types";
import { PERMISSIONS } from "./model";

interface TextFieldProps {
  label: string;
  value: string;
  onChange: (value: string) => void;
  type?: string;
}

export function TextField({
  label,
  value,
  onChange,
  type = "text",
}: TextFieldProps) {
  return (
    <label className="block">
      <span className="mb-2 block font-mono text-xs font-bold uppercase tracking-[0.18em] text-zinc-400">
        {label}
      </span>
      <input
        type={type}
        value={value}
        onChange={(event) => onChange(event.target.value)}
        className="w-full border-2 border-zinc-700 bg-black px-4 py-3 font-mono text-sm text-white outline-none transition duration-100 ease-linear focus:border-[var(--theme-accent-lime)] focus:ring-2 focus:ring-[var(--theme-accent-lime)]"
      />
    </label>
  );
}

interface ToggleFieldProps {
  label: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
}

export function ToggleField({ label, checked, onChange }: ToggleFieldProps) {
  return (
    <label className="flex items-center gap-3 border-2 border-zinc-700 bg-black px-4 py-3 font-mono text-sm text-zinc-200">
      <input
        type="checkbox"
        checked={checked}
        onChange={(event) => onChange(event.target.checked)}
        className="h-4 w-4 border-2 border-zinc-500 bg-black accent-[var(--theme-accent-lime)]"
      />
      {label}
    </label>
  );
}

interface PermissionGroupProps {
  permissions: UserPermission[];
  onToggle: (permission: UserPermission) => void;
}

export function PermissionGroup({ permissions, onToggle }: PermissionGroupProps) {
  return (
    <div>
      <div className="mb-2 font-mono text-xs font-bold uppercase tracking-[0.18em] text-zinc-400">
        Permissions
      </div>
      <div
        className="grid gap-2 sm:grid-cols-3"
        role="group"
        aria-label="User permissions"
      >
        {PERMISSIONS.map((permission) => {
          const enabled = permissions.includes(permission);
          return (
            <button
              key={permission}
                type="button"
                onClick={() => onToggle(permission)}
                aria-pressed={enabled}
                className={`border-2 px-4 py-3 font-mono text-xs font-bold uppercase tracking-[0.12em] transition duration-100 ease-linear ${
                  enabled
                    ? "border-[var(--theme-accent-lime)] bg-zinc-950 text-[var(--theme-accent-lime)]"
                    : "border-zinc-700 bg-black text-zinc-300 hover:border-white hover:bg-zinc-950"
                }`}
              >
              {permission}
            </button>
          );
        })}
      </div>
    </div>
  );
}
