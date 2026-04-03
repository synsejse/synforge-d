import {
  useEffect,
  useId,
  useRef,
  useState,
  type FormEvent,
  type ReactNode,
} from "react";
import {
  faKey,
  faPen,
  faPlus,
  faTrash,
  faUserPlus,
  faUsers,
} from "@fortawesome/free-solid-svg-icons";
import api from "../lib/api";
import { formatDateTime } from "../lib/datetime";
import type {
  CreateUserRequest,
  SessionResponse,
  UserPermission,
  UserResponse,
} from "../lib/types";
import EmptyState from "./EmptyState";
import FaIcon from "./FaIcon";
import LoadingBlock from "./LoadingBlock";
import PageHeader from "./PageHeader";

type ModalState =
  | { type: "create" }
  | { type: "edit"; user: UserResponse }
  | { type: "password"; user: UserResponse }
  | { type: "delete"; user: UserResponse }
  | null;

interface UserDraft {
  handle: string;
  display_name: string;
  permissions: UserPermission[];
  active: boolean;
}

const PERMISSIONS: UserPermission[] = ["read", "write", "repo"];

export default function Users() {
  const [users, setUsers] = useState<UserResponse[]>([]);
  const [currentUserId, setCurrentUserId] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [modal, setModal] = useState<ModalState>(null);
  const [createForm, setCreateForm] =
    useState<CreateUserRequest>(emptyCreateForm());
  const [editForm, setEditForm] = useState<UserDraft | null>(null);
  const [password, setPassword] = useState("");
  const lastFocusedRef = useRef<HTMLElement | null>(null);

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
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to load users");
    } finally {
      setLoading(false);
    }
  }

  function openCreateModal() {
    lastFocusedRef.current =
      document.activeElement instanceof HTMLElement
        ? document.activeElement
        : null;
    setCreateForm(emptyCreateForm());
    setModal({ type: "create" });
  }

  function openEditModal(user: UserResponse) {
    lastFocusedRef.current =
      document.activeElement instanceof HTMLElement
        ? document.activeElement
        : null;
    setEditForm({
      handle: user.user.handle,
      display_name: user.user.display_name,
      permissions: [...user.user.permissions],
      active: user.user.active,
    });
    setModal({ type: "edit", user });
  }

  function openPasswordModal(user: UserResponse) {
    lastFocusedRef.current =
      document.activeElement instanceof HTMLElement
        ? document.activeElement
        : null;
    setPassword("");
    setModal({ type: "password", user });
  }

  function openDeleteModal(user: UserResponse) {
    lastFocusedRef.current =
      document.activeElement instanceof HTMLElement
        ? document.activeElement
        : null;
    setModal({ type: "delete", user });
  }

  function closeModal() {
    if (submitting) {
      return;
    }
    setModal(null);
    setEditForm(null);
    setPassword("");
    lastFocusedRef.current?.focus();
  }

  useEffect(() => {
    if (!modal) {
      return;
    }
    function handleKeyDown(event: KeyboardEvent) {
      if (event.key === "Escape") {
        closeModal();
      }
    }
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [modal, submitting]);

  function togglePermissions(
    permissions: UserPermission[],
    permission: UserPermission,
    onChange: (next: UserPermission[]) => void,
  ) {
    onChange(
      permissions.includes(permission)
        ? permissions.filter((value) => value !== permission)
        : [...permissions, permission],
    );
  }

  async function handleCreate(event: FormEvent) {
    event.preventDefault();
    try {
      setSubmitting(true);
      await api.createUser(createForm);
      closeModal();
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to create user");
    } finally {
      setSubmitting(false);
    }
  }

  async function handleEdit(event: FormEvent) {
    event.preventDefault();
    if (!modal || modal.type !== "edit" || !editForm) {
      return;
    }
    try {
      setSubmitting(true);
      await api.updateUser(modal.user.user.id, editForm);
      closeModal();
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to update user");
    } finally {
      setSubmitting(false);
    }
  }

  async function handlePasswordChange(event: FormEvent) {
    event.preventDefault();
    if (!modal || modal.type !== "password") {
      return;
    }
    if (!password.trim()) {
      setError("Password must not be empty");
      return;
    }
    try {
      setSubmitting(true);
      await api.changeUserPassword(modal.user.user.id, { password });
      closeModal();
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to change password");
    } finally {
      setSubmitting(false);
    }
  }

  async function handleDelete() {
    if (!modal || modal.type !== "delete") {
      return;
    }
    try {
      setSubmitting(true);
      await api.deleteUser(modal.user.user.id);
      closeModal();
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to delete user");
    } finally {
      setSubmitting(false);
    }
  }

  if (loading) {
    return <LoadingBlock label="Loading users…" lines={4} />;
  }

  return (
    <div className="space-y-8">
      <PageHeader
        eyebrow="Access Control"
        title="Users"
        description="Manage operator accounts, credentials, repository access, and download usage from one screen."
        actions={[
          {
            onClick: openCreateModal,
            label: "Add User",
            icon: faPlus,
            variant: "primary",
          },
        ]}
      />

      {error ? (
        <div className="border border-zinc-800 bg-black p-4 text-zinc-200">
          Error: {error}
        </div>
      ) : null}

      {users.length === 0 ? (
        <EmptyState>No users have been created yet.</EmptyState>
      ) : (
        <section className="overflow-hidden border border-zinc-800 bg-black">
          <div className="border-b border-zinc-800 px-6 py-4">
            <div className="flex items-center gap-3">
              <div className="flex h-11 w-11 items-center justify-center border border-zinc-800 bg-zinc-950 text-white">
                <FaIcon icon={faUsers} />
              </div>
              <div>
                <h2 className="text-xl font-semibold text-white">
                  User Directory
                </h2>
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
                <article
                  key={entry.user.id}
                  className="flex flex-col gap-5 px-6 py-5"
                >
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
                      <button
                        type="button"
                        onClick={() => openEditModal(entry)}
                        className="inline-flex min-w-[108px] items-center justify-center border border-zinc-800 bg-black px-3 py-2 text-sm text-zinc-200 transition hover:border-zinc-600 hover:bg-zinc-950"
                      >
                        <FaIcon icon={faPen} className="mr-2" />
                        Edit
                      </button>
                      <button
                        type="button"
                        onClick={() => openPasswordModal(entry)}
                        className="inline-flex min-w-[108px] items-center justify-center border border-zinc-800 bg-black px-3 py-2 text-sm text-zinc-200 transition hover:border-zinc-600 hover:bg-zinc-950"
                      >
                        <FaIcon icon={faKey} className="mr-2" />
                        Password
                      </button>
                      <button
                        type="button"
                        onClick={() => openDeleteModal(entry)}
                        disabled={isCurrentUser}
                        className="inline-flex min-w-[108px] items-center justify-center border border-zinc-800 bg-black px-3 py-2 text-sm text-zinc-200 transition hover:border-zinc-600 hover:bg-zinc-950 disabled:cursor-not-allowed disabled:opacity-50"
                        aria-label={`Delete user ${entry.user.handle}`}
                      >
                        <FaIcon icon={faTrash} className="mr-2" />
                        Delete
                      </button>
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
      )}

      {modal?.type === "create" ? (
        <ModalShell title="Add User" onClose={closeModal}>
          <form onSubmit={handleCreate} className="space-y-4">
            <TextField
              label="Handle"
              value={createForm.handle}
              onChange={(value) =>
                setCreateForm((current) => ({ ...current, handle: value }))
              }
            />
            <TextField
              label="Display name"
              value={createForm.display_name}
              onChange={(value) =>
                setCreateForm((current) => ({
                  ...current,
                  display_name: value,
                }))
              }
            />
            <TextField
              label="Password"
              type="password"
              value={createForm.password}
              onChange={(value) =>
                setCreateForm((current) => ({ ...current, password: value }))
              }
            />
            <ToggleField
              label="Active"
              checked={createForm.active}
              onChange={(checked) =>
                setCreateForm((current) => ({ ...current, active: checked }))
              }
            />
            <PermissionGroup
              permissions={createForm.permissions}
              onToggle={(permission) =>
                togglePermissions(createForm.permissions, permission, (next) =>
                  setCreateForm((current) => ({
                    ...current,
                    permissions: next,
                  })),
                )
              }
            />
            <ModalActions
              onClose={closeModal}
              submitting={submitting}
              submitLabel="Create user"
              submitIcon={faUserPlus}
            />
          </form>
        </ModalShell>
      ) : null}

      {modal?.type === "edit" && editForm ? (
        <ModalShell
          title={`Edit ${modal.user.user.display_name}`}
          onClose={closeModal}
        >
          <form onSubmit={handleEdit} className="space-y-4">
            <TextField
              label="Handle"
              value={editForm.handle}
              onChange={(value) =>
                setEditForm((current) =>
                  current ? { ...current, handle: value } : current,
                )
              }
            />
            <TextField
              label="Display name"
              value={editForm.display_name}
              onChange={(value) =>
                setEditForm((current) =>
                  current ? { ...current, display_name: value } : current,
                )
              }
            />
            <ToggleField
              label="Active"
              checked={editForm.active}
              onChange={(checked) =>
                setEditForm((current) =>
                  current ? { ...current, active: checked } : current,
                )
              }
            />
            <PermissionGroup
              permissions={editForm.permissions}
              onToggle={(permission) =>
                togglePermissions(editForm.permissions, permission, (next) =>
                  setEditForm((current) =>
                    current ? { ...current, permissions: next } : current,
                  ),
                )
              }
            />
            <ModalActions
              onClose={closeModal}
              submitting={submitting}
              submitLabel="Save changes"
              submitIcon={faPen}
            />
          </form>
        </ModalShell>
      ) : null}

      {modal?.type === "password" ? (
        <ModalShell
          title={`Change password for ${modal.user.user.display_name}`}
          onClose={closeModal}
        >
          <form onSubmit={handlePasswordChange} className="space-y-4">
            <TextField
              label="New password"
              type="password"
              value={password}
              onChange={setPassword}
            />
            <ModalActions
              onClose={closeModal}
              submitting={submitting}
              submitLabel="Update password"
              submitIcon={faKey}
            />
          </form>
        </ModalShell>
      ) : null}

      {modal?.type === "delete" ? (
        <ModalShell
          title={`Delete ${modal.user.user.display_name}?`}
          onClose={closeModal}
        >
          <div className="space-y-5">
            <p className="text-sm leading-6 text-zinc-400">
              This removes the account, its permissions, and its tracked repo
              download metrics. This action cannot be undone.
            </p>
            <div className="border border-zinc-800 bg-zinc-950 p-4">
              <div className="font-medium text-white">
                {modal.user.user.display_name}
              </div>
              <div className="mt-1 font-mono text-sm text-zinc-400">
                @{modal.user.user.handle}
              </div>
            </div>
            <div className="flex justify-end gap-3">
              <button
                type="button"
                onClick={closeModal}
                disabled={submitting}
                className="border border-zinc-800 bg-black px-4 py-2 text-sm font-medium text-zinc-200 transition hover:border-zinc-600 hover:bg-zinc-950"
              >
                Cancel
              </button>
              <button
                type="button"
                onClick={() => void handleDelete()}
                disabled={submitting}
                className="border border-zinc-200 bg-zinc-100 px-4 py-2 text-sm font-semibold text-black transition hover:bg-white disabled:opacity-70"
              >
                <FaIcon icon={faTrash} className="mr-2" />
                {submitting ? "Deleting…" : "Delete user"}
              </button>
            </div>
          </div>
        </ModalShell>
      ) : null}
    </div>
  );
}

