import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Link } from "react-router";
import { ApiClientError } from "../../api/client";
import { createBooking } from "../../api/bookings";
import { deleteClass, listClasses } from "../../api/classes";

function formatStartsAt(iso: string): string {
  return new Date(iso).toLocaleString();
}

function hasStarted(iso: string): boolean {
  return new Date(iso).getTime() <= Date.now();
}

export function ClassesList() {
  const queryClient = useQueryClient();

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
      window.alert("Booked!");
    },
    onError: (err) => {
      // The API returns 409 (already booked) / 400 (full) with a message.
      window.alert(
        err instanceof ApiClientError ? err.message : "Could not book",
      );
    },
  });

  if (isPending) {
    return <p>Loading…</p>;
  }

  if (isError) {
    return (
      <p role="alert">
        {error instanceof Error ? error.message : "Failed to load classes"}
      </p>
    );
  }

  return (
    <div>
      <div className="page-header">
        <h1>Classes</h1>
        <Link to="/classes/new" className="btn">
          New class
        </Link>
      </div>

      {data.items.length === 0 ? (
        <div className="card empty">No classes yet.</div>
      ) : (
        <div className="card">
        <table>
          <thead>
            <tr>
              <th>Title</th>
              <th>Instructor</th>
              <th>Starts</th>
              <th>Duration</th>
              <th>Spots</th>
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
                <td>{formatStartsAt(klass.starts_at)}</td>
                <td>{klass.duration_minutes} min</td>
                <td>
                  {started ? (
                    <span className="badge badge-muted">Started</span>
                  ) : full ? (
                    <span className="badge badge-full">Full</span>
                  ) : (
                    <span className="badge badge-open">
                      {klass.spots_left} / {klass.capacity} left
                    </span>
                  )}
                </td>
                <td>
                  <button
                    type="button"
                    disabled={book.isPending || full || started}
                    onClick={() => book.mutate(klass.id)}
                  >
                    Book
                  </button>{" "}
                  <button
                    type="button"
                    className="btn-danger"
                    disabled={remove.isPending}
                    onClick={() => {
                      if (window.confirm(`Delete "${klass.title}"?`)) {
                        remove.mutate(klass.id);
                      }
                    }}
                  >
                    Delete
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
        {data.total_items} total
      </p>
    </div>
  );
}
