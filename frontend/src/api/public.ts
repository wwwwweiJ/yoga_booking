import type { PublicOrganization } from "../bindings/PublicOrganization";
import { get } from "./client";

// The only unauthenticated data call: resolve a studio's public name from the
// token in its /register/<token> link. Single-token lookup, never a list.
export function getPublicOrganization(
  token: string,
): Promise<PublicOrganization> {
  return get<PublicOrganization>(`/api/public/organizations/${token}`);
}
