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

type HeaderColor = "lime" | "green" | "orange" | "cyan" | "purple" | "white";

const colorMap: Record<HeaderColor, { border: string; text: string }> = {
  lime: {
    border: "border-[var(--theme-accent-lime)]",
    text: "text-[var(--theme-accent-lime)]",
  },
  green: {
    border: "border-[var(--theme-terminal-green)]",
    text: "text-[var(--theme-terminal-green)]",
  },
  orange: {
    border: "border-[var(--theme-accent-orange)]",
    text: "text-[var(--theme-accent-orange)]",
  },
  cyan: {
    border: "border-cyan-400",
    text: "text-cyan-400",
  },
  purple: {
    border: "border-purple-400",
    text: "text-purple-400",
  },
  white: {
    border: "border-white",
    text: "text-white",
  },
};

interface Props {
  eyebrow: string;
  title: string;
  description: string;
  color?: HeaderColor;
  actions?: Array<ActionLink | ActionButton>;
}

export default function PageHeader({
  eyebrow,
  title,
  description,
  color = "lime",
  actions = [],
}: Props) {
  const colors = colorMap[color];

  return (
    <section className={`border-4 ${colors.border} bg-black p-6`}>
      <div className="flex flex-col gap-4 xl:flex-row xl:items-start xl:justify-between">
        <div className="min-w-0 flex-1">
          <p className={`font-mono text-xs font-bold uppercase tracking-[0.3em] ${colors.text}`}>
            {eyebrow}
          </p>
          <h1 className="mt-2 font-mono text-3xl font-bold uppercase text-white">
            {title}
          </h1>
          <p className="mt-2 text-sm text-zinc-400">{description}</p>
        </div>
        {actions.length > 0 ? (
          <div className="flex flex-wrap gap-3">
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
