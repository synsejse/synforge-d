import type { ApiError } from "../types";

export const API_BASE = import.meta.env.PUBLIC_API_URL || "";

export class ApiClientError extends Error {
  status: number;
  error: ApiError;

  constructor(status: number, error: ApiError) {
    super(error.message);
    this.name = "ApiClientError";
    this.status = status;
    this.error = error;
  }
}

export class BaseApiClient {
  protected authHeaders(contentType = true): Record<string, string> {
    const headers: Record<string, string> = {};
    if (contentType) {
      headers["Content-Type"] = "application/json";
    }
    return headers;
  }

  protected async request<T>(
    method: string,
    path: string,
    body?: unknown,
  ): Promise<T> {
    const headers = this.authHeaders();

    const res = await fetch(`${API_BASE}${path}`, {
      method,
      headers,
      credentials: "include",
      body: body ? JSON.stringify(body) : undefined,
    });

    if (!res.ok) {
      const error: ApiError = await res.json().catch(() => ({
        code: "internal_error",
        message: res.statusText,
      }));
      if (res.status === 401 && typeof window !== "undefined") {
        window.dispatchEvent(
          new CustomEvent("synforge:auth-required", {
            detail: {
              path,
              error,
            },
          }),
        );
      }
      throw new ApiClientError(res.status, error);
    }

    if (res.status === 204) {
      return undefined as T;
    }

    return res.json();
  }
}
