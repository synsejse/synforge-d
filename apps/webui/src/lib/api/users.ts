import type {
  ChangePasswordRequest,
  CreateUserRequest as CreateUserPayload,
  UpdateUserRequest as UpdateUserPayload,
  UserListResponse,
  UserMetricsResponse,
  UserResponse,
} from "../types";
import { ConfigApiClient } from "./config";

export class UserApiClient extends ConfigApiClient {
  async listUsers(): Promise<UserListResponse> {
    return this.request("GET", "/api/v1/users");
  }

  async createUser(req: CreateUserPayload): Promise<UserResponse> {
    return this.request("POST", "/api/v1/users", req);
  }

  async updateUser(id: string, req: UpdateUserPayload): Promise<UserResponse> {
    return this.request("PUT", `/api/v1/users/${encodeURIComponent(id)}`, req);
  }

  async changeUserPassword(
    id: string,
    req: ChangePasswordRequest,
  ): Promise<void> {
    return this.request(
      "POST",
      `/api/v1/users/${encodeURIComponent(id)}/password`,
      req,
    );
  }

  async deleteUser(id: string): Promise<UserResponse> {
    return this.request("DELETE", `/api/v1/users/${encodeURIComponent(id)}`);
  }

  async getUserMetrics(id: string): Promise<UserMetricsResponse> {
    return this.request("GET", `/api/v1/users/${encodeURIComponent(id)}`);
  }
}
