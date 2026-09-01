import { get } from "./client";

export interface CurrentUser {
  pid: string;
  name: string;
  email: string;
  role: "member" | "staff" | "admin";
}

export function getCurrentUser(): Promise<CurrentUser> {
  return get<CurrentUser>("/api/auth/current");
}
