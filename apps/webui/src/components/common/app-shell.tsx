import { useEffect, useMemo, useState, type ReactNode } from "react";
import { useRouter } from "@tanstack/react-router";
import {
  faAnglesLeft,
  faAnglesRight,
  faBars,
  faBookOpen,
  faBoxesStacked,
  faChartLine,
  faChartSimple,
  faFolderTree,
  faGaugeHigh,
  faKey,
  faRightFromBracket,
  faSliders,
  faUsers,
  faXmark,
} from "@fortawesome/free-solid-svg-icons";
import Navigation, { type NavGroup, type NavItem } from "../ui/navigation";
import FaIcon from "../ui/fa-icon";
import Tooltip from "../ui/tooltip";
import { cn } from "../../lib/utils";
import api from "../../lib/api";
import { useSession } from "./session-provider";

const navGroups: NavGroup[] = [
  {
    label: "Operations",
    items: [
      { href: "/", label: "Overview", icon: faGaugeHigh, description: "System summary" },
      { href: "/jobs", label: "Jobs", icon: faChartLine, description: "Runs and live traces" },
      { href: "/statistics", label: "Statistics", icon: faChartSimple, description: "Telemetry and cache" },
    ],
  },
  {
    label: "Build",
    items: [
      { href: "/packages", label: "Packages", icon: faBoxesStacked, description: "Sources and history" },
      { href: "/repository", label: "Repository", icon: faFolderTree, description: "Published files" },
      { href: "/signing", label: "Signing", icon: faKey, description: "GPG metadata signing" },
    ],
  },
  {
    label: "Admin",
    items: [
      { href: "/users", label: "Users", icon: faUsers, description: "Accounts and permissions" },
      { href: "/settings", label: "Settings", icon: faSliders, description: "Daemon config" },
    ],
  },
];

