import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Link } from "react-router";
import { cancelBooking, listBookings, payBooking } from "../../api/bookings";
import { useI18n } from "../../i18n";

export function BookingsList() {
  const queryClient = useQueryClient();
  const { t, locale } = useI18n();

  const { data, isPending, isError, error } = useQuery({
    queryKey: ["bookings"],
    queryFn: () => listBookings(),
  });

  const cancel = useMutation({
    mutationFn: (id: number) => cancelBooking(id),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["bookings"] }),
  });

  const pay = useMutation({
    mutationFn: (id: number) => payBooking(id),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["bookings"] }),
  });

  if (isPending) {
    return <p>{t("common.loading")}</p>;
  }

  if (isError) {
    return (
      <p role="alert">
        {error instanceof Error ? error.message : t("bookings.loadFailed")}
      </p>
    );
  }

  return (
    <div>
      <div className="page-header">
        <h1>{t("bookings.title")}</h1>
      </div>

      {data.items.length === 0 ? (
        <div className="card empty">
          {t("bookings.empty")} <Link to="/classes">{t("bookings.browse")}</Link>
        </div>
      ) : (
        <div className="card">
        <table>
          <thead>
            <tr>
              <th>{t("bookings.col.class")}</th>
              <th>{t("bookings.col.instructor")}</th>
              <th>{t("bookings.col.starts")}</th>
              <th>{t("bookings.col.payment")}</th>
              <th aria-label="actions" />
            </tr>
          </thead>
          <tbody>
            {data.items.map((booking) => (
              <tr key={booking.id}>
                <td>{booking.class.title}</td>
                <td>{booking.class.instructor}</td>
                <td>
                  {new Date(booking.class.starts_at).toLocaleString(locale)}
                </td>
                <td>
                  {booking.payment_status === "paid" ? (
                    <span className="badge badge-open">
                      {t("bookings.status.paid")}
                    </span>
                  ) : (
                    <span className="badge badge-muted">
                      {t("bookings.status.pending")}
                    </span>
                  )}
                </td>
                <td>
                  {booking.payment_status === "pending" && (
                    <>
                      <button
                        type="button"
                        disabled={pay.isPending}
                        onClick={() => pay.mutate(booking.id)}
                      >
                        {pay.isPending
                          ? t("bookings.paying")
                          : t("bookings.pay")}
                      </button>{" "}
                    </>
                  )}
                  <button
                    type="button"
                    className="btn-danger"
                    disabled={cancel.isPending}
                    onClick={() => {
                      if (
                        window.confirm(
                          t("bookings.cancelConfirm", {
                            title: booking.class.title,
                          }),
                        )
                      ) {
                        cancel.mutate(booking.id);
                      }
                    }}
                  >
                    {t("bookings.cancel")}
                  </button>
                </td>
              </tr>
            ))}
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