function emptyCreateForm(): CreateUserRequest {
  return {
    handle: "",
    display_name: "",
    password: "",
    permissions: ["read"],
    active: true,
  };
}

function TextField({
  label,
  value,
  onChange,
  type = "text",
}: {
  label: string;
  value: string;
  onChange: (value: string) => void;
  type?: string;
}) {
  return (
    <label className="block">
      <span className="mb-2 block text-sm font-medium text-zinc-300">
        {label}
      </span>
      <input
        type={type}
        value={value}
        onChange={(event) => onChange(event.target.value)}
        className="w-full border border-zinc-800 bg-black px-4 py-3 text-sm text-white outline-none transition focus:border-zinc-600"
      />
    </label>
  );
}

function ToggleField({
  label,
  checked,
  onChange,
}: {
  label: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
}) {
  return (
    <label className="flex items-center gap-3 border border-zinc-800 bg-black px-4 py-3 text-sm text-zinc-200">
      <input
        type="checkbox"
        checked={checked}
        onChange={(event) => onChange(event.target.checked)}
      />
      {label}
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
    <div>
      <div className="mb-2 text-sm font-medium text-zinc-300">Permissions</div>
      <div
        className="grid gap-2 sm:grid-cols-3"
        role="group"
        aria-label="User permissions"
      >
        {PERMISSIONS.map((permission) => {
          const enabled = permissions.includes(permission);
          return (
            <button
              key={permission}
              type="button"
              onClick={() => onToggle(permission)}
              aria-pressed={enabled}
              className={`border px-4 py-3 text-sm capitalize transition ${
                enabled
                  ? "border-zinc-600 bg-zinc-950 text-white"
                  : "border-zinc-800 bg-black text-zinc-300 hover:border-zinc-600 hover:bg-zinc-950"
              }`}
            >
              {permission}
            </button>
          );
        })}
      </div>
    </div>
  );
}

