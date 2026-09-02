import type { Class } from "../bindings/Class";
import type { CreateClassParams } from "../bindings/CreateClassParams";
import type { Page } from "../bindings/Page";
import type { UpdateClassParams } from "../bindings/UpdateClassParams";
import { getToken } from "../auth/token";
import { ApiClientError, del, get, post, put } from "./client";

export type ClassScope = "upcoming" | "all";

export function listClasses(
  scope: ClassScope = "upcoming",
  page = 1,
  pageSize = 20,
): Promise<Page<Class>> {
  const params = new URLSearchParams({
    scope,
    page: String(page),
    page_size: String(pageSize),
  });
  return get<Page<Class>>(`/api/classes?${params.toString()}`);
}

export function getClass(id: number): Promise<Class> {
  return get<Class>(`/api/classes/${id}`);
}

export function createClass(body: CreateClassParams): Promise<Class> {
  return post<Class>("/api/classes", body);
}

export function updateClass(
  id: number,
  body: UpdateClassParams,
): Promise<Class> {
  return put<Class>(`/api/classes/${id}`, body);
}

export function deleteClass(id: number): Promise<void> {
  return del(`/api/classes/${id}`);
}

// Multipart upload — the shared JSON client can't be reused (it forces a JSON
// content type; the browser must set the multipart boundary itself).
export async function uploadClassPhoto(id: number, file: File): Promise<Class> {
  const form = new FormData();
  form.append("photo", file);
  const headers: Record<string, string> = {};
  const token = getToken();
  if (token) {
    headers["Authorization"] = `Bearer ${token}`;
  }
  const res = await fetch(`/api/classes/${id}/photo`, {
    method: "POST",
    headers,
    body: form,
  });
  if (!res.ok) {
    throw new ApiClientError(res.status, `Upload failed (${res.status})`);
  }
  return (await res.json()) as Class;
}
