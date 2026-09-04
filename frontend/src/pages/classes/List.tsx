import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Link } from "react-router";
import { ApiClientError } from "../../api/client";
import { createBooking } from "../../api/bookings";
import type { ClassScope } from "../../api/classes";
import { deleteClass, listClasses } from "../../api/classes";
import { useCurrentUser } from "../../auth/useCurrentUser";
import { useI18n } from "../../i18n";

function hasStarted(iso: string): boolean {
  return new Date(iso).getTime() <= Date.now();
}

export function ClassesList() {
  const queryClient = useQueryClient();
  const { t, locale } = useI18n();
  const { isStaff } = useCurrentUser();
  const [scope, setScope] = useState<ClassScope>("upcoming");

  const { data, isPending, isError, error } = useQuery({
    queryKey: ["classes", scope],
    queryFn: () => listClasses(scope),
  });

  const remove = useMutation({
    mutationFn: (id: number) => deleteClass(id),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["classes"] }),
  });

  const book = useMutation({
    mutationFn: (classId: number) => createBooking({ class_id: classId }),
    onSuccess: (booking) => {
      queryClient.invalidateQueries({ queryKey: ["bookings"] });
      window.alert(
        booking.status === "waitlisted"
          ? t("classes.waitlistedToast")
          : t("classes.booked"),
      );
    },
    onError: (err) => {
      // The API returns 409 (already booked) with a message.
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
        <div style={{ display: "flex", gap: "0.75rem", alignItems: "center" }}>
          <div className="lang-switch">
            {(["upcoming", "all"] as const).map((s) => (
              <button
                key={s}
                type="button"
                className={s === scope ? "is-active" : ""}
                onClick={() => setScope(s)}
              >
                {t(`classes.scope.${s}`)}
              </button>
            ))}
          </div>
          {isStaff && (
            <Link to="/classes/new" className="btn">
              {t("classes.new")}
            </Link>
          )}
        </div>
      </div>

      {data.items.length === 0 ? (
        <div className="card empty">{t("classes.empty")}</div>
      ) : (
        <div className="card table-wrap">
          <table>
            <thead>
              <tr>
                <th>{t("classes.col.title")}</th>
                <th>{t("classes.col.instructor")}</th>
                <th>{t("classes.col.starts")}</th>
                <th>{t("classes.col.duration")}</th>
                <th>{t("classes.col.price")}</th>
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
                      {isStaff ? (
                        <Link to={`/classes/${klass.id}/edit`}>
                          {klass.title}
                        </Link>
                      ) : (
                        klass.title
                      )}
                    </td>
                    <td>
                      <span className="instructor-cell">
                        {klass.photo_url && (
                          <img
                            src={klass.photo_url}
                            alt=""
                            className="instructor-thumb"
                          />
                        )}
                        {klass.instructor}
                      </span>
                    </td>
                    <td>{new Date(klass.starts_at).toLocaleString(locale)}</td>
                    <td>
                      {t("classes.minutes", { count: klass.duration_minutes })}
                    </td>
                    <td>
                      {klass.price === 0 ? t("classes.free") : klass.price}
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
                        disabled={book.isPending || started}
                        onClick={() => book.mutate(klass.id)}
                      >
                        {full ? t("classes.joinWaitlist") : t("classes.book")}
                      </button>{" "}
                      {isStaff && (
                        <button
                          type="button"
                          className="btn-danger"
                          disabled={remove.isPending}
                          onClick={() => {
                            if (
                              window.confirm(
                                t("classes.deleteConfirm", {
                                  title: klass.title,
                                }),
                              )
                            ) {
                              remove.mutate(klass.id);
                            }
                          }}
                        >
                          {t("classes.delete")}
                        </button>
                      )}
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
