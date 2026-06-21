import type { ConfigSection } from "../model";
import { inputClass, labelTitleClass } from "../model";

interface ConfigStepProps {
  sections: ConfigSection[];
  values: Record<string, string>;
  onChange: (key: string, value: string) => void;
}

export default function ConfigStep({
  sections,
  values,
  onChange,
}: ConfigStepProps) {
  return (
    <div className="space-y-4">
      <div className="grid gap-4 xl:grid-cols-2">
        {sections.map((section) => (
          <section
            key={section.label}
            className="xl:col-span-2 border border-edge bg-black p-5"
          >
            <h2 className="font-mono text-[13px] font-bold uppercase tracking-[0.06em] text-white">{section.label}</h2>
            <div className="mt-4 grid gap-4 xl:grid-cols-2">
              {section.fields.map((field) => (
                <label key={field.key} className="block">
                  <span className={labelTitleClass}>{field.label}</span>
                  <input
                    type={field.type === "number" ? "number" : "text"}
                    required={field.required}
                    min={
                      field.min_value !== undefined
                        ? String(field.min_value)
                        : undefined
                    }
                    value={values[field.key] ?? ""}
                    onChange={(e) => onChange(field.key, e.target.value)}
                    className={inputClass}
                  />
                  <span className="mt-2 block text-xs text-soft">
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
