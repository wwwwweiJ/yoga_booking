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
      <h1>Classes</h1>
      <p>
        <Link to="/classes/new">New class</Link>
      </p>

      {data.items.length === 0 ? (
        <p>No classes yet.</p>
      ) : (
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
                  {started
                    ? "Started"
                    : full
                      ? "Full"
                      : `${klass.spots_left} / ${klass.capacity} left`}
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
      )}

      <p>{data.total_items} total</p>
    </div>
  );
}
