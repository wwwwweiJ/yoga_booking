import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Link } from "react-router";
import { cancelBooking, listBookings } from "../../api/bookings";

function formatStartsAt(iso: string): string {
  return new Date(iso).toLocaleString();
}

export function BookingsList() {
  const queryClient = useQueryClient();

  const { data, isPending, isError, error } = useQuery({
    queryKey: ["bookings"],
    queryFn: () => listBookings(),
  });

  const cancel = useMutation({
    mutationFn: (id: number) => cancelBooking(id),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["bookings"] }),
  });

  if (isPending) {
    return <p>Loading…</p>;
  }

  if (isError) {
    return (
      <p role="alert">
        {error instanceof Error ? error.message : "Failed to load bookings"}
      </p>
    );
  }

  return (
    <div>
      <div className="page-header">
        <h1>My bookings</h1>
      </div>

      {data.items.length === 0 ? (
        <div className="card empty">
          You haven&apos;t booked anything yet. Browse{" "}
          <Link to="/classes">classes</Link>.
        </div>
      ) : (
        <div className="card">
        <table>
          <thead>
            <tr>
              <th>Class</th>
              <th>Instructor</th>
              <th>Starts</th>
              <th aria-label="actions" />
            </tr>
          </thead>
          <tbody>
            {data.items.map((booking) => (
              <tr key={booking.id}>
                <td>{booking.class.title}</td>
                <td>{booking.class.instructor}</td>
                <td>{formatStartsAt(booking.class.starts_at)}</td>
                <td>
                  <button
                    type="button"
                    className="btn-danger"
                    disabled={cancel.isPending}
                    onClick={() => {
                      if (window.confirm(`Cancel "${booking.class.title}"?`)) {
                        cancel.mutate(booking.id);
                      }
                    }}
                  >
                    Cancel
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
        </div>
      )}

      <p className="muted" style={{ marginTop: "0.75rem" }}>
        {data.total_items} total
      </p>
    </div>
  );
}
