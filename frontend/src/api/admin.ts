import type { AdminOrganization } from "../bindings/AdminOrganization";
import type { AdminUser } from "../bindings/AdminUser";
import type { AdminUserListItem } from "../bindings/AdminUserListItem";
import type { CreateOrganizationParams } from "../bindings/CreateOrganizationParams";
import type { CreateStaffParams } from "../bindings/CreateStaffParams";
import type { SetRoleParams } from "../bindings/SetRoleParams";
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

export function listAdminUsers(
  organizationId: number,
): Promise<AdminUserListItem[]> {
  return get<AdminUserListItem[]>(
    `/api/admin/users?organization_id=${organizationId}`,
  );
}

export function setUserRole(
  pid: string,
  role: string,
): Promise<AdminUserListItem> {
  const body: SetRoleParams = { role };
  return post<AdminUserListItem>(`/api/admin/users/${pid}/role`, body);
}
