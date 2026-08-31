import type { Organization } from "../bindings/Organization";
import type { Page } from "../bindings/Page";
import { get } from "./client";

// Studios are created out-of-band (operator task / seed) and a user belongs to
// exactly one, so the API — and this module — is read-only. `listOrganizations`
// returns a one-item page: the caller's own studio.

export function listOrganizations(): Promise<Page<Organization>> {
  return get<Page<Organization>>("/api/organizations");
}

export function getOrganization(id: number): Promise<Organization> {
  return get<Organization>(`/api/organizations/${id}`);
}
