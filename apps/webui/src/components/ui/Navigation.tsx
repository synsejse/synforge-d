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
    <nav className="mt-4 flex-1 space-y-0 overflow-auto" aria-label="Primary navigation">
      {items.map((item) => {
        const isActive =
          currentPath === item.href ||
          (item.href !== "/" && currentPath === item.href.slice(0, -1));

        return (
          <a
            key={item.href}
            href={item.href}
            aria-current={isActive ? "page" : undefined}
            className={`group block border-l-4 px-5 py-4 transition-all duration-100 ${
              isActive
                ? "border-[var(--theme-accent-lime)] bg-[var(--theme-surface-alt)] shadow-[inset_4px_0_0_var(--theme-accent-lime)]"
                : "border-transparent hover:border-[var(--theme-border-strong)] hover:bg-[var(--theme-surface-hover)]"
            }`}
          >
            <div className="flex items-center gap-4">
              <div
                className={`flex h-11 w-11 items-center justify-center border-2 text-lg transition-all ${
                  isActive
                    ? "border-[var(--theme-accent-lime)] bg-black text-[var(--theme-accent-lime)]"
                    : "border-[var(--theme-border-strong)] bg-black text-white group-hover:border-white"
                }`}
              >
                <FaIcon icon={item.icon} />
              </div>
              <div>
                <div
                  className={`font-display text-sm font-bold uppercase tracking-wide ${
                    isActive ? "text-white" : "text-zinc-100"
                  }`}
                >
                  {item.label}
                </div>
                <div className="font-mono text-xs text-zinc-500">
                  {item.description}
                </div>
              </div>
            </div>
          </a>
        );
      })}
    </nav>
  );
}
