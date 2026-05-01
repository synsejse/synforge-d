import { useEffect, useState, type ReactNode } from "react";
import { useRouter } from "@tanstack/react-router";
import {
  faAnglesLeft,
  faAnglesRight,
  faBoxesStacked,
  faBookOpen,
  faChartSimple,
  faChartLine,
  faFolderTree,
  faGaugeHigh,
  faKey,
  faRightFromBracket,
  faSliders,
  faUsers,
} from "@fortawesome/free-solid-svg-icons";
import Navigation from "../ui/navigation";
import FaIcon from "../ui/fa-icon";
import Tooltip from "../ui/tooltip";
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

const DESKTOP_QUERY = "(min-width: 1024px)";
const RAIL_STORAGE_KEY = "synforge.sidebarRail";

function readRailPreference(): boolean {
  if (typeof window === "undefined") return false;
  try {
    return window.localStorage.getItem(RAIL_STORAGE_KEY) === "1";
  } catch {
    return false;
  }
}

function writeRailPreference(value: boolean): void {
  try {
    window.localStorage.setItem(RAIL_STORAGE_KEY, value ? "1" : "0");
  } catch {
    /* localStorage disabled — preference is session-only */
  }
}

export default function AppShell({ children }: { children: ReactNode }) {
  const router = useRouter();
  const [mobileNavOpen, setMobileNavOpen] = useState(false);
  const [isDesktop, setIsDesktop] = useState(() =>
    typeof window === "undefined" ? true : window.matchMedia(DESKTOP_QUERY).matches,
  );
  const [railCollapsed, setRailCollapsed] = useState(readRailPreference);

  useEffect(() => {
    const mql = window.matchMedia(DESKTOP_QUERY);
    const onChange = () => setIsDesktop(mql.matches);
    mql.addEventListener("change", onChange);
    return () => mql.removeEventListener("change", onChange);
  }, []);

  const showNav = isDesktop || mobileNavOpen;
  // Rail mode only applies on desktop. On mobile, the panel is full-width when open.
  const isRail = isDesktop && railCollapsed;

  const toggleRail = () => {
    setRailCollapsed((prev) => {
      const next = !prev;
      writeRailPreference(next);
      return next;
    });
  };

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

  const gridCols = isRail
    ? "lg:grid-cols-[64px_minmax(0,1fr)]"
    : "lg:grid-cols-[240px_minmax(0,1fr)] xl:grid-cols-[280px_minmax(0,1fr)]";

  return (
    <div className="box-border min-h-full w-full max-w-full px-2 py-2 sm:px-3 sm:py-3 lg:h-screen lg:overflow-hidden lg:px-5 lg:py-5">
      <div className={`grid min-h-full min-w-0 gap-3 lg:h-full ${gridCols}`}>
        <aside className="flex min-w-0 flex-col border-4 border-white app-section-band-vertical p-0 shadow-card-md lg:min-h-0">
          <div className="border-b-4 border-edge-strong bg-black px-4 py-4 sm:px-6 sm:py-5 lg:px-3 lg:py-3">
            <div className="flex items-center justify-between gap-3">
              <div className={`min-w-0 ${isRail ? "lg:hidden" : ""}`}>
                <div className="font-mono text-xs font-bold uppercase tracking-[0.35em] text-accent-lime">
                  Synforge
                </div>
                <h1 className="font-display mt-2 text-xl font-extrabold uppercase tracking-tighter text-white sm:text-2xl">
                  Build_Control
                </h1>
              </div>
              {isRail ? (
                <div
                  aria-hidden="true"
                  className="hidden lg:flex h-10 w-10 items-center justify-center border-2 border-accent-lime bg-black font-mono text-base font-extrabold uppercase tracking-tighter text-accent-lime"
                >
                  S
                </div>
              ) : null}
              <button
                type="button"
                aria-controls="mobile-nav-panel"
                aria-expanded={mobileNavOpen}
                onClick={() => setMobileNavOpen((open) => !open)}
                className="lg:hidden border-2 border-edge-strong bg-black px-3 py-2 font-mono text-xs font-bold uppercase tracking-[0.15em] text-strong transition hover:border-white hover:bg-surface-hover focus:outline-none focus:ring-2 focus:ring-accent-lime"
              >
                {mobileNavOpen ? "Close" : "Menu"}
              </button>
            </div>
          </div>

          <div
            id="mobile-nav-panel"
            className={`${showNav ? "" : "hidden"} lg:flex lg:min-h-0 lg:flex-1 lg:flex-col`}
          >
            <Navigation
              items={navItems}
              rail={isRail}
              onNavigate={() => {
                if (!isDesktop) setMobileNavOpen(false);
              }}
            />

            <div className="border-t-4 border-edge-strong lg:mt-auto">
              <div className={`bg-black ${isRail ? "lg:px-2 lg:py-3" : "px-5 py-5"}`}>
                {!isRail ? (
                  <div className="font-mono text-xs font-bold uppercase tracking-[0.25em] text-soft">
                    Session
                  </div>
                ) : null}
                {isRail ? (
                  <div className="hidden lg:flex flex-col gap-2">
                    <Tooltip content="Change account" side="right">
                      <button
                        type="button"
                        onClick={handleLogout}
                        aria-label="Change account"
                        className="flex h-10 items-center justify-center border-2 border-edge-strong bg-black text-strong transition hover:border-white hover:bg-surface-hover focus:outline-none focus:ring-2 focus:ring-accent-lime"
                      >
                        <FaIcon icon={faRightFromBracket} />
                      </button>
                    </Tooltip>
                  </div>
                ) : (
                  <button
                    type="button"
                    onClick={handleLogout}
                    className="mt-3 w-full border-2 border-edge-strong bg-black px-4 py-2.5 text-left font-mono text-sm font-medium text-strong transition hover:border-white hover:bg-surface-hover focus:outline-none focus:ring-2 focus:ring-accent-lime"
                  >
                    Change account
                  </button>
                )}
              </div>
              <div className="hidden lg:block border-t-2 border-edge bg-black">
                <Tooltip
                  content={isRail ? "Expand sidebar" : "Collapse sidebar"}
                  side="right"
                >
                  <button
                    type="button"
                    onClick={toggleRail}
                    aria-label={isRail ? "Expand sidebar" : "Collapse sidebar"}
                    aria-pressed={isRail}
                    className={`group flex w-full items-center gap-2 px-4 py-3 font-mono text-xs font-bold uppercase tracking-[0.18em] text-soft transition hover:bg-surface-hover hover:text-white focus:outline-none focus:ring-2 focus:ring-accent-lime ${isRail ? "justify-center" : ""}`}
                  >
                    <FaIcon icon={isRail ? faAnglesRight : faAnglesLeft} />
                    {!isRail ? <span>Collapse</span> : null}
                  </button>
                </Tooltip>
              </div>
            </div>
          </div>
        </aside>

        <main
          id="main-content"
          tabIndex={-1}
          className="min-h-0 min-w-0 overflow-x-auto overflow-y-auto border-4 border-white bg-black p-3 shadow-card-md sm:p-5 lg:p-8"
        >
          <div className="mx-auto min-w-0 max-w-[96rem]">{children}</div>
        </main>
      </div>
    </div>
  );
}
