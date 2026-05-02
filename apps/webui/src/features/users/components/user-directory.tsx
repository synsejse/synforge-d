import {
  faKey,
  faPen,
  faTrash,
  faUsers,
} from "@fortawesome/free-solid-svg-icons";
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
    <section className="overflow-hidden border-2 border-edge-strong bg-black">
      <div className="border-b-2 border-edge bg-surface-alt px-6 py-5">
        <div className="flex items-center gap-3">
          <div className="flex h-10 w-10 items-center justify-center border-2 border-edge-strong bg-black text-white">
            <FaIcon icon={faUsers} />
          </div>
          <div>
            <h2 className="text-lg font-bold text-white">User directory</h2>
            <p className="mt-1 text-sm text-soft">
              Handles, permissions, and repository traffic at a glance.
            </p>
          </div>
        </div>
      </div>

      <div className="divide-y divide-edge">
        {users.map((entry) => {
          const isCurrentUser = currentUserId === entry.user.id;
          return (
            <article key={entry.user.id} className="flex flex-col gap-5 bg-black px-6 py-5">
              <div className="flex flex-wrap items-start justify-between gap-4">
                <div className="min-w-0">
                  <div className="flex flex-wrap items-center gap-3">
                    <h3 className="text-lg font-semibold text-white">
                      {entry.user.display_name}
                    </h3>
                    <span className="border-2 border-edge-strong bg-surface-alt px-2.5 py-1 text-xs uppercase tracking-[0.18em] text-muted">
                      {entry.user.active ? "active" : "disabled"}
                    </span>
                    {isCurrentUser ? (
                      <span className="border-2 border-accent-lime bg-surface-alt px-2.5 py-1 text-xs uppercase tracking-[0.18em] text-accent-lime">
                        current
                      </span>
                    ) : null}
                  </div>
                  <div className="mt-2 font-mono text-sm text-muted">
                    @{entry.user.handle}
                  </div>
                </div>

                <div className="flex shrink-0 justify-end gap-1">
                  <Tooltip content="Edit user" side="top">
                    <Button
                      variant="ghost"
                      size="icon-sm"
                      onClick={() => onEdit(entry)}
                      aria-label={`Edit user ${entry.user.handle}`}
                    >
                      <FaIcon icon={faPen} />
                    </Button>
                  </Tooltip>
                  <Tooltip content="Set password" side="top">
                    <Button
                      variant="ghost"
                      size="icon-sm"
                      onClick={() => onPassword(entry)}
                      aria-label={`Set password for ${entry.user.handle}`}
                    >
                      <FaIcon icon={faKey} />
                    </Button>
                  </Tooltip>
                  <Tooltip content="Delete user" side="top">
                    <Button
                      variant="ghost"
                      size="icon-sm"
                      onClick={() => onDelete(entry)}
                      disabled={isCurrentUser}
                      aria-label={`Delete user ${entry.user.handle}`}
                      className="hover:border-error hover:text-error"
                    >
                      <FaIcon icon={faTrash} />
                    </Button>
                  </Tooltip>
                </div>
              </div>

              <div className="flex flex-wrap items-center justify-between gap-4 border-t-2 border-edge pt-4">
                <div className="flex flex-wrap gap-2">
                  {entry.user.permissions.map((permission) => (
                    <span
                      key={`${entry.user.id}:${permission}`}
                      className="border-2 border-edge-strong bg-surface-alt px-3 py-1 text-xs uppercase tracking-[0.18em] text-muted"
                    >
                      {permission}
                    </span>
                  ))}
                </div>
                <dl className="flex flex-wrap items-center gap-x-5 gap-y-2 text-sm text-muted">
                  <CompactMetric
                    label="Repo Usage"
                    value={formatBytes(entry.metrics.downloaded_bytes)}
                  />
                  <CompactMetric
                    label="Created"
                    value={formatDateTime(entry.user.created_at)}
                  />
                  <CompactMetric
                    label="Updated"
                    value={formatDateTime(entry.user.updated_at)}
                  />
                </dl>
              </div>
            </article>
          );
        })}
      </div>
    </section>
  );
}

function CompactMetric({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex items-center gap-2 leading-none">
      <dt className="self-center font-mono text-[10px] uppercase tracking-[0.18em] text-soft">
        {label}
      </dt>
      <dd className="self-center font-mono text-sm font-medium text-strong">{value}</dd>
    </div>
  );
}
