import { useEffect, useRef, useState, type SyntheticEvent } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { z } from "zod";
import {
  faKey,
  faPen,
  faPlus,
  faTrash,
  faUserPlus,
} from "@fortawesome/free-solid-svg-icons";
import api from "../../lib/api";
import { usersQueries } from "../../lib/queries";
import type { UserResponse } from "../../lib/types";
import ErrorMessage from "../../components/common/error-message";
import { useSession } from "../../components/common/session-provider";
import Button from "../../components/ui/button";
import EmptyState from "../../components/ui/empty-state";
import FaIcon from "../../components/ui/fa-icon";
import LoadingBlock from "../../components/ui/loading-block";
import PageHeader from "../../components/ui/page-header";
import { PermissionGroup, TextField, ToggleField } from "./components/form-fields";
import { UserModalActions, UserModalShell } from "./components/modal-shell";
import UserDirectory from "./components/user-directory";
import {
  type CreateUserDraft,
  type ModalState,
  type UserDraft,
  emptyCreateForm,
  togglePermission,
} from "./components/model";

type CreateFieldErrors = Partial<Record<"handle" | "display_name" | "password", string>>;
type EditFieldErrors = Partial<Record<"handle" | "display_name", string>>;

const handleSchema = z
  .string()
  .trim()
  .min(1, "Handle is required")
  .max(64, "Handle must be 64 characters or fewer")
  .regex(
    /^[a-z0-9][a-z0-9_-]*$/,
    "Use lowercase letters, digits, underscore, or hyphen",
  );
const displayNameSchema = z
  .string()
  .trim()
  .min(1, "Display name is required")
  .max(120, "Display name is too long");
const passwordSchema = z
  .string()
  .min(8, "Password must be at least 8 characters");

const createUserSchema = z.object({
  handle: handleSchema,
  display_name: displayNameSchema,
  password: passwordSchema,
});
const editUserSchema = z.object({
  handle: handleSchema,
  display_name: displayNameSchema,
});

function flatErrors<T extends string>(
  result: { error: { issues: Array<{ path: PropertyKey[]; message: string }> } },
): Partial<Record<T, string>> {
  const out: Partial<Record<T, string>> = {};
  for (const issue of result.error.issues) {
    const key = issue.path[0];
    if (typeof key === "string" && !(key in out)) {
      (out as Record<string, string>)[key] = issue.message;
    }
  }
  return out;
}

