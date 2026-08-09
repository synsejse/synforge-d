import { useMemo } from "react";
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
  faSliders,
  faUsers,
  faXmark,
} from "@fortawesome/free-solid-svg-icons";
import Navigation, { type NavGroup, type NavItem } from "../ui/navigation";
import Button from "../ui/button";
import FaIcon from "../ui/fa-icon";
import Tooltip from "../ui/tooltip";
import { cn } from "../../lib/utils";
import BrandMark from "./brand-mark";
import { SidebarFooter, SidebarSystemTick } from "./sidebar-status";

function buildNavGroups(activeJobCount: number): NavGroup[] {
  return [
    {
      label: "Operations",
      items: [
        {
          href: "/",
          label: "Overview",
          icon: faGaugeHigh,
          description: "System summary",
        },
        {
          href: "/jobs",
          label: "Jobs",
          icon: faChartLine,
          description: "Runs and live traces",
          badge: activeJobCount > 0 ? activeJobCount : null,
          badgeTone: "lime",
        },
        {
          href: "/statistics",
          label: "Statistics",
          icon: faChartSimple,
          description: "Telemetry and cache",
        },
      ],
    },
    {
      label: "Build",
      items: [
        {
          href: "/packages",
          label: "Packages",
          icon: faBoxesStacked,
          description: "Sources and history",
        },
        {
          href: "/repository",
          label: "Repository",
          icon: faFolderTree,
          description: "Published files",
        },
        {
          href: "/signing",
          label: "Signing",
          icon: faKey,
          description: "GPG metadata signing",
        },
      ],
    },
    {
      label: "Admin",
      items: [
        {
          href: "/users",
          label: "Users",
          icon: faUsers,
          description: "Accounts and permissions",
        },
        {
          href: "/settings",
          label: "Settings",
          icon: faSliders,
          description: "Daemon config",
        },
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

interface Props {
  isDesktop: boolean;
  isRail: boolean;
  mobileNavOpen: boolean;
  activeJobCount: number;
  live: boolean;
  userInitial: string;
  displayName: string | null;
  handle: string | null;
  onMobileToggle: () => void;
  onRailToggle: () => void;
  onLogout: () => void;
}

export default function AppSidebar({
  isDesktop,
  isRail,
  mobileNavOpen,
  activeJobCount,
  live,
  userInitial,
  displayName,
  handle,
  onMobileToggle,
  onRailToggle,
  onLogout,
}: Props) {
  const navGroups = useMemo(
    () => buildNavGroups(activeJobCount),
    [activeJobCount],
  );
  const showNav = isDesktop || mobileNavOpen;

  return (
    <aside className="app-section-band-vertical flex min-w-0 flex-col border border-edge-strong p-0 lg:min-h-0">
      <SidebarHeader
        isRail={isRail}
        mobileNavOpen={mobileNavOpen}
        onMobileToggle={onMobileToggle}
        onRailToggle={onRailToggle}
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
            if (!isDesktop && mobileNavOpen) onMobileToggle();
          }}
        />
        <SidebarSystemTick
          isRail={isRail}
          activeJobCount={activeJobCount}
          live={live}
        />
        <SidebarFooter
          isRail={isRail}
          userInitial={userInitial}
          displayName={displayName}
          handle={handle}
          onLogout={onLogout}
        />
      </div>
    </aside>
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
        isRail
          ? "px-2 py-3 lg:px-2"
          : "px-4 py-4 sm:px-5 sm:py-5 lg:px-4 lg:py-4",
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
            className="hidden shrink-0 lg:inline-flex"
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
          className="h-10 w-10 shrink-0 lg:hidden"
        >
          <FaIcon icon={mobileNavOpen ? faXmark : faBars} />
        </Button>
      </div>
    </div>
  );
}