const externalNav: NavItem[] = [
  {
    href: "/docs",
    label: "API Docs",
    icon: faBookOpen,
    description: "OpenAPI reference",
    external: true,
  },
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
  const { session } = useSession();
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

  const userInitial = useMemo(() => {
    const name = session?.user?.display_name ?? session?.user?.handle ?? "";
    const trimmed = name.trim();
    return trimmed ? trimmed.charAt(0).toUpperCase() : "?";
  }, [session]);

  const gridCols = isRail
    ? "lg:grid-cols-[64px_minmax(0,1fr)]"
    : "lg:grid-cols-[240px_minmax(0,1fr)] xl:grid-cols-[280px_minmax(0,1fr)]";

  return (
    <div className="box-border min-h-full w-full max-w-full px-2 py-2 sm:px-3 sm:py-3 lg:h-screen lg:overflow-hidden lg:px-5 lg:py-5">
      <div className={`grid min-h-full min-w-0 gap-3 lg:h-full ${gridCols}`}>
        <aside className="flex min-w-0 flex-col border-4 border-white app-section-band-vertical p-0 shadow-card-md lg:min-h-0">
          <SidebarHeader
            isRail={isRail}
            mobileNavOpen={mobileNavOpen}
            onMobileToggle={() => setMobileNavOpen((open) => !open)}
            onRailToggle={toggleRail}
          />

          <div
            id="mobile-nav-panel"
            className={cn(
              showNav ? "" : "hidden",
              "lg:flex lg:min-h-0 lg:flex-1 lg:flex-col",
            )}
          >
            <Navigation
              groups={navGroups}
              external={externalNav}
              rail={isRail}
              onNavigate={() => {
                if (!isDesktop) setMobileNavOpen(false);
              }}
            />

            <SidebarFooter
              isRail={isRail}
              userInitial={userInitial}
              displayName={session?.user?.display_name ?? null}
              handle={session?.user?.handle ?? null}
              onLogout={handleLogout}
              onRailToggle={toggleRail}
            />
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

interface SidebarHeaderProps {
  isRail: boolean;
  mobileNavOpen: boolean;
  onMobileToggle: () => void;
  onRailToggle: () => void;
}

function SidebarHeader({
  isRail,
  mobileNavOpen,
  onMobileToggle,
  onRailToggle,
}: SidebarHeaderProps) {
  return (
    <div
      className={cn(
        "border-b-4 border-edge-strong bg-black",
        isRail ? "px-2 py-3 lg:px-2" : "px-4 py-4 sm:px-5 sm:py-5 lg:px-4 lg:py-4",
      )}
    >
      <div
        className={cn(
          "flex items-center gap-3",
          isRail ? "lg:flex-col lg:gap-2" : "",
        )}
      >
        <div
          aria-hidden="true"
          className="flex h-10 w-10 shrink-0 items-center justify-center border-2 border-accent-lime bg-black font-mono text-base font-extrabold uppercase tracking-tighter text-accent-lime shadow-brutal-sm"
        >
          S
        </div>

        <div
          className={cn(
            "min-w-0 flex-1 pr-1 leading-none",
            isRail ? "lg:hidden" : "",
          )}
        >
          <div className="truncate font-mono text-[10px] font-bold uppercase tracking-[0.28em] text-accent-lime">
            Synforge
          </div>
          <h1 className="font-display mt-1.5 truncate text-base font-extrabold uppercase tracking-tighter text-white xl:text-lg">
            Build_Control
          </h1>
        </div>

        <Tooltip
          content={isRail ? "Expand sidebar" : "Collapse sidebar"}
          side={isRail ? "right" : "bottom"}
        >
          <button
            type="button"
            onClick={onRailToggle}
            aria-label={isRail ? "Expand sidebar" : "Collapse sidebar"}
            aria-pressed={isRail}
            className="hidden lg:inline-flex h-9 w-9 shrink-0 items-center justify-center border-2 border-edge-strong bg-black text-soft transition-colors hover:border-white hover:text-white focus:outline-none focus-visible:ring-2 focus-visible:ring-accent-lime"
          >
            <FaIcon icon={isRail ? faAnglesRight : faAnglesLeft} />
          </button>
        </Tooltip>

        <button
          type="button"
          aria-controls="mobile-nav-panel"
          aria-expanded={mobileNavOpen}
          aria-label={mobileNavOpen ? "Close menu" : "Open menu"}
          onClick={onMobileToggle}
          className="lg:hidden inline-flex h-10 w-10 shrink-0 items-center justify-center border-2 border-edge-strong bg-black text-strong transition-colors hover:border-white hover:bg-surface-hover focus:outline-none focus-visible:ring-2 focus-visible:ring-accent-lime"
        >
          <FaIcon icon={mobileNavOpen ? faXmark : faBars} />
        </button>
      </div>
    </div>
  );
}

interface SidebarFooterProps {
  isRail: boolean;
  userInitial: string;
  displayName: string | null;
  handle: string | null;
  onLogout: () => void;
  onRailToggle: () => void;
}

function SidebarFooter({
  isRail,
  userInitial,
  displayName,
  handle,
  onLogout,
  onRailToggle,
}: SidebarFooterProps) {
  if (isRail) {
    return (
      <div className="border-t-4 border-edge-strong bg-black lg:mt-auto">
        <div className="hidden lg:flex flex-col items-center gap-2 px-2 py-3">
          <Tooltip
            content={
              <div className="flex flex-col gap-0.5">
                <span className="font-display text-xs font-bold uppercase tracking-[0.12em] text-white">
                  {displayName ?? "Account"}
                </span>
                {handle ? (
                  <span className="font-mono text-[10px] normal-case tracking-normal text-soft">
                    @{handle}
                  </span>
                ) : null}
              </div>
            }
            side="right"
          >
            <div
              aria-hidden="true"
              className="flex h-9 w-9 items-center justify-center border-2 border-edge-strong bg-black font-mono text-sm font-extrabold uppercase text-white"
            >
              {userInitial}
            </div>
          </Tooltip>
          <Tooltip content="Sign out" side="right">
            <button
              type="button"
              onClick={onLogout}
              aria-label="Sign out"
              className="flex h-9 w-9 items-center justify-center border-2 border-edge-strong bg-black text-soft transition-colors hover:border-accent-lime hover:text-accent-lime focus:outline-none focus-visible:ring-2 focus-visible:ring-accent-lime"
            >
              <FaIcon icon={faRightFromBracket} />
            </button>
          </Tooltip>
        </div>
      </div>
    );
  }

  return (
    <div className="border-t-4 border-edge-strong bg-black lg:mt-auto">
      <div className="px-3 py-3 sm:px-4 sm:py-4">
        <div className="font-mono text-[10px] font-bold uppercase tracking-[0.28em] text-soft">
          Session
        </div>
        <div className="mt-2 flex items-center gap-3 border-2 border-edge bg-black px-3 py-2.5">
          <div
            aria-hidden="true"
            className="flex h-9 w-9 shrink-0 items-center justify-center border-2 border-accent-lime bg-black font-mono text-sm font-extrabold uppercase text-accent-lime"
          >
            {userInitial}
          </div>
          <div className="min-w-0 flex-1 leading-tight">
            <div className="truncate font-display text-sm font-bold uppercase tracking-wide text-white">
              {displayName ?? "Account"}
            </div>
            {handle ? (
              <div className="truncate font-mono text-[11px] text-soft">
                @{handle}
              </div>
            ) : null}
          </div>
          <Tooltip content="Sign out" side="top">
            <button
              type="button"
              onClick={onLogout}
              aria-label="Sign out"
              className="inline-flex h-9 w-9 shrink-0 items-center justify-center border-2 border-edge-strong bg-black text-soft transition-colors hover:border-accent-lime hover:text-accent-lime focus:outline-none focus-visible:ring-2 focus-visible:ring-accent-lime"
            >
              <FaIcon icon={faRightFromBracket} />
            </button>
          </Tooltip>
        </div>
      </div>
      <div className="hidden lg:block border-t-2 border-edge bg-black">
        <button
          type="button"
          onClick={onRailToggle}
          aria-label="Collapse sidebar"
          aria-pressed={false}
          className="flex w-full items-center justify-center gap-2 px-3 py-2.5 font-mono text-[11px] font-bold uppercase tracking-[0.18em] text-soft transition-colors hover:bg-surface-hover hover:text-white focus:outline-none focus-visible:ring-2 focus-visible:ring-accent-lime"
        >
          <FaIcon icon={faAnglesLeft} />
          <span>Collapse</span>
        </button>
      </div>
    </div>
  );
}
