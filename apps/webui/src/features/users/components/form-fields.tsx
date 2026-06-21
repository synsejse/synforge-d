import type { UserPermission } from "../../../lib/types";
import { PERMISSIONS } from "./model";

// Re-export shared form components
export { TextField, ToggleField } from "../../../components/ui/form-fields";

interface PermissionGroupProps {
  permissions: UserPermission[];
  onToggle: (permission: UserPermission) => void;
}

export function PermissionGroup({ permissions, onToggle }: PermissionGroupProps) {
  return (
    <div>
      <div className="mb-2 font-mono text-xs font-bold uppercase tracking-[0.15em] text-muted">
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
              className={`border px-4 py-3 font-mono text-xs font-bold uppercase tracking-[0.12em] transition duration-100 ease-linear ${
                enabled
                  ? "border-accent-lime bg-surface-alt text-accent-lime"
                  : "border-edge-strong bg-black text-muted hover:border-white hover:bg-surface-alt"
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