function Users() {
  const queryClient = useQueryClient();
  const { session } = useSession();
  const currentUserId = session?.user.id ?? null;

  const usersQuery = useQuery(usersQueries.list());

  const [error, setError] = useState<string | null>(null);
  const [modal, setModal] = useState<ModalState>(null);
  const [createForm, setCreateForm] =
    useState<CreateUserDraft>(emptyCreateForm());
  const [createErrors, setCreateErrors] = useState<CreateFieldErrors>({});
  const [editForm, setEditForm] = useState<UserDraft | null>(null);
  const [editErrors, setEditErrors] = useState<EditFieldErrors>({});
  const [password, setPassword] = useState("");
  const [passwordError, setPasswordError] = useState<string | null>(null);
  const lastFocusedRef = useRef<HTMLElement | null>(null);

  const invalidateUsers = () =>
    queryClient.invalidateQueries({ queryKey: usersQueries.list().queryKey });

  function openCreateModal() {
    lastFocusedRef.current =
      document.activeElement instanceof HTMLElement
        ? document.activeElement
        : null;
    setCreateForm(emptyCreateForm());
    setCreateErrors({});
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
    setEditErrors({});
    setModal({ type: "edit", user });
  }

  function openPasswordModal(user: UserResponse) {
    lastFocusedRef.current =
      document.activeElement instanceof HTMLElement
        ? document.activeElement
        : null;
    setPassword("");
    setPasswordError(null);
    setModal({ type: "password", user });
  }

  function openDeleteModal(user: UserResponse) {
    lastFocusedRef.current =
      document.activeElement instanceof HTMLElement
        ? document.activeElement
        : null;
    setModal({ type: "delete", user });
  }

  const createMutation = useMutation({
    mutationFn: () => api.createUser(createForm),
    onSuccess: async () => {
      closeModal();
      await invalidateUsers();
    },
    onError: (err) =>
      setError(err instanceof Error ? err.message : "Failed to create user"),
  });

  const editMutation = useMutation({
    mutationFn: ({
      userId,
      draft,
    }: {
      userId: string;
      draft: UserDraft;
    }) => api.updateUser(userId, draft),
    onSuccess: async () => {
      closeModal();
      await invalidateUsers();
    },
    onError: (err) =>
      setError(err instanceof Error ? err.message : "Failed to update user"),
  });

  const passwordMutation = useMutation({
    mutationFn: ({
      userId,
      newPassword,
    }: {
      userId: string;
      newPassword: string;
    }) => api.changeUserPassword(userId, { password: newPassword }),
    onSuccess: () => {
      closeModal();
      setError(null);
    },
    onError: (err) =>
      setError(err instanceof Error ? err.message : "Failed to change password"),
  });

  const deleteMutation = useMutation({
    mutationFn: (userId: string) => api.deleteUser(userId),
    onSuccess: async () => {
      closeModal();
      await invalidateUsers();
    },
    onError: (err) =>
      setError(err instanceof Error ? err.message : "Failed to delete user"),
  });

  const submitting =
    createMutation.isPending ||
    editMutation.isPending ||
    passwordMutation.isPending ||
    deleteMutation.isPending;

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
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [modal, submitting]);

  function handleCreate(event: SyntheticEvent) {
    event.preventDefault();
    const result = createUserSchema.safeParse({
      handle: createForm.handle,
      display_name: createForm.display_name,
      password: createForm.password,
    });
    if (!result.success) {
      setCreateErrors(flatErrors<"handle" | "display_name" | "password">(result));
      return;
    }
    setCreateErrors({});
    createMutation.mutate();
  }

  function handleEdit(event: SyntheticEvent) {
    event.preventDefault();
    if (!modal || modal.type !== "edit" || !editForm) {
      return;
    }
    const result = editUserSchema.safeParse({
      handle: editForm.handle,
      display_name: editForm.display_name,
    });
    if (!result.success) {
      setEditErrors(flatErrors<"handle" | "display_name">(result));
      return;
    }
    setEditErrors({});
    editMutation.mutate({ userId: modal.user.user.id, draft: editForm });
  }

  function handlePasswordChange(event: SyntheticEvent) {
    event.preventDefault();
    if (!modal || modal.type !== "password") {
      return;
    }
    const result = passwordSchema.safeParse(password);
    if (!result.success) {
      setPasswordError(result.error.issues[0]?.message ?? "Invalid password");
      return;
    }
    setPasswordError(null);
    passwordMutation.mutate({
      userId: modal.user.user.id,
      newPassword: password,
    });
  }

  function handleDelete() {
    if (!modal || modal.type !== "delete") {
      return;
    }
    deleteMutation.mutate(modal.user.user.id);
  }

  if (usersQuery.isPending) {
    return <LoadingBlock label="Loading users…" lines={4} />;
  }

  const loadError = usersQuery.error;
  if (loadError) {
    return (
      <ErrorMessage
        message={
          loadError instanceof Error ? loadError.message : "Failed to load users"
        }
      />
    );
  }

  return (
    <div className="space-y-8">
      <PageHeader
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

      {usersQuery.data.users.length === 0 ? (
        <EmptyState>No users have been created yet.</EmptyState>
      ) : (
        <UserDirectory
          users={usersQuery.data.users}
          currentUserId={currentUserId}
          onEdit={openEditModal}
          onPassword={openPasswordModal}
          onDelete={openDeleteModal}
        />
      )}

      {modal?.type === "create" ? (
        <UserModalShell title="Add User" onClose={closeModal}>
          <form onSubmit={handleCreate} className="space-y-4" noValidate>
            <TextField
              label="Handle"
              value={createForm.handle}
              error={createErrors.handle}
              onChange={(value) => {
                setCreateForm((current) => ({ ...current, handle: value }));
                if (createErrors.handle) {
                  setCreateErrors((prev) => ({ ...prev, handle: undefined }));
                }
              }}
            />
            <TextField
              label="Display name"
              value={createForm.display_name}
              error={createErrors.display_name}
              onChange={(value) => {
                setCreateForm((current) => ({
                  ...current,
                  display_name: value,
                }));
                if (createErrors.display_name) {
                  setCreateErrors((prev) => ({ ...prev, display_name: undefined }));
                }
              }}
            />
            <TextField
              label="Password"
              type="password"
              value={createForm.password}
              error={createErrors.password}
              onChange={(value) => {
                setCreateForm((current) => ({ ...current, password: value }));
                if (createErrors.password) {
                  setCreateErrors((prev) => ({ ...prev, password: undefined }));
                }
              }}
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
          <form onSubmit={handleEdit} className="space-y-4" noValidate>
            <TextField
              label="Handle"
              value={editForm.handle}
              error={editErrors.handle}
              onChange={(value) => {
                setEditForm((current) =>
                  current ? { ...current, handle: value } : current,
                );
                if (editErrors.handle) {
                  setEditErrors((prev) => ({ ...prev, handle: undefined }));
                }
              }}
            />
            <TextField
              label="Display name"
              value={editForm.display_name}
              error={editErrors.display_name}
              onChange={(value) => {
                setEditForm((current) =>
                  current ? { ...current, display_name: value } : current,
                );
                if (editErrors.display_name) {
                  setEditErrors((prev) => ({ ...prev, display_name: undefined }));
                }
              }}
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
          <form onSubmit={handlePasswordChange} className="space-y-4" noValidate>
            <TextField
              label="New password"
              type="password"
              value={password}
              error={passwordError ?? undefined}
              onChange={(value) => {
                setPassword(value);
                if (passwordError) setPasswordError(null);
              }}
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
            <p className="text-sm leading-6 text-muted">
              This removes the account, its permissions, and its tracked repo
              download metrics. This action cannot be undone.
            </p>
            <div className="border-2 border-edge-strong bg-surface-alt p-4">
              <div className="font-mono font-semibold uppercase text-white">
                {modal.user.user.display_name}
              </div>
              <div className="mt-1 font-mono text-sm text-muted">
                @{modal.user.user.handle}
              </div>
            </div>
            <div className="flex justify-end gap-3">
              <Button
                variant="ghost"
                size="sm"
                onClick={closeModal}
                disabled={submitting}
              >
                Cancel
              </Button>
              <Button
                variant="danger"
                size="sm"
                onClick={() => void handleDelete()}
                loading={submitting}
              >
                {submitting ? null : <FaIcon icon={faTrash} />}
                {submitting ? "Deleting…" : "Delete user"}
              </Button>
            </div>
          </div>
        </UserModalShell>
      ) : null}
    </div>
  );
}

export default function UsersPage() {
  return <Users />;
}
