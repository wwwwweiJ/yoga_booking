import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Link } from "react-router";
import { ApiClientError } from "../../api/client";
import { createBooking } from "../../api/bookings";
import { deleteClass, listClasses } from "../../api/classes";
import { useI18n } from "../../i18n";

function hasStarted(iso: string): boolean {
  return new Date(iso).getTime() <= Date.now();
}

export function ClassesList() {
  const queryClient = useQueryClient();
  const { t, locale } = useI18n();

  const { data, isPending, isError, error } = useQuery({
    queryKey: ["classes"],
    queryFn: () => listClasses(),
  });

  const remove = useMutation({
    mutationFn: (id: number) => deleteClass(id),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["classes"] }),
  });

  const book = useMutation({
    mutationFn: (classId: number) => createBooking({ class_id: classId }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["bookings"] });
      window.alert(t("classes.booked"));
    },
    onError: (err) => {
      // The API returns 409 (already booked) / 400 (full) with a message.
      window.alert(
        err instanceof ApiClientError ? err.message : t("classes.bookFailed"),
      );
    },
  });

  if (isPending) {
    return <p>{t("common.loading")}</p>;
  }

  if (isError) {
    return (
      <p role="alert">
        {error instanceof Error ? error.message : t("classes.loadFailed")}
      </p>
    );
  }

  return (
    <div>
      <div className="page-header">
        <h1>{t("classes.title")}</h1>
        <Link to="/classes/new" className="btn">
          {t("classes.new")}
        </Link>
      </div>

      {data.items.length === 0 ? (
        <div className="card empty">{t("classes.empty")}</div>
      ) : (
        <div className="card">
          <table>
            <thead>
              <tr>
                <th>{t("classes.col.title")}</th>
                <th>{t("classes.col.instructor")}</th>
                <th>{t("classes.col.starts")}</th>
                <th>{t("classes.col.duration")}</th>
                <th>{t("classes.col.spots")}</th>
                <th aria-label="actions" />
              </tr>
            </thead>
            <tbody>
              {data.items.map((klass) => {
                const started = hasStarted(klass.starts_at);
                const full = klass.spots_left <= 0;
                return (
                  <tr key={klass.id}>
                    <td>
                      <Link to={`/classes/${klass.id}/edit`}>{klass.title}</Link>
                    </td>
                    <td>{klass.instructor}</td>
                    <td>{new Date(klass.starts_at).toLocaleString(locale)}</td>
                    <td>
                      {t("classes.minutes", { count: klass.duration_minutes })}
                    </td>
                    <td>
                      {started ? (
                        <span className="badge badge-muted">
                          {t("classes.badge.started")}
                        </span>
                      ) : full ? (
                        <span className="badge badge-full">
                          {t("classes.badge.full")}
                        </span>
                      ) : (
                        <span className="badge badge-open">
                          {t("classes.badge.left", {
                            left: klass.spots_left,
                            capacity: klass.capacity,
                          })}
                        </span>
                      )}
                    </td>
                    <td>
                      <button
                        type="button"
                        disabled={book.isPending || full || started}
                        onClick={() => book.mutate(klass.id)}
                      >
                        {t("classes.book")}
                      </button>{" "}
                      <button
                        type="button"
                        className="btn-danger"
                        disabled={remove.isPending}
                        onClick={() => {
                          if (
                            window.confirm(
                              t("classes.deleteConfirm", { title: klass.title }),
                            )
                          ) {
                            remove.mutate(klass.id);
                          }
                        }}
                      >
                        {t("classes.delete")}
                      </button>
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      )}

      <p className="muted" style={{ marginTop: "0.75rem" }}>
        {t("common.total", { count: data.total_items })}
      </p>
    </div>
  );
}
