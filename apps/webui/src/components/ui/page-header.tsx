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
    border: "border-accent-lime",
    text: "text-accent-lime",
  },
  green: {
    border: "border-success",
    text: "text-success",
  },
  orange: {
    border: "border-accent-orange",
    text: "text-accent-orange",
  },
  cyan: {
    border: "border-accent-cyan",
    text: "text-accent-cyan",
  },
  purple: {
    border: "border-accent-violet",
    text: "text-accent-violet",
  },
  white: {
    border: "border-white",
    text: "text-white",
  },
};

interface Props {
  /** Optional small uppercase tag above the title. Most pages omit this. */
  eyebrow?: string;
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
  const stickyClasses = sticky ? "sticky top-0 z-10 bg-black" : "";

  return (
    <section
      className={`border-b-2 ${colors.border} pb-4 ${stickyClasses}`}
    >
      <div className="flex flex-col gap-4 lg:flex-row lg:items-end lg:justify-between">
        <div className="min-w-0 flex-1">
          {eyebrow ? (
            <p
              className={`font-mono text-[10px] font-bold uppercase tracking-[0.3em] ${colors.text}`}
            >
              {eyebrow}
            </p>
          ) : null}
          <h1
            className={`font-mono text-2xl font-bold uppercase text-white sm:text-3xl ${eyebrow ? "mt-2" : ""}`}
          >
            {title}
          </h1>
          {description ? (
            <p className="mt-2 max-w-3xl text-sm text-muted">{description}</p>
          ) : null}
        </div>
        {actions.length > 0 ? (
          <div className="flex flex-wrap gap-2 lg:flex-nowrap">
            {actions.map((action) => {
              if ("to" in action) {
                return (
                  <ActionButton
                    key={`${String(action.to)}:${action.label}`}
                    to={action.to}
                    icon={action.icon}
                    variant={action.variant}
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
