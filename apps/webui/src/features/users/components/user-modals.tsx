import { type Dispatch, type SetStateAction, type SyntheticEvent } from "react";
import { faKey, faPen, faTrash, faUserPlus } from "@fortawesome/free-solid-svg-icons";
import Button from "../../../components/ui/button";
import FaIcon from "../../../components/ui/fa-icon";
import { PermissionGroup, TextField, ToggleField } from "./form-fields";
import { UserModalActions, UserModalShell } from "./modal-shell";
import {
  type CreateUserDraft,
  type ModalState,
  type UserDraft,
  togglePermission,
} from "./model";
import type { CreateFieldErrors, EditFieldErrors } from "./validation";

interface UserModalsProps {
  modal: ModalState;
  submitting: boolean;
  onClose: () => void;

  createForm: CreateUserDraft;
  setCreateForm: Dispatch<SetStateAction<CreateUserDraft>>;
  createErrors: CreateFieldErrors;
  setCreateErrors: Dispatch<SetStateAction<CreateFieldErrors>>;
  onCreate: (event: SyntheticEvent) => void;

  editForm: UserDraft | null;
  setEditForm: Dispatch<SetStateAction<UserDraft | null>>;
  editErrors: EditFieldErrors;
  setEditErrors: Dispatch<SetStateAction<EditFieldErrors>>;
  onEdit: (event: SyntheticEvent) => void;

  password: string;
  setPassword: Dispatch<SetStateAction<string>>;
  passwordError: string | null;
  setPasswordError: Dispatch<SetStateAction<string | null>>;
  onPasswordChange: (event: SyntheticEvent) => void;

  onDelete: () => void;
}

export default function UserModals({
  modal,
  submitting,
  onClose,
  createForm,
  setCreateForm,
  createErrors,
  setCreateErrors,
  onCreate,
  editForm,
  setEditForm,
  editErrors,
  setEditErrors,
  onEdit,
  password,
  setPassword,
  passwordError,
  setPasswordError,
  onPasswordChange,
  onDelete,
}: UserModalsProps) {
  if (modal?.type === "create") {
    return (
      <UserModalShell title="Add User" onClose={onClose}>
        <form onSubmit={onCreate} className="space-y-4" noValidate>
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
              setCreateForm((current) => ({ ...current, display_name: value }));
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
            onClose={onClose}
            submitting={submitting}
            submitLabel="Create user"
            submitIcon={faUserPlus}
          />
        </form>
      </UserModalShell>
    );
  }

  if (modal?.type === "edit" && editForm) {
    return (
      <UserModalShell
        title={`Edit ${modal.user.user.display_name}`}
        onClose={onClose}
      >
        <form onSubmit={onEdit} className="space-y-4" noValidate>
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
            onClose={onClose}
            submitting={submitting}
            submitLabel="Save changes"
            submitIcon={faPen}
          />
        </form>
      </UserModalShell>
    );
  }

  if (modal?.type === "password") {
    return (
      <UserModalShell
        title={`Change password for ${modal.user.user.display_name}`}
        onClose={onClose}
      >
        <form onSubmit={onPasswordChange} className="space-y-4" noValidate>
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
            onClose={onClose}
            submitting={submitting}
            submitLabel="Update password"
            submitIcon={faKey}
          />
        </form>
      </UserModalShell>
    );
  }

  if (modal?.type === "delete") {
    return (
      <UserModalShell
        title={`Delete ${modal.user.user.display_name}?`}
        onClose={onClose}
      >
        <div className="space-y-5">
          <p className="text-sm leading-6 text-muted">
            This removes the account, its permissions, and its tracked repo
            download metrics. This action cannot be undone.
          </p>
          <div className="border border-edge bg-surface-alt p-4">
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
              onClick={onClose}
              disabled={submitting}
            >
              Cancel
            </Button>
            <Button
              variant="danger"
              size="sm"
              onClick={() => onDelete()}
              loading={submitting}
            >
              {submitting ? null : <FaIcon icon={faTrash} />}
              {submitting ? "Deleting…" : "Delete user"}
            </Button>
          </div>
        </div>
      </UserModalShell>
    );
  }

  return null;
}
