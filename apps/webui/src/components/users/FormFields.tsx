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
      <span className="mb-2 block text-sm font-medium text-zinc-300">
        {label}
      </span>
      <input
        type={type}
        value={value}
        onChange={(event) => onChange(event.target.value)}
        className="w-full border border-zinc-800 bg-black px-4 py-3 text-sm text-white outline-none transition focus:border-zinc-600"
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
    <label className="flex items-center gap-3 border border-zinc-800 bg-black px-4 py-3 text-sm text-zinc-200">
      <input
        type="checkbox"
        checked={checked}
        onChange={(event) => onChange(event.target.checked)}
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
      <div className="mb-2 text-sm font-medium text-zinc-300">Permissions</div>
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
              className={`border px-4 py-3 text-sm capitalize transition ${
                enabled
                  ? "border-zinc-600 bg-zinc-950 text-white"
                  : "border-zinc-800 bg-black text-zinc-300 hover:border-zinc-600 hover:bg-zinc-950"
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
