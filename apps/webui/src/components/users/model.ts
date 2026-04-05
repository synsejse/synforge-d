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

export const PERMISSIONS: UserPermission[] = ["read", "write", "repo"];

export function emptyCreateForm(): CreateUserRequest {
  return {
    handle: "",
    display_name: "",
    password: "",
    permissions: ["read"],
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