function ModalShell({
  title,
  children,
  onClose,
}: {
  title: string;
  children: ReactNode;
  onClose: () => void;
}) {
  const titleId = useId();
  const dialogRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    const firstFocusable = dialogRef.current?.querySelector<HTMLElement>(
      'input, select, textarea, button, [href], [tabindex]:not([tabindex="-1"])',
    );
    firstFocusable?.focus();
  }, []);

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 px-4 py-8"
      onMouseDown={(event) => {
        if (event.target === event.currentTarget) {
          onClose();
        }
      }}
    >
      <div
        ref={dialogRef}
        role="dialog"
        aria-modal="true"
        aria-labelledby={titleId}
        className="w-full max-w-xl border border-zinc-800 bg-black p-6 shadow-2xl"
      >
        <div className="mb-5 flex items-start justify-between gap-4">
          <div>
            <div className="text-xs uppercase tracking-[0.22em] text-zinc-500">
              Users
            </div>
            <h2 id={titleId} className="mt-2 text-2xl font-semibold text-white">
              {title}
            </h2>
          </div>
          <button
            type="button"
            onClick={onClose}
            className="border border-zinc-800 px-3 py-2 text-sm text-zinc-300 transition hover:border-zinc-600 hover:bg-zinc-950"
          >
            Close
          </button>
        </div>
        {children}
      </div>
    </div>
  );
}

function ModalActions({
  onClose,
  submitting,
  submitLabel,
  submitIcon,
}: {
  onClose: () => void;
  submitting: boolean;
  submitLabel: string;
  submitIcon: typeof faUserPlus;
}) {
  return (
    <div className="flex justify-end gap-3">
      <button
        type="button"
        onClick={onClose}
        disabled={submitting}
        className="border border-zinc-800 bg-black px-4 py-2 text-sm font-medium text-zinc-200 transition hover:border-zinc-600 hover:bg-zinc-950"
      >
        Cancel
      </button>
      <button
        type="submit"
        disabled={submitting}
        className="border border-zinc-200 bg-zinc-100 px-4 py-2 text-sm font-semibold text-black transition hover:bg-white disabled:opacity-70"
      >
        <FaIcon icon={submitIcon} className="mr-2" />
        {submitting ? "Saving…" : submitLabel}
      </button>
    </div>
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

function formatBytes(bytes: number) {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KiB`;
  if (bytes < 1024 * 1024 * 1024)
    return `${(bytes / (1024 * 1024)).toFixed(1)} MiB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GiB`;
}
