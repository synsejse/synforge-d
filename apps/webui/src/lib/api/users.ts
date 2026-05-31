import type {
  ChangePasswordRequest,
  CreateUserRequest as CreateUserPayload,
  UpdateUserRequest as UpdateUserPayload,
  UserListResponse,
  UserMetricsResponse,
  UserResponse,
} from "../types";
import { request } from "./client";

export function listUsers(
  limit = 50,
  offset = 0,
): Promise<UserListResponse> {
  const params = new URLSearchParams({
    limit: String(limit),
    offset: String(offset),
  });
  return request("GET", `/api/v1/users?${params.toString()}`);
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

export function deleteUser(id: string): Promise<void> {
  return request<void>("DELETE", `/api/v1/users/${encodeURIComponent(id)}`);
}

export function getUserMetrics(id: string): Promise<UserMetricsResponse> {
  return request("GET", `/api/v1/users/${encodeURIComponent(id)}`);
}
