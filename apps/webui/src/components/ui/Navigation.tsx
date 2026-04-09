import type { IconDefinition } from "@fortawesome/fontawesome-svg-core";
import FaIcon from "../ui/FaIcon";

interface NavItem {
  href: string;
  label: string;
  icon: IconDefinition;
  description: string;
}

interface NavigationProps {
  items: NavItem[];
  currentPath: string;
}

export default function Navigation({ items, currentPath }: NavigationProps) {
  return (
    <nav
      className="border-y-4 border-[var(--theme-border-strong)] xl:mt-4 xl:flex-1 xl:border-y-0 xl:overflow-auto"
      aria-label="Primary navigation"
    >
      <div className="max-h-[52dvh] space-y-2 overflow-y-auto px-3 py-3 xl:max-h-none xl:block xl:space-y-0 xl:overflow-visible xl:px-0 xl:py-0">
        {items.map((item) => {
          const isActive =
            currentPath === item.href ||
            (item.href !== "/" && currentPath === item.href.slice(0, -1));

          return (
            <a
              key={item.href}
              href={item.href}
              aria-current={isActive ? "page" : undefined}
              className={`group flex w-full items-center gap-3 border-2 px-3 py-2 transition-all duration-100 xl:border-0 xl:border-l-4 xl:px-5 xl:py-4 ${
                isActive
                  ? "border-[var(--theme-accent-lime)] bg-[var(--theme-surface-alt)] xl:border-[var(--theme-accent-lime)] xl:shadow-[inset_4px_0_0_var(--theme-accent-lime)]"
                  : "border-[var(--theme-border)] bg-black hover:border-[var(--theme-border-strong)] hover:bg-[var(--theme-surface-hover)] xl:border-transparent"
              }`}
            >
              <div
                className={`flex h-9 w-9 items-center justify-center border-2 text-sm transition-all xl:h-11 xl:w-11 xl:text-lg ${
                  isActive
                    ? "border-[var(--theme-accent-lime)] bg-black text-[var(--theme-accent-lime)]"
                    : "border-[var(--theme-border-strong)] bg-black text-white group-hover:border-white"
                }`}
              >
                <FaIcon icon={item.icon} />
              </div>
              <div className="leading-tight">
                <div
                  className={`font-display text-xs font-bold uppercase tracking-wide xl:text-sm ${
                    isActive ? "text-white" : "text-zinc-100"
                  }`}
                >
                  {item.label}
                </div>
                <div className="font-mono text-xs text-zinc-500">
                  {item.description}
                </div>
              </div>
            </a>
          );
        })}
      </div>
    </nav>
  );
}
