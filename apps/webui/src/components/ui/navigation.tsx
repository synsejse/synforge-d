import { Link } from "@tanstack/react-router";
import type { IconDefinition } from "@fortawesome/fontawesome-svg-core";
import FaIcon from "./fa-icon";

interface NavItem {
  href: string;
  label: string;
  icon: IconDefinition;
  description: string;
}

interface NavigationProps {
  items: NavItem[];
  onNavigate?: () => void;
}

export default function Navigation({ items, onNavigate }: NavigationProps) {
  return (
    <nav
      className="border-y-4 border-[var(--theme-border-strong)] lg:mt-4 lg:flex-1 lg:border-y-0 lg:overflow-auto"
      aria-label="Primary navigation"
    >
      <div className="max-h-[52dvh] space-y-2 overflow-y-auto px-3 py-3 lg:max-h-none lg:block lg:space-y-0 lg:overflow-visible lg:px-0 lg:py-0">
        {items.map((item) => (
          <Link
            key={item.href}
            to={item.href}
            onClick={onNavigate}
            activeOptions={{ exact: item.href === "/" }}
            className="group flex w-full items-center gap-3 border-2 border-[var(--theme-border)] bg-black px-3 py-2 transition-all duration-100 hover:border-[var(--theme-border-strong)] hover:bg-[var(--theme-surface-hover)] lg:border-0 lg:border-l-4 lg:border-transparent lg:px-5 lg:py-4"
            activeProps={{
              "aria-current": "page",
              className:
                "group flex w-full items-center gap-3 border-2 border-[var(--theme-accent-lime)] bg-[var(--theme-surface-alt)] px-3 py-2 transition-all duration-100 lg:border-0 lg:border-l-4 lg:px-5 lg:py-4 lg:border-[var(--theme-accent-lime)] lg:shadow-[inset_4px_0_0_var(--theme-accent-lime)]",
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
                <div className="leading-tight">
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
        ))}
      </div>
    </nav>
  );
}
