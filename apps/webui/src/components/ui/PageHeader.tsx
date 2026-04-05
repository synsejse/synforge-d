import type { IconDefinition } from "@fortawesome/fontawesome-svg-core";
import ActionButton from "./ActionButton";

interface ActionLink {
  href: string;
  label: string;
  icon?: IconDefinition;
  variant?: "default" | "primary";
}

interface ActionButton {
  onClick: () => void;
  label: string;
  icon?: IconDefinition;
  variant?: "default" | "primary";
}

interface Props {
  eyebrow: string;
  title: string;
  description: string;
  actions?: Array<ActionLink | ActionButton>;
}

export default function PageHeader({ eyebrow, title, description, actions = [] }: Props) {
  return (
    <section className="border border-zinc-800 bg-black p-6">
      <div className="flex flex-col gap-4 lg:flex-row lg:items-end lg:justify-between">
        <div>
          <p className="text-xs uppercase tracking-[0.28em] text-zinc-500">{eyebrow}</p>
          <h1 className="mt-2 text-4xl font-semibold tracking-tight text-white">{title}</h1>
          <p className="mt-3 max-w-3xl text-sm leading-6 text-zinc-400">{description}</p>
        </div>
        {actions.length > 0 ? (
          <div className="flex flex-wrap gap-2">
            {actions.map((action) => {
              if ("href" in action) {
                return (
                  <ActionButton
                    key={`${action.href}:${action.label}`}
                    href={action.href}
                    icon={action.icon}
                    variant={action.variant}
                    className="px-4 py-2 text-sm"
                  >
                    {action.label}
                  </ActionButton>
                );
              }

              return (
                <ActionButton
                  key={action.label}
                  onClick={action.onClick}
                  icon={action.icon}
                  variant={action.variant}
                  className="px-4 py-2 text-sm"
                >
                  {action.label}
                </ActionButton>
              );
            })}
          </div>
        ) : null}
      </div>
    </section>
  );
}
