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

function jsonHeaders(): HeadersInit {
  return { "Content-Type": "application/json" };
}

function emitAuthRequired(path: string, error: ApiError) {
  if (typeof window === "undefined") return;
  if (path !== "/api/v1/session") return;
  window.dispatchEvent(
    new CustomEvent("synforge:auth-required", { detail: { path, error } }),
  );
}

export async function request<T>(
  method: string,
  path: string,
  body?: unknown,
): Promise<T> {
  const res = await fetch(`${API_BASE}${path}`, {
    method,
    headers: jsonHeaders(),
    credentials: "include",
    body: body ? JSON.stringify(body) : undefined,
  });

  if (!res.ok) {
    const error: ApiError = await res.json().catch(() => ({
      code: "internal_error",
      message: res.statusText,
    }));
    if (res.status === 401) {
      emitAuthRequired(path, error);
    }
    throw new ApiClientError(res.status, error);
  }

  if (res.status === 204) {
    return undefined as T;
  }

  return res.json();
}

export async function downloadStream(path: string): Promise<Response> {
  const res = await fetch(`${API_BASE}${path}`, {
    method: "GET",
    credentials: "include",
  });

  if (!res.ok) {
    const error: ApiError = await res.json().catch(() => ({
      code: "internal_error",
      message: res.statusText,
    }));
    throw new ApiClientError(res.status, error);
  }

  return res;
}
