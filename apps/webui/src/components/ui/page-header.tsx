import type { IconDefinition } from "@fortawesome/fontawesome-svg-core";
import type { LinkProps } from "@tanstack/react-router";
import ActionButton from "./action-button";

interface ActionLink {
  to: LinkProps["to"];
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
    border: "border-[var(--theme-accent-cyan)]",
    text: "text-[var(--theme-accent-cyan)]",
  },
  purple: {
    border: "border-[var(--theme-accent-violet)]",
    text: "text-[var(--theme-accent-violet)]",
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
  sticky?: boolean;
}

export default function PageHeader({
  eyebrow,
  title,
  description,
  color = "lime",
  actions = [],
  sticky = false,
}: Props) {
  const colors = colorMap[color];
  const stickyClasses = sticky ? "sticky top-0 z-10" : "";

  return (
    <section className={`border-4 ${colors.border} bg-black p-6 ${stickyClasses}`}>
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
              if ("to" in action) {
                return (
                  <ActionButton
                    key={`${String(action.to)}:${action.label}`}
                    to={action.to}
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
