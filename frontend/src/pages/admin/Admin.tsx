import { useState } from "react";
import type { FormEvent } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { ApiClientError } from "../../api/client";
import {
  createAdminOrganization,
  createStaff,
  listAdminOrganizations,
} from "../../api/admin";
import { useCurrentUser } from "../../auth/useCurrentUser";
import { useI18n } from "../../i18n";

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

  if (isPending) {
    return <p>{t("common.loading")}</p>;
  }
  if (!isAdmin) {
    return <p role="alert">{t("admin.forbidden")}</p>;
  }

  const origin = window.location.origin;

  return (
    <div>
      <div className="page-header">
        <h1>{t("admin.title")}</h1>
      </div>

      <div className="card" style={{ marginBottom: "1.5rem" }}>
        <h2>{t("admin.studios")}</h2>
        {orgs.data && orgs.data.length > 0 && (
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
                    <code>{`${origin}/register/${org.public_id}`}</code>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
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
    </div>
  );
}
