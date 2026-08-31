import type { Class } from "../bindings/Class";
import type { CreateClassParams } from "../bindings/CreateClassParams";
import type { Page } from "../bindings/Page";
import type { UpdateClassParams } from "../bindings/UpdateClassParams";
import { del, get, post, put } from "./client";

export function listClasses(
  organizationId?: number,
  page = 1,
  pageSize = 20,
): Promise<Page<Class>> {
  const params = new URLSearchParams({
    page: String(page),
    page_size: String(pageSize),
  });
  if (organizationId !== undefined) {
    params.set("organization_id", String(organizationId));
  }
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
