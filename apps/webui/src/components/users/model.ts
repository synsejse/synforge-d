import type {
  CreateUserRequest,
  UserPermission,
  UserResponse,
} from "../../lib/types";

export type ModalState =
  | { type: "create" }
  | { type: "edit"; user: UserResponse }
  | { type: "password"; user: UserResponse }
  | { type: "delete"; user: UserResponse }
  | null;

export interface UserDraft {
  handle: string;
  display_name: string;
  permissions: UserPermission[];
  active: boolean;
}

export type CreateUserDraft = Omit<CreateUserRequest, "active"> & {
  active: boolean;
};

const USER_PERMISSION_LABELS: Record<UserPermission, string> = {
  read: "read",
  write: "write",
  repo: "repo",
};

export const PERMISSIONS: UserPermission[] =
  Object.keys(USER_PERMISSION_LABELS) as UserPermission[];

export function emptyCreateForm(): CreateUserDraft {
  const [defaultPermission] = PERMISSIONS;
  return {
    handle: "",
    display_name: "",
    password: "",
    permissions: [defaultPermission],
    active: true,
  };
}

export function togglePermission(
  permissions: UserPermission[],
  permission: UserPermission,
): UserPermission[] {
  if (permissions.includes(permission)) {
    return permissions.filter((value) => value !== permission);
  }
  return [...permissions, permission];
}
