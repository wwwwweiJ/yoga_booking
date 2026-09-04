import { useState } from "react";
import type { FormEvent } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { ApiClientError } from "../../api/client";
import {
  createAdminOrganization,
  createStaff,
  listAdminOrganizations,
  listAdminUsers,
  setUserRole,
} from "../../api/admin";
import { useCurrentUser } from "../../auth/useCurrentUser";
import { useI18n } from "../../i18n";
import { Spinner } from "../../components/Spinner";

export function Admin() {
  const { t } = useI18n();
  const { isAdmin, isPending } = useCurrentUser();
  const queryClient = useQueryClient();

  const orgs = useQuery({
    queryKey: ["admin-organizations"],
    queryFn: listAdminOrganizations,
    enabled: isAdmin,
  });

  // New studio
  const [orgName, setOrgName] = useState("");
  const [orgTz, setOrgTz] = useState("Asia/Taipei");
  const [orgError, setOrgError] = useState<string | null>(null);
  const createOrg = useMutation({
    mutationFn: () =>
      createAdminOrganization({ name: orgName, timezone: orgTz }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["admin-organizations"] });
      setOrgName("");
    },
    onError: (err) =>
      setOrgError(
        err instanceof ApiClientError ? err.message : t("admin.createFailed"),
      ),
  });

  // New teacher
  const [staffOrg, setStaffOrg] = useState<number | "">("");
  const [staffName, setStaffName] = useState("");
  const [staffEmail, setStaffEmail] = useState("");
  const [staffPassword, setStaffPassword] = useState("");
  const [staffError, setStaffError] = useState<string | null>(null);
  const [staffOk, setStaffOk] = useState(false);
  const addTeacher = useMutation({
    mutationFn: () =>
      createStaff({
        organization_id: staffOrg as number,
        name: staffName,
        email: staffEmail,
        password: staffPassword,
      }),
    onSuccess: () => {
      setStaffOk(true);
      setStaffName("");
      setStaffEmail("");
      setStaffPassword("");
    },
    onError: (err) =>
      setStaffError(
        err instanceof ApiClientError ? err.message : t("admin.createFailed"),
      ),
  });

  // Members of a studio + role management. This is how a teacher who signed in
  // with LINE (and so starts as a plain member) gets promoted to staff.
  const [usersOrg, setUsersOrg] = useState<number | "">("");
  const members = useQuery({
    queryKey: ["admin-users", usersOrg],
    queryFn: () => listAdminUsers(usersOrg as number),
    enabled: isAdmin && usersOrg !== "",
  });
  const changeRole = useMutation({
    mutationFn: (vars: { pid: string; role: string }) =>
      setUserRole(vars.pid, vars.role),
    onSuccess: () =>
      queryClient.invalidateQueries({ queryKey: ["admin-users", usersOrg] }),
  });

  if (isPending) {
    return <Spinner />;
  }
  if (!isAdmin) {
    return <p role="alert">{t("admin.forbidden")}</p>;
  }

  const origin = window.location.origin;
  const roleLabel = (role: string) =>
    role === "admin"
      ? t("admin.roleAdmin")
      : role === "staff"
        ? t("admin.roleStaff")
        : t("admin.roleMember");

  return (
    <div>
      <div className="page-header">
        <h1>{t("admin.title")}</h1>
      </div>

      <div className="card" style={{ marginBottom: "1.5rem" }}>
        <h2>{t("admin.studios")}</h2>
        {orgs.data && orgs.data.length > 0 && (
          <div className="table-wrap">
            <table>
              <thead>
                <tr>
                  <th>{t("studio.name")}</th>
                  <th>{t("studio.timezone")}</th>
                  <th>{t("admin.registerLink")}</th>
                </tr>
              </thead>
              <tbody>
                {orgs.data.map((org) => (
                  <tr key={org.id}>
                    <td>{org.name}</td>
                    <td>{org.timezone}</td>
                    <td>
                      <code className="break-anywhere">
                        {`${origin}/register/${org.public_id}`}
                      </code>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}

        <form
          onSubmit={(e: FormEvent<HTMLFormElement>) => {
            e.preventDefault();
            setOrgError(null);
            createOrg.mutate();
          }}
          style={{ marginTop: "1rem" }}
        >
          <h3>{t("admin.newStudio")}</h3>
          <div>
            <label htmlFor="orgName">{t("studio.name")}</label>
            <input
              id="orgName"
              type="text"
              required
              minLength={2}
              value={orgName}
              onChange={(e) => setOrgName(e.target.value)}
            />
          </div>
          <div>
            <label htmlFor="orgTz">{t("studio.timezone")}</label>
            <input
              id="orgTz"
              type="text"
              required
              value={orgTz}
              onChange={(e) => setOrgTz(e.target.value)}
            />
          </div>
          <button type="submit" disabled={createOrg.isPending}>
            {createOrg.isPending ? t("admin.creating") : t("admin.create")}
          </button>
          {orgError && <p role="alert">{orgError}</p>}
        </form>
      </div>

      <div className="card">
        <h2>{t("admin.addTeacher")}</h2>
        <form
          onSubmit={(e: FormEvent<HTMLFormElement>) => {
            e.preventDefault();
            setStaffError(null);
            setStaffOk(false);
            if (staffOrg === "") {
              setStaffError(t("admin.createFailed"));
              return;
            }
            addTeacher.mutate();
          }}
        >
          <div>
            <label htmlFor="staffOrg">{t("admin.studio")}</label>
            <select
              id="staffOrg"
              required
              value={staffOrg}
              onChange={(e) => setStaffOrg(Number(e.target.value))}
            >
              <option value="" disabled>
                —
              </option>
              {orgs.data?.map((org) => (
                <option key={org.id} value={org.id}>
                  {org.name}
                </option>
              ))}
            </select>
          </div>
          <div>
            <label htmlFor="staffName">{t("auth.name")}</label>
            <input
              id="staffName"
              type="text"
              required
              minLength={2}
              value={staffName}
              onChange={(e) => setStaffName(e.target.value)}
            />
          </div>
          <div>
            <label htmlFor="staffEmail">{t("auth.email")}</label>
            <input
              id="staffEmail"
              type="email"
              required
              value={staffEmail}
              onChange={(e) => setStaffEmail(e.target.value)}
            />
          </div>
          <div>
            <label htmlFor="staffPassword">{t("auth.password")}</label>
            <input
              id="staffPassword"
              type="password"
              required
              value={staffPassword}
              onChange={(e) => setStaffPassword(e.target.value)}
            />
          </div>
          <button type="submit" disabled={addTeacher.isPending}>
            {addTeacher.isPending ? t("admin.creating") : t("admin.create")}
          </button>
          {staffOk && <p className="muted">{t("admin.teacherCreated")}</p>}
          {staffError && <p role="alert">{staffError}</p>}
        </form>
      </div>

      <div className="card" style={{ marginTop: "1.5rem" }}>
        <h2>{t("admin.members")}</h2>
        <div>
          <label htmlFor="usersOrg">{t("admin.studio")}</label>
          <select
            id="usersOrg"
            value={usersOrg}
            onChange={(e) =>
              setUsersOrg(e.target.value === "" ? "" : Number(e.target.value))
            }
          >
            <option value="">{t("admin.selectStudio")}</option>
            {orgs.data?.map((org) => (
              <option key={org.id} value={org.id}>
                {org.name}
              </option>
            ))}
          </select>
        </div>

        {usersOrg !== "" &&
          (members.data && members.data.length > 0 ? (
            <div className="table-wrap" style={{ marginTop: "1rem" }}>
              <table>
              <thead>
                <tr>
                  <th>{t("auth.name")}</th>
                  <th>{t("admin.role")}</th>
                  <th />
                </tr>
              </thead>
              <tbody>
                {members.data.map((u) => (
                  <tr key={u.pid}>
                    <td>
                      {u.name}{" "}
                      {u.is_line && (
                        <span className="badge badge-open">LINE</span>
                      )}
                    </td>
                    <td>{roleLabel(u.role)}</td>
                    <td>
                      {u.role === "member" && (
                        <button
                          type="button"
                          disabled={changeRole.isPending}
                          onClick={() =>
                            changeRole.mutate({ pid: u.pid, role: "staff" })
                          }
                        >
                          {t("admin.makeTeacher")}
                        </button>
                      )}
                      {u.role === "staff" && (
                        <button
                          type="button"
                          disabled={changeRole.isPending}
                          onClick={() =>
                            changeRole.mutate({ pid: u.pid, role: "member" })
                          }
                        >
                          {t("admin.makeStudent")}
                        </button>
                      )}
                    </td>
                  </tr>
                ))}
              </tbody>
              </table>
            </div>
          ) : (
            <p className="muted" style={{ marginTop: "1rem" }}>
              {t("admin.noMembers")}
            </p>
          ))}
      </div>
    </div>
  );
}
