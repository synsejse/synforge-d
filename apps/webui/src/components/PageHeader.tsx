import type { IconDefinition } from "@fortawesome/fontawesome-svg-core";
import FaIcon from "./FaIcon";

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
              const classes = [
                "inline-flex items-center border px-4 py-2 text-sm transition",
                (action.variant ?? "default") === "primary"
                  ? "border-zinc-200 bg-zinc-100 font-semibold text-black hover:bg-white"
                  : "border-zinc-800 bg-black text-zinc-200 hover:border-zinc-600 hover:bg-zinc-950",
              ].join(" ");

              const content = (
                <>
                  {action.icon ? <FaIcon icon={action.icon} className="mr-2 text-[0.95em]" /> : null}
                  {action.label}
                </>
              );

              if ("href" in action) {
                return (
                  <a key={`${action.href}:${action.label}`} href={action.href} className={classes}>
                    {content}
                  </a>
                );
              }

              return (
                <button key={action.label} onClick={action.onClick} className={classes}>
                  {content}
                </button>
              );
            })}
          </div>
        ) : null}
      </div>
    </section>
  );
}
