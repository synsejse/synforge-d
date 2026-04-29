import { useEffect, useRef, useState, type SyntheticEvent } from "react";
import {
  faKey,
  faPen,
  faPlus,
  faTrash,
  faUserPlus,
} from "@fortawesome/free-solid-svg-icons";
import api from "../../lib/api";
import type { UserResponse } from "../../lib/types";
import PageRoot from "../../components/common/PageRoot";
import ErrorMessage from "../../components/common/ErrorMessage";
import SessionProvider, {
  useSession,
} from "../../components/common/SessionProvider";
import EmptyState from "../../components/ui/EmptyState";
import FaIcon from "../../components/ui/FaIcon";
import LoadingBlock from "../../components/ui/LoadingBlock";
import PageHeader from "../../components/ui/PageHeader";
import { PermissionGroup, TextField, ToggleField } from "./components/FormFields";
import { UserModalActions, UserModalShell } from "./components/ModalShell";
import UserDirectory from "./components/UserDirectory";
import {
  type CreateUserDraft,
  type ModalState,
  type UserDraft,
  emptyCreateForm,
  togglePermission,
} from "./components/model";

function Users() {
  const { session } = useSession();
  const currentUserId = session?.user.id ?? null;
  const [users, setUsers] = useState<UserResponse[]>([]);
  const [loading, setLoading] = useState(true);
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [modal, setModal] = useState<ModalState>(null);
  const [createForm, setCreateForm] =
    useState<CreateUserDraft>(emptyCreateForm());
  const [editForm, setEditForm] = useState<UserDraft | null>(null);
  const [password, setPassword] = useState("");
  const lastFocusedRef = useRef<HTMLElement | null>(null);

  useEffect(() => {
    void load();
  }, []);

  async function load() {
    try {
      setLoading(true);
      const usersRes = await api.listUsers();
      setUsers(usersRes.users);
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

  async function handleCreate(event: SyntheticEvent) {
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

  async function handleEdit(event: SyntheticEvent) {
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

  async function handlePasswordChange(event: SyntheticEvent) {
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
        eyebrow="ACCESS_CONTROL"
        title="Users"
        description="Manage operator accounts, credentials, repository access, and download usage from one screen."
        color="white"
        actions={[
          {
            onClick: openCreateModal,
            label: "Add User",
            icon: faPlus,
            variant: "primary",
          },
        ]}
      />

      {error ? <ErrorMessage message={error} /> : null}

      {users.length === 0 ? (
        <EmptyState>No users have been created yet.</EmptyState>
      ) : (
        <UserDirectory
          users={users}
          currentUserId={currentUserId}
          onEdit={openEditModal}
          onPassword={openPasswordModal}
          onDelete={openDeleteModal}
        />
      )}

      {modal?.type === "create" ? (
        <UserModalShell title="Add User" onClose={closeModal}>
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
                setCreateForm((current) => ({
                  ...current,
                  permissions: togglePermission(current.permissions, permission),
                }))
              }
            />
            <UserModalActions
              onClose={closeModal}
              submitting={submitting}
              submitLabel="Create user"
              submitIcon={faUserPlus}
            />
          </form>
        </UserModalShell>
      ) : null}

      {modal?.type === "edit" && editForm ? (
        <UserModalShell
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
                setEditForm((current) =>
                  current
                    ? {
                        ...current,
                        permissions: togglePermission(
                          current.permissions,
                          permission,
                        ),
                      }
                    : current,
                )
              }
            />
            <UserModalActions
              onClose={closeModal}
              submitting={submitting}
              submitLabel="Save changes"
              submitIcon={faPen}
            />
          </form>
        </UserModalShell>
      ) : null}

      {modal?.type === "password" ? (
        <UserModalShell
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
            <UserModalActions
              onClose={closeModal}
              submitting={submitting}
              submitLabel="Update password"
              submitIcon={faKey}
            />
          </form>
        </UserModalShell>
      ) : null}

      {modal?.type === "delete" ? (
        <UserModalShell
          title={`Delete ${modal.user.user.display_name}?`}
          onClose={closeModal}
        >
          <div className="space-y-5">
            <p className="text-sm leading-6 text-zinc-400">
              This removes the account, its permissions, and its tracked repo
              download metrics. This action cannot be undone.
            </p>
            <div className="border-2 border-zinc-700 bg-zinc-950 p-4">
              <div className="font-mono font-semibold uppercase text-white">
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
                className="border-2 border-zinc-700 bg-black px-4 py-2 font-mono text-xs font-bold uppercase tracking-[0.15em] text-zinc-200 transition duration-100 ease-linear hover:-translate-x-[1px] hover:-translate-y-[1px] hover:border-white hover:bg-zinc-950"
              >
                Cancel
              </button>
              <button
                type="button"
                onClick={() => void handleDelete()}
                disabled={submitting}
                className="border-2 border-[var(--theme-error-red)] bg-[var(--theme-error-red)] px-4 py-2 font-mono text-xs font-bold uppercase tracking-[0.15em] text-black transition duration-100 ease-linear hover:-translate-x-[1px] hover:-translate-y-[1px] hover:brightness-110 disabled:opacity-70"
              >
                <FaIcon icon={faTrash} className="mr-2" />
                {submitting ? "Deleting…" : "Delete user"}
              </button>
            </div>
          </div>
        </UserModalShell>
      ) : null}
    </div>
  );
}

export default function UsersPage() {
  return (
    <PageRoot>
      <SessionProvider>
        <Users />
      </SessionProvider>
    </PageRoot>
  );
}
