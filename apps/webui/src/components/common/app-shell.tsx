import { useEffect, useMemo, useState, type ReactNode } from "react";
import { useNavigate } from "@tanstack/react-router";
import { useQuery, useQueryClient } from "@tanstack/react-query";
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
import Button from "../ui/button";
import FaIcon from "../ui/fa-icon";
import Tooltip from "../ui/tooltip";
import { cn } from "../../lib/utils";
import api from "../../lib/api";
import { dashboardQueries } from "../../lib/queries";
import BrandMark from "./brand-mark";
import { useSession } from "./session-provider";
import { usePageVisible } from "./page-visibility-provider";
import { KeyboardShortcutsProvider } from "./keyboard-shortcuts";
import SystemNoticeBar from "./system-notice-bar";

function buildNavGroups(activeJobCount: number): NavGroup[] {
  return [
    {
      label: "Operations",
      items: [
        { href: "/", label: "Overview", icon: faGaugeHigh, description: "System summary" },
        {
          href: "/jobs",
          label: "Jobs",
          icon: faChartLine,
          description: "Runs and live traces",
          badge: activeJobCount > 0 ? activeJobCount : null,
          badgeTone: "lime",
        },
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
}

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
  const navigate = useNavigate();
  const queryClient = useQueryClient();
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
    } catch {
      // Bail out of the authed UI either way — server may already
      // consider us logged out, network may be flaky.
    }
    // Drop the cached session before navigating. Without this, the
    // useSession hook keeps returning the stale logged-in record and
    // anything reading it (sidebar, _authed guard on remount) thinks
    // we're still signed in, which prevented the navigate from
    // landing on /login.
    queryClient.removeQueries({ queryKey: ["session"] });
    navigate({
      to: "/login",
      search: { message: "Enter account credentials to continue." },
    });
  };

  const userInitial = useMemo(() => {
    const name = session?.user?.display_name ?? session?.user?.handle ?? "";
    const trimmed = name.trim();
    return trimmed ? trimmed.charAt(0).toUpperCase() : "?";
  }, [session]);

  // Live system pulse — shared with the sidebar tick + nav badges. Reuses
  // the dashboard overview query so this fetch dedups with the dashboard
  // page's own poll when the user is there.
  const pageVisible = usePageVisible();
  const overviewQuery = useQuery({
    ...dashboardQueries.overview(),
    refetchInterval: pageVisible ? 15_000 : false,
  });
  const activeJobCount = overviewQuery.data?.activeJobCount ?? 0;
  const navGroups = useMemo(
    () => buildNavGroups(activeJobCount),
    [activeJobCount],
  );

  const gridCols = isRail
    ? "lg:grid-cols-[64px_minmax(0,1fr)]"
    : "lg:grid-cols-[240px_minmax(0,1fr)] xl:grid-cols-[280px_minmax(0,1fr)]";

  return (
    <KeyboardShortcutsProvider>
    <SystemNoticeBar />
    <div className="box-border min-h-full w-full max-w-full px-2 py-2 sm:px-3 sm:py-3 lg:h-screen lg:overflow-hidden lg:px-5 lg:py-5">
      <div className={`grid min-h-full min-w-0 gap-3 lg:h-full ${gridCols}`}>
        <aside className="flex min-w-0 flex-col border border-edge-strong app-section-band-vertical p-0 lg:min-h-0">
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

            <SidebarSystemTick
              isRail={isRail}
              activeJobCount={activeJobCount}
              live={pageVisible}
            />

            <SidebarFooter
              isRail={isRail}
              userInitial={userInitial}
              displayName={session?.user?.display_name ?? null}
              handle={session?.user?.handle ?? null}
              onLogout={handleLogout}
            />
          </div>
        </aside>

        <main
          id="main-content"
          tabIndex={-1}
          className="min-h-0 min-w-0 overflow-x-auto overflow-y-auto border border-edge-strong bg-black p-3 [scrollbar-gutter:stable] sm:p-5 lg:p-8"
        >
          <div className="mx-auto min-w-0 max-w-[96rem]">{children}</div>
        </main>
      </div>
    </div>
    </KeyboardShortcutsProvider>
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
        "border-b border-edge bg-black",
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
          className="flex h-10 w-10 shrink-0 items-center justify-center border border-accent-lime bg-black"
        >
          <BrandMark className="h-6 w-6" />
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
          <Button
            variant="ghost"
            size="icon"
            onClick={onRailToggle}
            aria-label={isRail ? "Expand sidebar" : "Collapse sidebar"}
            aria-pressed={isRail}
            className="hidden lg:inline-flex shrink-0"
          >
            <FaIcon icon={isRail ? faAnglesRight : faAnglesLeft} />
          </Button>
        </Tooltip>

        <Button
          variant="ghost"
          size="icon"
          aria-controls="mobile-nav-panel"
          aria-expanded={mobileNavOpen}
          aria-label={mobileNavOpen ? "Close menu" : "Open menu"}
          onClick={onMobileToggle}
          className="lg:hidden h-10 w-10 shrink-0"
        >
          <FaIcon icon={mobileNavOpen ? faXmark : faBars} />
        </Button>
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
}

interface SidebarSystemTickProps {
  isRail: boolean;
  activeJobCount: number;
  /** Whether the page is visible — pauses the clock animation when hidden. */
  live: boolean;
}

function SidebarSystemTick({
  isRail,
  activeJobCount,
  live,
}: SidebarSystemTickProps) {
  const [now, setNow] = useState(() => new Date());
  useEffect(() => {
    if (!live) return;
    const id = window.setInterval(() => setNow(new Date()), 1000);
    return () => window.clearInterval(id);
  }, [live]);

  const clock = formatClock(now);
  const isBuilding = activeJobCount > 0;

  if (isRail) {
    return (
      <div className="hidden lg:block border-t border-edge bg-black">
        <Tooltip
          content={
            <div className="flex flex-col gap-0.5">
              <span className="font-display text-xs font-bold uppercase tracking-[0.12em] text-white">
                {isBuilding ? `Building · ${activeJobCount} active` : "Idle"}
              </span>
              <span className="font-mono text-[10px] tracking-normal text-soft">
                Local time {clock}
              </span>
            </div>
          }
          side="right"
        >
          <div
            className="flex flex-col items-center gap-1 px-2 py-2"
            aria-label={isBuilding ? `${activeJobCount} active builds` : "Idle"}
          >
            <span
              className={cn(
                "h-2 w-2",
                isBuilding ? "bg-accent-lime animate-pulse" : "bg-soft",
              )}
            />
            <span className="font-mono text-[9px] tabular-nums tracking-wide text-soft">
              {clock.slice(0, 5)}
            </span>
          </div>
        </Tooltip>
      </div>
    );
  }

  return (
    <div className="border-t border-edge bg-black px-3 py-2">
      <div className="flex items-center justify-between gap-3 font-mono text-[10px] uppercase tracking-[0.18em]">
        <div className="flex items-center gap-2">
          <span
            aria-hidden="true"
            className={cn(
              "h-2 w-2 shrink-0",
              isBuilding ? "bg-accent-lime animate-pulse" : "bg-soft",
            )}
          />
          <span
            className={cn(
              "font-bold",
              isBuilding ? "text-accent-lime" : "text-soft",
            )}
          >
            {isBuilding ? `Building · ${activeJobCount}` : "Idle"}
          </span>
        </div>
        <span className="tabular-nums text-soft">{clock}</span>
        {live ? (
          <span
            aria-hidden="true"
            className="inline-block h-3 w-[2px] animate-pulse bg-accent-lime"
          />
        ) : null}
      </div>
    </div>
  );
}

function formatClock(date: Date): string {
  const h = date.getHours().toString().padStart(2, "0");
  const m = date.getMinutes().toString().padStart(2, "0");
  const s = date.getSeconds().toString().padStart(2, "0");
  return `${h}:${m}:${s}`;
}

function SidebarFooter({
  isRail,
  userInitial,
  displayName,
  handle,
  onLogout,
}: SidebarFooterProps) {
  if (isRail) {
    return (
      <div className="border-t border-edge-strong bg-black lg:mt-auto">
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
              className="flex h-9 w-9 items-center justify-center border border-edge bg-black font-mono text-sm font-extrabold uppercase text-white"
            >
              {userInitial}
            </div>
          </Tooltip>
          <Tooltip content="Sign out" side="right">
            <Button
              variant="ghost"
              size="icon"
              onClick={onLogout}
              aria-label="Sign out"
              className="text-soft hover:border-accent-lime hover:text-accent-lime"
            >
              <FaIcon icon={faRightFromBracket} />
            </Button>
          </Tooltip>
        </div>
      </div>
    );
  }

  return (
    <div className="border-t border-edge-strong bg-black lg:mt-auto">
      <div className="px-3 py-3 sm:px-4 sm:py-4">
        <div className="font-mono text-[10px] font-bold uppercase tracking-[0.28em] text-soft">
          Session
        </div>
        <div className="mt-2 flex items-center gap-3 border border-edge bg-black px-3 py-2.5">
          <div
            aria-hidden="true"
            className="flex h-9 w-9 shrink-0 items-center justify-center border border-accent-lime bg-black font-mono text-sm font-extrabold uppercase text-accent-lime"
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
            <Button
              variant="ghost"
              size="icon"
              onClick={onLogout}
              aria-label="Sign out"
              className="shrink-0 text-soft hover:border-accent-lime hover:text-accent-lime"
            >
              <FaIcon icon={faRightFromBracket} />
            </Button>
          </Tooltip>
        </div>
      </div>
    </div>
  );
}
