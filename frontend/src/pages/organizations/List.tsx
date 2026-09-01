import { useQuery } from "@tanstack/react-query";
import { listOrganizations } from "../../api/organizations";
import { useI18n } from "../../i18n";

// A user belongs to one studio and can't create or edit it (studios are set up
// by an operator), so this is a read-only view of "my studio".
export function OrganizationsList() {
  const { t } = useI18n();
  const { data, isPending, isError, error } = useQuery({
    queryKey: ["organizations"],
    queryFn: () => listOrganizations(),
  });

  if (isPending) {
    return <p>{t("common.loading")}</p>;
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
        </div>
      ) : (
        <div className="card empty">{t("studio.none")}</div>
      )}
    </div>
  );
}
