import { useEffect, useState } from "react";
import type { FormEvent } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Link } from "react-router";
import { listOrganizations } from "../../api/organizations";
import { getStudioLine, updateStudioLine } from "../../api/studio";
import { useCurrentUser } from "../../auth/useCurrentUser";
import { useI18n } from "../../i18n";
import { Spinner } from "../../components/Spinner";

// A user belongs to one studio and can't create or edit it (studios are set up
// by an operator), so this is a read-only view of "my studio" — except that a
// teacher can configure the studio's own LINE login here.
export function OrganizationsList() {
  const { t } = useI18n();
  const { isStaff } = useCurrentUser();
  const queryClient = useQueryClient();
  const { data, isPending, isError, error } = useQuery({
    queryKey: ["organizations"],
    queryFn: () => listOrganizations(),
  });

  // Per-studio LINE settings (teacher-only).
  const line = useQuery({
    queryKey: ["studio-line"],
    queryFn: getStudioLine,
    enabled: isStaff,
  });
  const [liffId, setLiffId] = useState("");
  const [channelId, setChannelId] = useState("");
  const [saved, setSaved] = useState(false);
  useEffect(() => {
    if (line.data) {
      setLiffId(line.data.liff_id);
      setChannelId(line.data.channel_id);
    }
  }, [line.data]);
  const saveLine = useMutation({
    mutationFn: () =>
      updateStudioLine({ liff_id: liffId, channel_id: channelId }),
    onSuccess: () => {
      setSaved(true);
      queryClient.invalidateQueries({ queryKey: ["studio-line"] });
    },
  });

  if (isPending) {
    return <Spinner />;
  }

  if (isError) {
    return (
      <p role="alert">
        {error instanceof Error ? error.message : t("studio.loadFailed")}
      </p>
    );
  }

  const studio = data.items[0];

  return (
    <div>
      <div className="page-header">
        <h1>{t("studio.title")}</h1>
      </div>
      {studio ? (
        <div className="card">
          <dl>
            <dt>{t("studio.name")}</dt>
            <dd>{studio.name}</dd>
            <dt>{t("studio.timezone")}</dt>
            <dd>{studio.timezone}</dd>
          </dl>
          <p style={{ marginBottom: 0, display: "flex", gap: "1rem" }}>
            <Link to={`/studio/${studio.public_id}`}>{t("studio.viewPage")}</Link>
            {isStaff && <Link to="/studio/edit">{t("studio.editPage")}</Link>}
          </p>
        </div>
      ) : (
        <div className="card empty">{t("studio.none")}</div>
      )}

      {isStaff && studio && (
        <div className="card" style={{ marginTop: "1.5rem" }}>
          <h2>{t("studio.line.title")}</h2>
          <p className="muted">{t("studio.line.hint")}</p>
          <form
            onSubmit={(e: FormEvent<HTMLFormElement>) => {
              e.preventDefault();
              setSaved(false);
              saveLine.mutate();
            }}
          >
            <div>
              <label htmlFor="liffId">{t("studio.line.liffId")}</label>
              <input
                id="liffId"
                type="text"
                value={liffId}
                placeholder="1234567890-abcdEFGH"
                onChange={(e) => setLiffId(e.target.value)}
              />
            </div>
            <div>
              <label htmlFor="channelId">{t("studio.line.channelId")}</label>
              <input
                id="channelId"
                type="text"
                value={channelId}
                placeholder="2001234567"
                onChange={(e) => setChannelId(e.target.value)}
              />
            </div>
            <button type="submit" disabled={saveLine.isPending}>
              {saveLine.isPending ? t("studio.saving") : t("studio.line.save")}
            </button>
            {saved && <p className="muted">{t("studio.saved")}</p>}
            {saveLine.isError && (
              <p role="alert">{t("studio.saveFailed")}</p>
            )}
          </form>
        </div>
      )}
    </div>
  );
}
