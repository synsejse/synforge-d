import { useEffect, useMemo, useState, type ReactNode } from "react";
import { useNavigate } from "@tanstack/react-router";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import api from "../../lib/api";
import { dashboardQueries } from "../../lib/queries";
import AppSidebar from "./app-sidebar";
import { useSession } from "./session-provider";
import { usePageVisible } from "./page-visibility-context";
import { KeyboardShortcutsProvider } from "./keyboard-shortcuts";
import SystemNoticeBar from "./system-notice-bar";

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

  const isRail = isDesktop && railCollapsed;

  const toggleRail = () => {
    setRailCollapsed((previous) => {
      const next = !previous;
      writeRailPreference(next);
      return next;
    });
  };

  const handleLogout = async () => {
    try {
      await api.logout();
    } catch {
      // The server may already consider the session logged out.
    }
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

  const pageVisible = usePageVisible();
  const overviewQuery = useQuery({
    ...dashboardQueries.overview(),
    refetchInterval: pageVisible ? 15_000 : false,
  });
  const activeJobCount = overviewQuery.data?.activeJobCount ?? 0;
  const gridCols = isRail
    ? "lg:grid-cols-[64px_minmax(0,1fr)]"
    : "lg:grid-cols-[240px_minmax(0,1fr)] xl:grid-cols-[280px_minmax(0,1fr)]";

  return (
    <KeyboardShortcutsProvider>
      <SystemNoticeBar />
      <div className="box-border min-h-full w-full max-w-full px-2 py-2 sm:px-3 sm:py-3 lg:h-screen lg:overflow-hidden lg:px-5 lg:py-5">
        <div className={`grid min-h-full min-w-0 gap-3 lg:h-full ${gridCols}`}>
          <AppSidebar
            isDesktop={isDesktop}
            isRail={isRail}
            mobileNavOpen={mobileNavOpen}
            activeJobCount={activeJobCount}
            live={pageVisible}
            userInitial={userInitial}
            displayName={session?.user?.display_name ?? null}
            handle={session?.user?.handle ?? null}
            onMobileToggle={() => setMobileNavOpen((open) => !open)}
            onRailToggle={toggleRail}
            onLogout={handleLogout}
          />

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
