import type { AdminForm } from "../model";
import { inputClass, labelTitleClass } from "../model";

interface AdminStepProps {
  admin: AdminForm;
  onChange: (next: AdminForm) => void;
}

export default function AdminStep({ admin, onChange }: AdminStepProps) {
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
