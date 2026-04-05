import { useEffect, useRef, useState, type FormEvent } from "react";
import {
  faKey,
  faPen,
  faPlus,
  faTrash,
  faUserPlus,
} from "@fortawesome/free-solid-svg-icons";
import api from "../../lib/api";
import type { CreateUserRequest, SessionResponse, UserResponse } from "../../lib/types";
import ErrorMessage from "../common/ErrorMessage";
import EmptyState from "../ui/EmptyState";
import FaIcon from "../ui/FaIcon";
import LoadingBlock from "../ui/LoadingBlock";
import PageHeader from "../ui/PageHeader";
import { PermissionGroup, TextField, ToggleField } from "../users/FormFields";
import { UserModalActions, UserModalShell } from "../users/ModalShell";
import UserDirectory from "../users/UserDirectory";
import {
  type ModalState,
  type UserDraft,
  emptyCreateForm,
  togglePermission,
} from "../users/model";

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
        </UserModalShell>
      ) : null}
    </div>
  );
}
