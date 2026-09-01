import { useQuery } from "@tanstack/react-query";
import { getCurrentUser } from "../api/auth";

/** The signed-in user (with their role). Used to gate teacher/admin controls. */
export function useCurrentUser() {
  const query = useQuery({
    queryKey: ["current-user"],
    queryFn: getCurrentUser,
  });
  const role = query.data?.role;
  return {
    ...query,
    isStaff: role === "staff" || role === "admin",
    isAdmin: role === "admin",
  };
}
