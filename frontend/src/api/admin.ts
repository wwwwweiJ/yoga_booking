import type { AdminOrganization } from "../bindings/AdminOrganization";
import type { AdminUser } from "../bindings/AdminUser";
import type { CreateOrganizationParams } from "../bindings/CreateOrganizationParams";
import type { CreateStaffParams } from "../bindings/CreateStaffParams";
import { get, post } from "./client";

export function listAdminOrganizations(): Promise<AdminOrganization[]> {
  return get<AdminOrganization[]>("/api/admin/organizations");
}

export function createAdminOrganization(
  body: CreateOrganizationParams,
): Promise<AdminOrganization> {
  return post<AdminOrganization>("/api/admin/organizations", body);
}

export function createStaff(body: CreateStaffParams): Promise<AdminUser> {
  return post<AdminUser>("/api/admin/staff", body);
}
