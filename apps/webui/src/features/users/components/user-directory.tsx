import {
  faKey,
  faPen,
  faTrash,
  faUsers,
} from "@fortawesome/free-solid-svg-icons";
import type { IconDefinition } from "@fortawesome/fontawesome-svg-core";
import { formatBytes } from "../../../lib/bytes";
import { formatDateTime } from "../../../lib/datetime";
import type { UserResponse } from "../../../lib/types";
import Button from "../../../components/ui/button";
import FaIcon from "../../../components/ui/fa-icon";
import Tooltip from "../../../components/ui/tooltip";

interface UserDirectoryProps {
  users: UserResponse[];
  currentUserId: string | null;
  onEdit: (user: UserResponse) => void;
  onPassword: (user: UserResponse) => void;
  onDelete: (user: UserResponse) => void;
}

export default function UserDirectory({
  users,
  currentUserId,
  onEdit,
  onPassword,
  onDelete,
}: UserDirectoryProps) {
  return (
    <div className="space-y-3">
      <div className="flex items-center gap-3 border border-edge bg-[#09090b] px-[18px] py-4">
        <div className="flex h-8 w-8 shrink-0 items-center justify-center border border-edge text-soft">
          <FaIcon icon={faUsers} className="text-[15px]" />
        </div>
        <div>
          <h2 className="font-mono text-[13px] font-bold tracking-[0.04em] text-white">
            User directory
          </h2>
          <p className="font-body mt-1.5 text-xs text-[#71717a]">
            Handles, permissions, and repository traffic at a glance.
          </p>
        </div>
      </div>

      {users.map((entry) => {
        const isCurrentUser = currentUserId === entry.user.id;
        const initial =
          entry.user.display_name.trim().charAt(0).toUpperCase() || "?";
        return (
          <article
            key={entry.user.id}
            className="sf-row border border-edge bg-black px-[18px] py-4 transition-colors hover:border-edge-strong hover:bg-[#0c0c0d]"
          >
            <div className="flex flex-wrap items-center gap-3">
              <div
                aria-hidden="true"
                className="flex h-[38px] w-[38px] shrink-0 items-center justify-center border border-edge bg-black font-mono text-[15px] font-extrabold text-muted"
              >
                {initial}
              </div>
              <div className="leading-tight">
                <div className="flex flex-wrap items-center gap-2.5">
                  <span className="font-mono text-[15px] font-bold text-white">
                    {entry.user.display_name}
                  </span>
                  <span className="border border-edge px-[7px] py-[3px] font-mono text-[9px] font-semibold uppercase leading-none tracking-[0.08em] text-[#71717a]">
                    {entry.user.active ? "Active" : "Disabled"}
                  </span>
                  {isCurrentUser ? (
                    <span className="border border-accent-lime px-[7px] py-[3px] font-mono text-[9px] font-bold uppercase leading-none tracking-[0.08em] text-accent-lime">
                      Current
                    </span>
                  ) : null}
                </div>
                <div className="mt-1.5 font-mono text-xs text-[#8b8b95]">
                  @{entry.user.handle}
                </div>
              </div>

              <div className="ml-auto flex gap-1.5">
                <IconButton
                  icon={faPen}
                  label={`Edit user ${entry.user.handle}`}
                  tooltip="Edit user"
                  onClick={() => onEdit(entry)}
                />
                <IconButton
                  icon={faKey}
                  label={`Set password for ${entry.user.handle}`}
                  tooltip="Set password"
                  onClick={() => onPassword(entry)}
                />
                <IconButton
                  icon={faTrash}
                  label={`Delete user ${entry.user.handle}`}
                  tooltip="Delete user"
                  onClick={() => onDelete(entry)}
                  disabled={isCurrentUser}
                  danger
                />
              </div>
            </div>

            <div className="mt-3.5 flex flex-wrap items-center gap-6 border-t border-[#161618] pt-3.5">
              <div className="flex flex-wrap gap-1.5">
                {entry.user.permissions.map((permission) => (
                  <span
                    key={`${entry.user.id}:${permission}`}
                    className="border border-edge px-[9px] py-[5px] font-mono text-[9px] font-semibold uppercase leading-none tracking-[0.08em] text-muted"
                  >
                    {permission}
                  </span>
                ))}
              </div>
              <div className="ml-auto flex flex-wrap gap-6">
                <Metric
                  label="Repo usage"
                  value={formatBytes(entry.metrics.downloaded_bytes)}
                  strong
                />
                <Metric label="Created" value={formatDateTime(entry.user.created_at)} />
                <Metric label="Updated" value={formatDateTime(entry.user.updated_at)} />
              </div>
            </div>
          </article>
        );
      })}
    </div>
  );
}

function Metric({
  label,
  value,
  strong,
}: {
  label: string;
  value: string;
  strong?: boolean;
}) {
  return (
    <div>
      <span className="font-mono text-[9px] font-semibold uppercase tracking-[0.14em] text-[#6b6b73]">
        {label}{" "}
      </span>
      <span
        className={`font-mono text-[11px] ${strong ? "font-bold text-strong" : "font-medium text-[#8b8b95]"}`}
      >
        {value}
      </span>
    </div>
  );
}

function IconButton({
  icon,
  label,
  tooltip,
  onClick,
  disabled,
  danger,
}: {
  icon: IconDefinition;
  label: string;
  tooltip: string;
  onClick: () => void;
  disabled?: boolean;
  danger?: boolean;
}) {
  return (
    <Tooltip content={tooltip} side="top">
      <Button
        variant="ghost"
        size="icon-sm"
        onClick={onClick}
        disabled={disabled}
        aria-label={label}
        className={`h-[30px] w-[30px] border-edge text-soft ${
          danger
            ? "hover:border-error hover:text-error"
            : "hover:border-accent-lime hover:text-accent-lime"
        }`}
      >
        <FaIcon icon={icon} className="text-[13px]" />
      </Button>
    </Tooltip>
  );
}
