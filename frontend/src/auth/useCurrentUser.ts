import { useQuery } from "@tanstack/react-query";
import { getCurrentUser } from "../api/auth";
import { getToken } from "./token";

/** The signed-in user (with their role). Used to gate teacher/admin controls.
 *  Only fetches when a token is present, so it's a no-op on public pages. */
export function useCurrentUser() {
  const query = useQuery({
    queryKey: ["current-user"],
    queryFn: getCurrentUser,
    enabled: getToken() !== null,
  });
  const role = query.data?.role;
  return {
    ...query,
    isStaff: role === "staff" || role === "admin",
    isAdmin: role === "admin",
  };
}
