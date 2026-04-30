import { useEffect, useState, type ReactNode } from "react";
import { useRouter } from "@tanstack/react-router";
import {
  faBoxesStacked,
  faBookOpen,
  faChartSimple,
  faChartLine,
  faFolderTree,
  faGaugeHigh,
  faKey,
  faSliders,
  faUsers,
} from "@fortawesome/free-solid-svg-icons";
import Navigation from "../ui/navigation";
import api from "../../lib/api";

const navItems = [
  { href: "/", label: "Overview", icon: faGaugeHigh, description: "System summary" },
  { href: "/packages", label: "Packages", icon: faBoxesStacked, description: "Sources and history" },
  { href: "/statistics", label: "Statistics", icon: faChartSimple, description: "Telemetry and cache" },
  { href: "/repository", label: "Repository", icon: faFolderTree, description: "Published files" },
  { href: "/jobs", label: "Jobs", icon: faChartLine, description: "Runs and live traces" },
  { href: "/users", label: "Users", icon: faUsers, description: "Accounts and permissions" },
  { href: "/docs", label: "Docs", icon: faBookOpen, description: "API reference" },
  { href: "/signing", label: "Signing", icon: faKey, description: "GPG metadata signing" },
  { href: "/settings", label: "Settings", icon: faSliders, description: "Daemon config" },
];

const DESKTOP_QUERY = "(min-width: 1280px)";

export default function AppShell({ children }: { children: ReactNode }) {
  const router = useRouter();
  const [mobileNavOpen, setMobileNavOpen] = useState(false);
  const [isDesktop, setIsDesktop] = useState(() =>
    typeof window === "undefined" ? true : window.matchMedia(DESKTOP_QUERY).matches,
  );

  useEffect(() => {
    const mql = window.matchMedia(DESKTOP_QUERY);
    const onChange = () => setIsDesktop(mql.matches);
    mql.addEventListener("change", onChange);
    return () => mql.removeEventListener("change", onChange);
  }, []);

  const showNav = isDesktop || mobileNavOpen;

  const handleLogout = async () => {
    try {
      await api.logout();
    } finally {
      router.navigate({
        to: "/login",
        search: { message: "Enter account credentials to continue." },
      });
    }
  };

  return (
    <div className="box-border min-h-full w-full max-w-full px-2 py-2 sm:px-3 sm:py-3 lg:h-screen lg:overflow-hidden lg:px-5 lg:py-5">
      <div className="grid min-h-full min-w-0 gap-3 xl:h-full xl:grid-cols-[280px_minmax(0,1fr)]">
        <aside className="flex min-w-0 flex-col border-4 border-white bg-gradient-to-b from-zinc-900 to-black p-0 shadow-[6px_6px_0_rgba(255,255,255,0.2)] xl:min-h-0">
          <div className="border-b-4 border-[var(--theme-border-strong)] bg-black px-4 py-4 sm:px-6 sm:py-5">
            <div className="flex items-center justify-between gap-3">
              <div className="min-w-0">
                <div className="font-mono text-xs font-bold uppercase tracking-[0.35em] text-[var(--theme-accent-lime)]">
                  Synforge
                </div>
                <h1 className="font-display mt-2 text-xl font-extrabold uppercase tracking-tighter text-white sm:text-2xl">
                  Build_Control
                </h1>
              </div>
              <button
                type="button"
                aria-controls="mobile-nav-panel"
                aria-expanded={mobileNavOpen}
                onClick={() => setMobileNavOpen((open) => !open)}
                className="xl:hidden border-2 border-[var(--theme-border-strong)] bg-black px-3 py-2 font-mono text-xs font-bold uppercase tracking-[0.15em] text-zinc-200 transition hover:border-white hover:bg-[var(--theme-surface-hover)] focus:outline-none focus:ring-2 focus:ring-[var(--theme-accent-lime)]"
              >
                {mobileNavOpen ? "Close" : "Menu"}
              </button>
            </div>
          </div>

          <div
            id="mobile-nav-panel"
            className={`${showNav ? "" : "hidden"} xl:flex xl:min-h-0 xl:flex-1 xl:flex-col`}
          >
            <Navigation
              items={navItems}
              onNavigate={() => {
                if (!isDesktop) setMobileNavOpen(false);
              }}
            />

            <div className="border-t-4 border-[var(--theme-border-strong)] xl:mt-auto">
              <div className="bg-black px-5 py-5">
                <div className="font-mono text-xs font-bold uppercase tracking-[0.25em] text-zinc-500">
                  Session
                </div>
                <button
                  type="button"
                  onClick={handleLogout}
                  className="mt-3 w-full border-2 border-[var(--theme-border-strong)] bg-black px-4 py-2.5 text-left font-mono text-sm font-medium text-zinc-200 transition hover:border-white hover:bg-[var(--theme-surface-hover)] focus:outline-none focus:ring-2 focus:ring-[var(--theme-accent-lime)]"
                >
                  Change account
                </button>
              </div>
            </div>
          </div>
        </aside>

        <main
          id="main-content"
          tabIndex={-1}
          className="min-h-0 min-w-0 overflow-x-auto overflow-y-auto border-4 border-white bg-black p-3 shadow-[6px_6px_0_rgba(255,255,255,0.2)] sm:p-5 lg:p-8"
        >
          <div className="min-w-0">{children}</div>
        </main>
      </div>
    </div>
  );
}
