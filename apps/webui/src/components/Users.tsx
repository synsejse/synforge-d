import { useEffect, useState, type ChangeEvent, type FormEvent } from "react";
import {
  faKey,
  faSave,
  faTrash,
  faUserPlus,
  faUsers,
} from "@fortawesome/free-solid-svg-icons";
import api from "../lib/api";
import { formatDateTime } from "../lib/datetime";
import type { CreateUserRequest, SessionResponse, UserPermission, UserResponse } from "../lib/types";
import FaIcon from "./FaIcon";
import PageHeader from "./PageHeader";

interface UserDraft {
  handle: string;
  display_name: string;
  permissions: UserPermission[];
  active: boolean;
}

const PERMISSIONS: UserPermission[] = ["read", "write", "repo"];

export default function Users() {
  const [users, setUsers] = useState<UserResponse[]>([]);
  const [drafts, setDrafts] = useState<Record<string, UserDraft>>({});
  const [passwords, setPasswords] = useState<Record<string, string>>({});
  const [currentUserId, setCurrentUserId] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [savingId, setSavingId] = useState<string | null>(null);
  const [passwordSavingId, setPasswordSavingId] = useState<string | null>(null);
  const [deletingId, setDeletingId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [createForm, setCreateForm] = useState<CreateUserRequest>({
    handle: "",
    display_name: "",
    password: "",
    permissions: ["read"],
    active: true,
  });

  useEffect(() => {
    void load();
  }, []);

  async function load() {
    try {
      setLoading(true);
      const [usersRes, sessionRes] = await Promise.all([
        api.listUsers(),
        api.getSession(),
      ]);
      setUsers(usersRes.users);
      setCurrentUserId((sessionRes as SessionResponse).user.id);
      setDrafts(
        Object.fromEntries(
          usersRes.users.map((entry) => [
            entry.user.id,
            {
              handle: entry.user.handle,
              display_name: entry.user.display_name,
              permissions: [...entry.user.permissions],
              active: entry.user.active,
            },
          ])
        )
      );
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to load users");
    } finally {
      setLoading(false);
    }
  }

  function setDraftField(id: string, field: keyof UserDraft, value: string | boolean | UserPermission[]) {
    setDrafts((current) => ({
      ...current,
      [id]: {
        ...current[id],
        [field]: value,
      },
    }));
  }

  function toggleDraftPermission(id: string, permission: UserPermission) {
    const current = drafts[id];
    if (!current) {
      return;
    }
    const permissions = current.permissions.includes(permission)
      ? current.permissions.filter((value) => value !== permission)
      : [...current.permissions, permission];
    setDraftField(id, "permissions", permissions);
  }

  function toggleCreatePermission(permission: UserPermission) {
    setCreateForm((current) => ({
      ...current,
      permissions: current.permissions.includes(permission)
        ? current.permissions.filter((value) => value !== permission)
        : [...current.permissions, permission],
    }));
  }

  async function handleCreate(event: FormEvent) {
    event.preventDefault();
    setSavingId("create");
    try {
      await api.createUser(createForm);
      setCreateForm({
        handle: "",
        display_name: "",
        password: "",
        permissions: ["read"],
        active: true,
      });
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to create user");
    } finally {
      setSavingId(null);
    }
  }

  async function handleSave(id: string) {
    const draft = drafts[id];
    if (!draft) {
      return;
    }
    setSavingId(id);
    try {
      await api.updateUser(id, draft);
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to update user");
    } finally {
      setSavingId(null);
    }
  }

  async function handlePasswordChange(id: string) {
    const password = passwords[id]?.trim();
    if (!password) {
      setError("Password must not be empty");
      return;
    }
    setPasswordSavingId(id);
    try {
      await api.changeUserPassword(id, { password });
      setPasswords((current) => ({ ...current, [id]: "" }));
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to change password");
    } finally {
      setPasswordSavingId(null);
    }
  }

  async function handleDelete(id: string) {
    if (!window.confirm("Delete this user?")) {
      return;
    }
    setDeletingId(id);
    try {
      await api.deleteUser(id);
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to delete user");
    } finally {
      setDeletingId(null);
    }
  }

  if (loading) {
    return <div className="text-zinc-400">Loading users…</div>;
  }

  return (
    <div className="space-y-8">
      <PageHeader
        eyebrow="Access Control"
        title="Users"
        description="Manage accounts, permissions, passwords, and repository download usage."
        actions={[{ href: "/settings/", label: "Settings", icon: faUsers }]}
      />

      {error ? (
        <div className="border border-zinc-800 bg-black p-4 text-zinc-200">Error: {error}</div>
      ) : null}

      <section className="border border-zinc-800 bg-black p-6">
        <div className="mb-6">
          <h2 className="text-xl font-semibold text-white">Create User</h2>
          <p className="mt-2 text-sm text-zinc-400">New users can be scoped for UI read, UI write, and repo access independently.</p>
        </div>
        <form onSubmit={handleCreate} className="grid gap-4 xl:grid-cols-2">
          <TextField
            label="Handle"
            value={createForm.handle}
            onChange={(event) => setCreateForm((current) => ({ ...current, handle: event.target.value }))}
          />
          <TextField
            label="Display name"
            value={createForm.display_name}
            onChange={(event) => setCreateForm((current) => ({ ...current, display_name: event.target.value }))}
          />
          <TextField
            label="Password"
            value={createForm.password}
            type="password"
            onChange={(event) => setCreateForm((current) => ({ ...current, password: event.target.value }))}
          />
          <label className="flex items-center gap-3 border border-zinc-800 bg-black px-4 py-3 text-sm text-zinc-200">
            <input
              type="checkbox"
              checked={createForm.active}
              onChange={(event) => setCreateForm((current) => ({ ...current, active: event.target.checked }))}
            />
            Active
          </label>
          <div className="xl:col-span-2">
            <PermissionGroup permissions={createForm.permissions} onToggle={toggleCreatePermission} />
          </div>
          <div className="xl:col-span-2 flex justify-end">
            <button
              type="submit"
              disabled={savingId === "create"}
              className="border border-zinc-200 bg-zinc-100 px-5 py-3 text-sm font-semibold text-black transition hover:bg-white disabled:opacity-70"
            >
              <FaIcon icon={faUserPlus} className="mr-2" />
              {savingId === "create" ? "Creating…" : "Create user"}
            </button>
          </div>
        </form>
      </section>

      <section className="space-y-4">
        {users.map((entry) => {
          const draft = drafts[entry.user.id];
          if (!draft) {
            return null;
          }
          const isCurrentUser = currentUserId === entry.user.id;
          return (
            <article key={entry.user.id} className="border border-zinc-800 bg-black p-6">
              <div className="grid gap-6 xl:grid-cols-[minmax(0,1fr)_320px]">
                <div className="space-y-4">
                  <div>
                    <h2 className="text-xl font-semibold text-white">{entry.user.display_name}</h2>
                    <p className="mt-1 font-mono text-sm text-zinc-400">{entry.user.handle}</p>
                  </div>
                  <div className="grid gap-4 xl:grid-cols-2">
                    <TextField
                      label="Handle"
                      value={draft.handle}
                      onChange={(event) => setDraftField(entry.user.id, "handle", event.target.value)}
                    />
                    <TextField
                      label="Display name"
                      value={draft.display_name}
                      onChange={(event) => setDraftField(entry.user.id, "display_name", event.target.value)}
                    />
                  </div>
                  <PermissionGroup
                    permissions={draft.permissions}
                    onToggle={(permission) => toggleDraftPermission(entry.user.id, permission)}
                  />
                  <label className="flex items-center gap-3 border border-zinc-800 bg-black px-4 py-3 text-sm text-zinc-200">
                    <input
                      type="checkbox"
                      checked={draft.active}
                      onChange={(event) => setDraftField(entry.user.id, "active", event.target.checked)}
                    />
                    Active
                  </label>
                  <div className="flex flex-wrap gap-3">
                    <button
                      type="button"
                      onClick={() => void handleSave(entry.user.id)}
                      disabled={savingId === entry.user.id}
                      className="border border-zinc-200 bg-zinc-100 px-4 py-2 text-sm font-semibold text-black transition hover:bg-white disabled:opacity-70"
                    >
                      <FaIcon icon={faSave} className="mr-2" />
                      {savingId === entry.user.id ? "Saving…" : "Save changes"}
                    </button>
                    <button
                      type="button"
                      onClick={() => void handleDelete(entry.user.id)}
                      disabled={deletingId === entry.user.id || isCurrentUser}
                      className="border border-zinc-800 bg-black px-4 py-2 text-sm font-semibold text-zinc-200 transition hover:border-zinc-600 hover:bg-zinc-950 disabled:opacity-50"
                    >
                      <FaIcon icon={faTrash} className="mr-2" />
                      {deletingId === entry.user.id ? "Deleting…" : isCurrentUser ? "Current user" : "Delete"}
                    </button>
                  </div>
                </div>

                <div className="space-y-4 border border-zinc-800 bg-zinc-950 p-4">
                  <div>
                    <div className="text-xs uppercase tracking-[0.18em] text-zinc-500">Repo Usage</div>
                    <div className="mt-2 text-2xl font-semibold text-white">
                      {formatBytes(entry.metrics.downloaded_bytes)}
                    </div>
                    <div className="mt-1 text-sm text-zinc-400">
                      Updated {formatDateTime(entry.metrics.updated_at)}
                    </div>
                  </div>
                  <div>
                    <div className="text-xs uppercase tracking-[0.18em] text-zinc-500">Created</div>
                    <div className="mt-2 text-sm text-zinc-300">
                      {formatDateTime(entry.user.created_at)}
                    </div>
                  </div>
                  <div>
                    <div className="text-xs uppercase tracking-[0.18em] text-zinc-500">Change Password</div>
                    <div className="mt-3 space-y-3">
                      <input
                        type="password"
                        value={passwords[entry.user.id] || ""}
                        onChange={(event) =>
                          setPasswords((current) => ({
                            ...current,
                            [entry.user.id]: event.target.value,
                          }))
                        }
                        className="w-full border border-zinc-800 bg-black px-4 py-3 text-sm text-white outline-none transition focus:border-zinc-600"
                        placeholder="New password"
                      />
                      <button
                        type="button"
                        onClick={() => void handlePasswordChange(entry.user.id)}
                        disabled={passwordSavingId === entry.user.id}
                        className="w-full border border-zinc-800 bg-black px-4 py-2 text-sm font-semibold text-zinc-200 transition hover:border-zinc-600 hover:bg-black disabled:opacity-70"
                      >
                        <FaIcon icon={faKey} className="mr-2" />
                        {passwordSavingId === entry.user.id ? "Updating…" : "Update password"}
                      </button>
                    </div>
                  </div>
                </div>
              </div>
            </article>
          );
        })}
      </section>
    </div>
  );
}

function TextField({
  label,
  value,
  onChange,
  type = "text",
}: {
  label: string;
  value: string;
  onChange: (event: ChangeEvent<HTMLInputElement>) => void;
  type?: string;
}) {
  return (
    <label className="block">
      <span className="mb-2 block text-sm font-medium text-zinc-300">{label}</span>
      <input
        type={type}
        value={value}
        onChange={onChange}
        className="w-full border border-zinc-800 bg-black px-4 py-3 text-sm text-white outline-none transition focus:border-zinc-600"
      />
    </label>
  );
}

function PermissionGroup({
  permissions,
  onToggle,
}: {
  permissions: UserPermission[];
  onToggle: (permission: UserPermission) => void;
}) {
  return (
    <div className="space-y-2">
      <div className="text-sm font-medium text-zinc-300">Permissions</div>
      <div className="flex flex-wrap gap-3">
        {PERMISSIONS.map((permission) => (
          <label
            key={permission}
            className="flex items-center gap-3 border border-zinc-800 bg-black px-4 py-3 text-sm text-zinc-200"
          >
            <input
              type="checkbox"
              checked={permissions.includes(permission)}
              onChange={() => onToggle(permission)}
            />
            {permission}
          </label>
        ))}
      </div>
    </div>
  );
}

function formatBytes(bytes: number) {
  if (bytes < 1024) {
    return `${bytes} B`;
  }
  const units = ["KiB", "MiB", "GiB", "TiB"];
  let value = bytes;
  let index = -1;
  while (value >= 1024 && index < units.length - 1) {
    value /= 1024;
    index += 1;
  }
  return `${value.toFixed(value >= 10 ? 1 : 2)} ${units[index]}`;
}
