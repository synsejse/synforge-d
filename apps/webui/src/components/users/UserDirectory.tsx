import {
  faKey,
  faPen,
  faTrash,
  faUsers,
} from "@fortawesome/free-solid-svg-icons";
import { formatBytes } from "../../lib/bytes";
import { formatDateTime } from "../../lib/datetime";
import type { UserResponse } from "../../lib/types";
import ActionButton from "../ui/ActionButton";
import FaIcon from "../ui/FaIcon";

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
    <section className="overflow-hidden border border-zinc-800 bg-black">
      <div className="border-b border-zinc-800 px-6 py-4">
        <div className="flex items-center gap-3">
          <div className="flex h-11 w-11 items-center justify-center border border-zinc-800 bg-zinc-950 text-white">
            <FaIcon icon={faUsers} />
          </div>
          <div>
            <h2 className="text-xl font-semibold text-white">User Directory</h2>
            <p className="mt-1 text-sm text-zinc-400">
              Handles, permissions, and repository traffic at a glance.
            </p>
          </div>
        </div>
      </div>

      <div className="divide-y divide-white/8">
        {users.map((entry) => {
          const isCurrentUser = currentUserId === entry.user.id;
          return (
            <article key={entry.user.id} className="flex flex-col gap-5 px-6 py-5">
              <div className="grid gap-5 xl:grid-cols-[minmax(0,1fr)_minmax(340px,420px)] xl:items-start">
                <div className="min-w-0">
                  <div className="flex flex-wrap items-center gap-3">
                    <h3 className="text-xl font-semibold text-white">
                      {entry.user.display_name}
                    </h3>
                    <span className="border border-zinc-800 bg-zinc-950 px-2.5 py-1 text-xs uppercase tracking-[0.18em] text-zinc-400">
                      {entry.user.active ? "active" : "disabled"}
                    </span>
                    {isCurrentUser ? (
                      <span className="border border-zinc-800 bg-zinc-950 px-2.5 py-1 text-xs uppercase tracking-[0.18em] text-zinc-400">
                        current
                      </span>
                    ) : null}
                  </div>
                  <div className="mt-2 font-mono text-sm text-zinc-400">
                    @{entry.user.handle}
                  </div>
                </div>

                <div className="flex flex-wrap justify-end gap-2">
                  <ActionButton
                    onClick={() => onEdit(entry)}
                    icon={faPen}
                    className="py-2 text-sm"
                  >
                    Edit
                  </ActionButton>
                  <ActionButton
                    onClick={() => onPassword(entry)}
                    icon={faKey}
                    className="py-2 text-sm"
                  >
                    Password
                  </ActionButton>
                  <ActionButton
                    onClick={() => onDelete(entry)}
                    disabled={isCurrentUser}
                    icon={faTrash}
                    className="py-2 text-sm disabled:cursor-not-allowed disabled:opacity-50"
                    aria-label={`Delete user ${entry.user.handle}`}
                  >
                    Delete
                  </ActionButton>
                </div>
              </div>

              <div className="flex flex-wrap items-center justify-between gap-4 border-t border-zinc-800 pt-4">
                <div className="flex flex-wrap gap-2">
                  {entry.user.permissions.map((permission) => (
                    <span
                      key={`${entry.user.id}:${permission}`}
                      className="border border-zinc-800 bg-zinc-950 px-3 py-1 text-xs uppercase tracking-[0.18em] text-zinc-300"
                    >
                      {permission}
                    </span>
                  ))}
                </div>
                <dl className="flex flex-wrap items-center gap-x-5 gap-y-2 text-sm text-zinc-400">
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
      <dt className="self-center text-[10px] uppercase tracking-[0.18em] text-zinc-500">
        {label}
      </dt>
      <dd className="self-center text-sm font-medium text-zinc-200">{value}</dd>
    </div>
  );
}
