import { Link } from "@tanstack/react-router";
import type { IconDefinition } from "@fortawesome/fontawesome-svg-core";
import { faArrowUpRightFromSquare } from "@fortawesome/free-solid-svg-icons";
import FaIcon from "./fa-icon";
import Tooltip from "./tooltip";
import { cn } from "../../lib/utils";

export interface NavItem {
  href: string;
  label: string;
  icon: IconDefinition;
  description: string;
  /** When true, render as a regular anchor that opens in a new tab. */
  external?: boolean;
  /** Optional badge — small bracketed count or short text on the right. */
  badge?: number | string | null;
  /** Visual treatment of the badge. Defaults to neutral. */
  badgeTone?: "lime" | "orange" | "neutral";
}

export interface NavGroup {
  /** Section label shown in expanded mode; hidden visually in rail mode. */
  label: string;
  items: NavItem[];
}

interface NavigationProps {
  groups: NavGroup[];
  /** External / reference items pinned at the bottom of the nav. */
  external?: NavItem[];
  onNavigate?: () => void;
  /** When true, render icon-only on desktop (lg+). Mobile is unaffected. */
  rail?: boolean;
}

export default function Navigation({
  groups,
  external,
  onNavigate,
  rail = false,
}: NavigationProps) {
  const sections: NavGroup[] = external && external.length > 0
    ? [...groups, { label: "Reference", items: external }]
    : groups;

  return (
    <nav
      className="border-y border-edge lg:flex-1 lg:border-y-0 lg:overflow-auto"
      aria-label="Primary navigation"
    >
      <div
        className={cn(
          "max-h-[52dvh] overflow-y-auto py-2 lg:max-h-none lg:block lg:overflow-visible lg:py-3",
          rail ? "px-2 lg:px-1.5" : "px-2 lg:px-2",
        )}
      >
        {sections.map((group, idx) => (
          <NavSection
            key={group.label}
            group={group}
            rail={rail}
            onNavigate={onNavigate}
            withDivider={idx > 0}
          />
        ))}
      </div>
    </nav>
  );
}

interface NavSectionProps {
  group: NavGroup;
  rail: boolean;
  onNavigate?: () => void;
  withDivider: boolean;
}

function NavSection({ group, rail, onNavigate, withDivider }: NavSectionProps) {
  return (
    <div
      className={cn(
        withDivider && "mt-2 pt-2 lg:mt-3 lg:pt-3",
        withDivider && "border-t border-edge",
      )}
    >
      {!rail ? (
        <div className="px-3 pb-1.5 font-mono text-[10px] font-bold uppercase tracking-[0.28em] text-soft">
          {group.label}
        </div>
      ) : null}
      <ul className="space-y-1 lg:space-y-0.5">
        {group.items.map((item) => (
          <li key={item.href}>
            <NavLink item={item} onNavigate={onNavigate} rail={rail} />
          </li>
        ))}
      </ul>
    </div>
  );
}

interface NavLinkProps {
  item: NavItem;
  onNavigate?: () => void;
  rail: boolean;
}

function NavLink({ item, onNavigate, rail }: NavLinkProps) {
  if (item.external) {
    return (
      <ExternalNavLink item={item} onNavigate={onNavigate} rail={rail} />
    );
  }
  return <InternalNavLink item={item} onNavigate={onNavigate} rail={rail} />;
}

const linkBase =
  "group relative flex w-full items-center gap-3 border-l-[3px] border-transparent bg-transparent text-strong transition-colors hover:bg-surface-hover hover:text-white focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent-lime";
const linkExpanded = "px-3 py-2";
const linkRail = "lg:px-0 lg:py-2.5 lg:justify-center";
const linkActive = "border-accent-lime bg-surface-hover text-white";

