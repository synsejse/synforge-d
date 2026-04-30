import { Link } from "@tanstack/react-router";
import type { IconDefinition } from "@fortawesome/fontawesome-svg-core";
import FaIcon from "./fa-icon";
import Tooltip from "./tooltip";

interface NavItem {
  href: string;
  label: string;
  icon: IconDefinition;
  description: string;
}

interface NavigationProps {
  items: NavItem[];
  onNavigate?: () => void;
  /** When true, render icon-only on desktop (lg+). Mobile is unaffected. */
  rail?: boolean;
}

export default function Navigation({
  items,
  onNavigate,
  rail = false,
}: NavigationProps) {
  return (
    <nav
      className="border-y-4 border-[var(--theme-border-strong)] lg:mt-4 lg:flex-1 lg:border-y-0 lg:overflow-auto"
      aria-label="Primary navigation"
    >
      <div
        className={`max-h-[52dvh] space-y-2 overflow-y-auto px-3 py-3 lg:max-h-none lg:block lg:space-y-0 lg:overflow-visible lg:py-0 ${rail ? "lg:px-2" : "lg:px-0"}`}
      >
        {items.map((item) => (
          <NavLink
            key={item.href}
            item={item}
            onNavigate={onNavigate}
            rail={rail}
          />
        ))}
      </div>
    </nav>
  );
}

interface NavLinkProps {
  item: NavItem;
  onNavigate?: () => void;
  rail: boolean;
}

function NavLink({ item, onNavigate, rail }: NavLinkProps) {
  const baseClass =
    "group flex w-full items-center gap-3 border-2 border-[var(--theme-border)] bg-black px-3 py-2 transition-all duration-100 hover:border-[var(--theme-border-strong)] hover:bg-[var(--theme-surface-hover)]";
  const expandedDesktop =
    "lg:border-0 lg:border-l-4 lg:border-transparent lg:px-5 lg:py-4";
  const railDesktop = "lg:border-0 lg:border-l-4 lg:border-transparent lg:px-2 lg:py-3 lg:justify-center";

  const link = (
    <Link
      to={item.href}
      onClick={onNavigate}
      activeOptions={{ exact: item.href === "/" }}
      className={`${baseClass} ${rail ? railDesktop : expandedDesktop}`}
      activeProps={{
        "aria-current": "page",
        className: `group flex w-full items-center gap-3 border-2 border-[var(--theme-accent-lime)] bg-[var(--theme-surface-alt)] px-3 py-2 transition-all duration-100 lg:border-0 lg:border-l-4 lg:border-[var(--theme-accent-lime)] lg:shadow-[inset_4px_0_0_var(--theme-accent-lime)] ${rail ? "lg:px-2 lg:py-3 lg:justify-center" : "lg:px-5 lg:py-4"}`,
      }}
    >
      {({ isActive }) => (
        <>
          <div
            className={`flex h-9 w-9 items-center justify-center border-2 text-sm transition-all lg:h-11 lg:w-11 lg:text-lg ${
              isActive
                ? "border-[var(--theme-accent-lime)] bg-black text-[var(--theme-accent-lime)]"
                : "border-[var(--theme-border-strong)] bg-black text-white group-hover:border-white"
            }`}
          >
            <FaIcon icon={item.icon} />
          </div>
          <div className={`leading-tight ${rail ? "lg:hidden" : ""}`}>
            <div
              className={`font-display text-xs font-bold uppercase tracking-wide lg:text-sm ${
                isActive ? "text-white" : "text-zinc-100"
              }`}
            >
              {item.label}
            </div>
            <div className="font-mono text-xs text-zinc-500">
              {item.description}
            </div>
          </div>
        </>
      )}
    </Link>
  );

  if (!rail) {
    return link;
  }

  return (
    <Tooltip content={item.label} side="right">
      {link}
    </Tooltip>
  );
}
