import { clearToken, getToken } from "../auth/token";

export type HttpMethod = "GET" | "POST" | "PUT" | "PATCH" | "DELETE";

/** Per-field validation messages, keyed by the backend field name. */
export type FieldErrors = Record<string, string>;

export class ApiClientError extends Error {
  readonly status: number;
  /** Field-level validation errors (from a 400 `{ errors: {...} }` body). */
  readonly fieldErrors: FieldErrors;

  constructor(status: number, message: string, fieldErrors: FieldErrors = {}) {
    super(message || `Request failed with status ${status}`);
    this.name = "ApiClientError";
    this.status = status;
    this.fieldErrors = fieldErrors;
  }
}

interface ParsedError {
  message: string | null;
  fieldErrors: FieldErrors;
}

// Loco validation errors arrive as `{ errors: { field: [{ code, message }] } }`;
// other errors carry a `message` / `description` / `error` string. Pull out
// whichever is present so callers can show a general message and/or highlight
// individual fields.
async function parseError(res: Response): Promise<ParsedError> {
  try {
    const data = (await res.json()) as Record<string, unknown>;
    const fieldErrors: FieldErrors = {};
    const errors = data.errors;
    if (errors && typeof errors === "object") {
      for (const [field, list] of Object.entries(errors)) {
        const first = Array.isArray(list) ? list[0] : undefined;
        if (first && typeof first.message === "string") {
          fieldErrors[field] = first.message;
        }
      }
    }
    const message =
      typeof data.message === "string"
        ? data.message
        : typeof data.description === "string"
          ? data.description
          : typeof data.error === "string"
            ? data.error
            : null;
    return { message, fieldErrors };
  } catch {
    return { message: null, fieldErrors: {} };
  }
}

export async function request<T>(
  method: HttpMethod,
  path: string,
  body?: unknown,
): Promise<T> {
  const headers: Record<string, string> = {};
  if (body !== undefined) {
    headers["Content-Type"] = "application/json";
  }
  const token = getToken();
  if (token) {
    headers["Authorization"] = `Bearer ${token}`;
  }

  const res = await fetch(path, {
    method,
    headers,
    credentials: "same-origin",
    body: body !== undefined ? JSON.stringify(body) : undefined,
  });

  if (res.ok) {
    return res.status === 204 ? (undefined as T) : ((await res.json()) as T);
  }

  const { message, fieldErrors } = await parseError(res);

  if (res.status === 401) {
    clearToken();
    window.location.href = "/login";
  }

  throw new ApiClientError(
    res.status,
    message ?? `Request failed with status ${res.status}`,
    fieldErrors,
  );
}

export function get<T>(path: string): Promise<T> {
  return request<T>("GET", path);
}

export function post<T>(path: string, body: unknown): Promise<T> {
  return request<T>("POST", path, body);
}

export function put<T>(path: string, body: unknown): Promise<T> {
  return request<T>("PUT", path, body);
}

export function del(path: string): Promise<void> {
  return request<void>("DELETE", path);
}