function InternalNavLink({ item, onNavigate, rail }: NavLinkProps) {
  const link = (
    <Link
      to={item.href}
      onClick={onNavigate}
      activeOptions={{ exact: item.href === "/" }}
      className={cn(linkBase, linkExpanded, rail && linkRail)}
      activeProps={{
        "aria-current": "page",
        className: cn(linkBase, linkExpanded, rail && linkRail, linkActive),
      }}
    >
      {({ isActive }) => <NavLinkBody item={item} rail={rail} isActive={isActive} />}
    </Link>
  );

  if (!rail) return link;

  return (
    <Tooltip content={<RailTooltipContent item={item} />} side="right">
      {link}
    </Tooltip>
  );
}

function ExternalNavLink({ item, onNavigate, rail }: NavLinkProps) {
  const anchor = (
    <a
      href={item.href}
      target="_blank"
      rel="noopener noreferrer"
      onClick={onNavigate}
      className={cn(linkBase, linkExpanded, rail && linkRail)}
    >
      <NavLinkBody item={item} rail={rail} isActive={false} external />
    </a>
  );

  if (!rail) return anchor;

  return (
    <Tooltip content={<RailTooltipContent item={item} />} side="right">
      {anchor}
    </Tooltip>
  );
}

function NavLinkBody({
  item,
  rail,
  isActive,
  external = false,
}: {
  item: NavItem;
  rail: boolean;
  isActive: boolean;
  external?: boolean;
}) {
  return (
    <>
      <span
        className={cn(
          "flex h-8 w-8 shrink-0 items-center justify-center text-base transition-colors",
          isActive
            ? "text-accent-lime"
            : "text-soft group-hover:text-accent-lime",
        )}
        aria-hidden="true"
      >
        <FaIcon icon={item.icon} />
      </span>
      <span className={cn("min-w-0 flex-1 leading-tight", rail && "lg:hidden")}>
        <span
          className={cn(
            "flex items-center gap-1.5 font-display text-sm font-bold uppercase tracking-wide",
            isActive ? "text-white" : "text-strong",
          )}
        >
          <span className="truncate">{item.label}</span>
          {external ? (
            <FaIcon
              icon={faArrowUpRightFromSquare}
              className="text-[0.7em] text-soft"
            />
          ) : null}
          {item.badge != null && item.badge !== 0 && item.badge !== "" ? (
            <NavBadge value={item.badge} tone={item.badgeTone ?? "neutral"} />
          ) : null}
        </span>
        <span className="block truncate font-mono text-[11px] text-soft">
          {item.description}
        </span>
      </span>
      {/* Rail mode: badge as a tiny dot in the corner of the icon. */}
      {rail && item.badge != null && item.badge !== 0 && item.badge !== "" ? (
        <span
          aria-hidden="true"
          className={cn(
            "lg:absolute lg:top-1.5 lg:right-1.5 hidden lg:block h-2 w-2",
            item.badgeTone === "orange"
              ? "bg-accent-orange"
              : item.badgeTone === "lime"
                ? "bg-accent-lime"
                : "bg-muted",
          )}
        />
      ) : null}
    </>
  );
}

function NavBadge({
  value,
  tone,
}: {
  value: number | string;
  tone: "lime" | "orange" | "neutral";
}) {
  const toneClass =
    tone === "orange"
      ? "border-accent-orange bg-accent-orange text-black"
      : tone === "lime"
        ? "border-accent-lime bg-accent-lime text-black"
        : "border-edge-strong bg-black text-soft";
  return (
    <span
      className={cn(
        // inline-flex + leading-none + items-center keeps the digit
        // optically centered inside the filled chip; tracking is
        // dropped because letter-spacing pads only the right side and
        // shoves single-character badges off-center.
        "ml-auto inline-flex h-5 min-w-[1.25rem] shrink-0 items-center justify-center border px-1 font-mono text-[10px] font-extrabold leading-none",
        toneClass,
      )}
    >
      {value}
    </span>
  );
}

function RailTooltipContent({ item }: { item: NavItem }) {
  return (
    <div className="flex flex-col gap-0.5">
      <span className="font-display text-xs font-bold uppercase tracking-[0.12em] text-white">
        {item.label}
      </span>
      <span className="font-mono text-[10px] normal-case tracking-normal text-soft">
        {item.description}
      </span>
    </div>
  );
}
