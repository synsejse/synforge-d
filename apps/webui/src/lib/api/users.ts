import type {
  ChangePasswordRequest,
  CreateUserRequest as CreateUserPayload,
  UpdateUserRequest as UpdateUserPayload,
  UserListResponse,
  UserMetricsResponse,
  UserResponse,
} from "../types";
import { request } from "./client";

export function listUsers(): Promise<UserListResponse> {
  return request("GET", "/api/v1/users");
}

export function createUser(req: CreateUserPayload): Promise<UserResponse> {
  return request("POST", "/api/v1/users", req);
}

export function updateUser(
  id: string,
  req: UpdateUserPayload,
): Promise<UserResponse> {
  return request("PUT", `/api/v1/users/${encodeURIComponent(id)}`, req);
}

export function changeUserPassword(
  id: string,
  req: ChangePasswordRequest,
): Promise<void> {
  return request(
    "POST",
    `/api/v1/users/${encodeURIComponent(id)}/password`,
    req,
  );
}

export function deleteUser(id: string): Promise<UserResponse> {
  return request("DELETE", `/api/v1/users/${encodeURIComponent(id)}`);
}

export function getUserMetrics(id: string): Promise<UserMetricsResponse> {
  return request("GET", `/api/v1/users/${encodeURIComponent(id)}`);
}
