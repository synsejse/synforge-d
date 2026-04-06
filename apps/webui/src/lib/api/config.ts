import type {
  ConfigSchemaResponse,
  EffectiveConfigResponse,
  ExportRepoSigningKeyResponse,
  ExportRepoSigningPublicKeyResponse,
  GenerateRepoSigningKeyResponse,
  ImportRepoSigningKeyRequest,
  ImportRepoSigningKeyResponse,
  RepoSigningReconcileProgressResponse,
  RepoSigningStatusResponse,
  SessionLoginRequest,
  SessionResponse,
  TestRepoSigningResponse,
  UpdateRepoSigningConfigRequest,
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

  async getRepoSigningStatus(): Promise<RepoSigningStatusResponse> {
    return this.request("GET", "/api/v1/signing/status");
  }

  async getRepoSigningReconcileProgress(): Promise<RepoSigningReconcileProgressResponse> {
    return this.request("GET", "/api/v1/signing/reconcile/progress");
  }

  async updateRepoSigningConfig(
    req: UpdateRepoSigningConfigRequest,
  ): Promise<RepoSigningStatusResponse> {
    return this.request("POST", "/api/v1/signing/config", req);
  }

  async generateRepoSigningKey(): Promise<GenerateRepoSigningKeyResponse> {
    return this.request("POST", "/api/v1/signing/generate", {});
  }

  async importRepoSigningKey(
    req: ImportRepoSigningKeyRequest,
  ): Promise<ImportRepoSigningKeyResponse> {
    return this.request("POST", "/api/v1/signing/import", req);
  }

  async removeRepoSigningKey(): Promise<RepoSigningStatusResponse> {
    return this.request("DELETE", "/api/v1/signing/key");
  }

  async testRepoSigning(): Promise<TestRepoSigningResponse> {
    return this.request("POST", "/api/v1/signing/test", {});
  }

  async exportRepoSigningKey(): Promise<ExportRepoSigningKeyResponse> {
    return this.request("GET", "/api/v1/signing/export");
  }

  async exportRepoSigningPublicKey(): Promise<ExportRepoSigningPublicKeyResponse> {
    return this.request("GET", "/api/v1/signing/export/public");
  }
}
