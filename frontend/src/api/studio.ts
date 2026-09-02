import type { Block } from "../bindings/Block";
import type { PublicClass } from "../bindings/PublicClass";
import type { StudioPage } from "../bindings/StudioPage";
import type { UpdatePageParams } from "../bindings/UpdatePageParams";
import { get, put } from "./client";

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
