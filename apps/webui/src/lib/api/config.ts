import type {
  ConfigSchemaResponse,
  EffectiveConfigResponse,
  SetupInitializeRequest,
  ExportRepoSigningKeyResponse,
  ExportRepoSigningPublicKeyResponse,
  GenerateRepoSigningKeyResponse,
  ImportRepoSigningKeyRequest,
  ImportRepoSigningKeyResponse,
  RepoSigningReconcileProgressResponse,
  RepoSigningStatusResponse,
  SessionLoginRequest,
  SessionResponse,
  SetupStatusResponse,
  TestRepoSigningResponse,
  UpdateRepoSigningConfigRequest,
  UpdateRuntimeSettingsRequest,
} from "../types";
import { request } from "./client";

export function getSession(): Promise<SessionResponse> {
  return request("GET", "/api/v1/session");
}

export function getSetupStatus(): Promise<SetupStatusResponse> {
  return request("GET", "/api/v1/setup/status");
}

export function initializeSetup(
  req: SetupInitializeRequest,
): Promise<EffectiveConfigResponse> {
  return request("POST", "/api/v1/setup/initialize", req);
}

export function login(req: SessionLoginRequest): Promise<SessionResponse> {
  return request("POST", "/api/v1/session/login", req);
}

export function logout(): Promise<void> {
  return request("POST", "/api/v1/session/logout", {});
}

export function getConfig(): Promise<EffectiveConfigResponse> {
  return request("GET", "/api/v1/config/effective");
}

export function getConfigSchema(): Promise<ConfigSchemaResponse> {
  return request("GET", "/api/v1/config/schema");
}

export function updateRuntimeSettings(
  req: UpdateRuntimeSettingsRequest,
): Promise<EffectiveConfigResponse> {
  return request("POST", "/api/v1/config/runtime", req);
}

export function getRepoSigningStatus(): Promise<RepoSigningStatusResponse> {
  return request("GET", "/api/v1/signing/status");
}

export function getRepoSigningReconcileProgress(): Promise<RepoSigningReconcileProgressResponse> {
  return request("GET", "/api/v1/signing/reconcile/progress");
}

export function updateRepoSigningConfig(
  req: UpdateRepoSigningConfigRequest,
): Promise<RepoSigningStatusResponse> {
  return request("POST", "/api/v1/signing/config", req);
}

export function generateRepoSigningKey(): Promise<GenerateRepoSigningKeyResponse> {
  return request("POST", "/api/v1/signing/generate", {});
}

export function importRepoSigningKey(
  req: ImportRepoSigningKeyRequest,
): Promise<ImportRepoSigningKeyResponse> {
  return request("POST", "/api/v1/signing/import", req);
}

export function removeRepoSigningKey(): Promise<RepoSigningStatusResponse> {
  return request("DELETE", "/api/v1/signing/key");
}

export function testRepoSigning(): Promise<TestRepoSigningResponse> {
  return request("POST", "/api/v1/signing/test", {});
}

export function exportRepoSigningKey(): Promise<ExportRepoSigningKeyResponse> {
  return request("GET", "/api/v1/signing/export");
}

export function exportRepoSigningPublicKey(): Promise<ExportRepoSigningPublicKeyResponse> {
  return request("GET", "/api/v1/signing/export/public");
}
