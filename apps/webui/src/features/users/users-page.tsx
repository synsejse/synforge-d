import { useState, type SyntheticEvent } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { getRouteApi } from "@tanstack/react-router";
import { faPlus } from "@fortawesome/free-solid-svg-icons";
import api from "../../lib/api";
import { usersQueries } from "../../lib/queries";
import type { UserResponse } from "../../lib/types";
import ErrorMessage from "../../components/common/error-message";
import PaginationControls from "../../components/common/pagination-controls";
import { useSession } from "../../components/common/session-provider";
import EmptyState from "../../components/ui/empty-state";
import LoadingBlock from "../../components/ui/loading-block";
import PageHeader from "../../components/ui/page-header";
import UserDirectory from "./components/user-directory";
import UserModals from "./components/user-modals";
import {
  type CreateUserDraft,
  type ModalState,
  type UserDraft,
  emptyCreateForm,
} from "./components/model";
import {
  createUserSchema,
  editUserSchema,
  flatErrors,
  passwordSchema,
  type CreateFieldErrors,
  type EditFieldErrors,
} from "./components/validation";

const route = getRouteApi("/_authed/users");

const PAGE_SIZE = 50;

function Users() {
  const queryClient = useQueryClient();
  const { session } = useSession();
  const currentUserId = session?.user.id ?? null;

  const navigate = route.useNavigate();
  const offset = route.useSearch().offset ?? 0;
  const setOffset = (next: number) =>
    navigate({ search: (prev) => ({ ...prev, offset: next }) });

  const usersQuery = useQuery(
    usersQueries.list({ limit: PAGE_SIZE, offset }),
  );

  const [error, setError] = useState<string | null>(null);
  const [modal, setModal] = useState<ModalState>(null);
  const [createForm, setCreateForm] =
    useState<CreateUserDraft>(emptyCreateForm());
  const [createErrors, setCreateErrors] = useState<CreateFieldErrors>({});
  const [editForm, setEditForm] = useState<UserDraft | null>(null);
  const [editErrors, setEditErrors] = useState<EditFieldErrors>({});
  const [password, setPassword] = useState("");
  const [passwordError, setPasswordError] = useState<string | null>(null);

  const invalidateUsers = () =>
    queryClient.invalidateQueries({ queryKey: ["users"] });

  function openCreateModal() {
    setCreateForm(emptyCreateForm());
    setCreateErrors({});
    setModal({ type: "create" });
  }

  function openEditModal(user: UserResponse) {
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
    setPassword("");
    setPasswordError(null);
    setModal({ type: "password", user });
  }

  function openDeleteModal(user: UserResponse) {
    setModal({ type: "delete", user });
  }

  const createMutation = useMutation({
    mutationFn: () => api.createUser(createForm),
    onSuccess: async () => {
      forceCloseModal();
      await invalidateUsers();
    },
    onError: (err) =>
      setError(err instanceof Error ? err.message : "Failed to create user"),
  });

  const editMutation = useMutation({
    mutationFn: ({ userId, draft }: { userId: string; draft: UserDraft }) =>
      api.updateUser(userId, draft),
    onSuccess: async () => {
      forceCloseModal();
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
      forceCloseModal();
      setError(null);
    },
    onError: (err) =>
      setError(err instanceof Error ? err.message : "Failed to change password"),
  });

  const deleteMutation = useMutation({
    mutationFn: (userId: string) => api.deleteUser(userId),
    onSuccess: async () => {
      // Bypass closeModal's submitting-guard: the closure captures
      // submitting=true from the click that started the mutation, so
      // closeModal would early-return and the dialog stayed open.
      forceCloseModal();
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

  // Escape / outside-click are owned by the Radix dialog inside UserModalShell,
  // which also restores focus to the trigger on close. The submitting guard
  // here keeps the dialog open while a mutation is in flight.
  function closeModal() {
    if (submitting) {
      return;
    }
    forceCloseModal();
  }

  function forceCloseModal() {
    setModal(null);
    setEditForm(null);
    setPassword("");
  }

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

  const loading = usersQuery.isPending;
  const users = usersQuery.data?.users ?? [];

  return (
    <div className="space-y-6">
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

      {loading ? (
        <LoadingBlock label="Loading users…" lines={4} />
      ) : users.length === 0 ? (
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

      {!loading && usersQuery.data && users.length > 0 && (
        <PaginationControls
          offset={offset}
          pageSize={PAGE_SIZE}
          count={users.length}
          hasMore={usersQuery.data.page.has_more}
          total={usersQuery.data.page.total}
          isFetching={usersQuery.isFetching}
          onOffsetChange={setOffset}
        />
      )}

      <UserModals
        modal={modal}
        submitting={submitting}
        onClose={closeModal}
        createForm={createForm}
        setCreateForm={setCreateForm}
        createErrors={createErrors}
        setCreateErrors={setCreateErrors}
        onCreate={handleCreate}
        editForm={editForm}
        setEditForm={setEditForm}
        editErrors={editErrors}
        setEditErrors={setEditErrors}
        onEdit={handleEdit}
        password={password}
        setPassword={setPassword}
        passwordError={passwordError}
        setPasswordError={setPasswordError}
        onPasswordChange={handlePasswordChange}
        onDelete={handleDelete}
      />
    </div>
  );
}

export default function UsersPage() {
  return <Users />;
}
