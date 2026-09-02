import type { Block } from "../bindings/Block";
import type { PublicClass } from "../bindings/PublicClass";
import type { StudioPage } from "../bindings/StudioPage";
import type { UpdatePageParams } from "../bindings/UpdatePageParams";
import type { UploadedFile } from "../bindings/UploadedFile";
import { getToken } from "../auth/token";
import { ApiClientError, get, put } from "./client";

export function getMyStudioPage(): Promise<StudioPage> {
  return get<StudioPage>("/api/studio/page");
}

export function updateStudioPage(blocks: Block[]): Promise<StudioPage> {
  const body: UpdatePageParams = { blocks };
  return put<StudioPage>("/api/studio/page", body);
}

export function getPublicStudioPage(token: string): Promise<StudioPage> {
  return get<StudioPage>(`/api/public/organizations/${token}/page`);
}

export function getPublicStudioClasses(token: string): Promise<PublicClass[]> {
  return get<PublicClass[]>(`/api/public/organizations/${token}/classes`);
}

// Multipart upload (see api/classes.ts uploadClassPhoto for why the shared JSON
// client can't be reused).
export async function uploadStudioImage(file: File): Promise<UploadedFile> {
  const form = new FormData();
  form.append("image", file);
  const headers: Record<string, string> = {};
  const token = getToken();
  if (token) {
    headers["Authorization"] = `Bearer ${token}`;
  }
  const res = await fetch("/api/studio/uploads", {
    method: "POST",
    headers,
    body: form,
  });
  if (!res.ok) {
    throw new ApiClientError(res.status, `Upload failed (${res.status})`);
  }
  return (await res.json()) as UploadedFile;
}
