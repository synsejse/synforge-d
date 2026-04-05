import type {
  ConfigSchemaResponse,
  EffectiveConfigResponse,
  SessionLoginRequest,
  SessionResponse,
  UpdateRuntimeSettingsRequest,
} from "../types";
import { JobApiClient } from "./jobs";

export class ConfigApiClient extends JobApiClient {
  async getSession(): Promise<SessionResponse> {
    return this.request("GET", "/api/v1/session");
  }

  async login(req: SessionLoginRequest): Promise<SessionResponse> {
    return this.request("POST", "/api/v1/session/login", req);
  }

  async logout(): Promise<void> {
    return this.request("POST", "/api/v1/session/logout", {});
  }

  async getConfig(): Promise<EffectiveConfigResponse> {
    return this.request("GET", "/api/v1/config/effective");
  }

  async getConfigSchema(): Promise<ConfigSchemaResponse> {
    return this.request("GET", "/api/v1/config/schema");
  }

  async updateRuntimeSettings(
    req: UpdateRuntimeSettingsRequest,
  ): Promise<EffectiveConfigResponse> {
    return this.request("POST", "/api/v1/config/runtime", req);
  }
}
