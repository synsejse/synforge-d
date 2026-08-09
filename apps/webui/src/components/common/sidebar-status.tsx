import { useEffect, useState } from "react";
import { faRightFromBracket } from "@fortawesome/free-solid-svg-icons";
import Button from "../ui/button";
import FaIcon from "../ui/fa-icon";
import Tooltip from "../ui/tooltip";
import { cn } from "../../lib/utils";

interface SidebarSystemTickProps {
  isRail: boolean;
  activeJobCount: number;
  live: boolean;
}

export function SidebarSystemTick({
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
      <div className="hidden border-t border-edge bg-black lg:block">
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
                isBuilding ? "animate-pulse bg-accent-lime" : "bg-soft",
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
              isBuilding ? "animate-pulse bg-accent-lime" : "bg-soft",
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

interface SidebarFooterProps {
  isRail: boolean;
  userInitial: string;
  displayName: string | null;
  handle: string | null;
  onLogout: () => void;
}

export function SidebarFooter({
  isRail,
  userInitial,
  displayName,
  handle,
  onLogout,
}: SidebarFooterProps) {
  if (isRail) {
    return (
      <div className="border-t border-edge-strong bg-black lg:mt-auto">
        <div className="hidden flex-col items-center gap-2 px-2 py-3 lg:flex">
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
            <div className="font-display truncate text-sm font-bold uppercase tracking-wide text-white">
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
