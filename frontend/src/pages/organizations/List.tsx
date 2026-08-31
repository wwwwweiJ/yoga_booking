import { useQuery } from "@tanstack/react-query";
import { listOrganizations } from "../../api/organizations";

// A user belongs to one studio and can't create or edit it (studios are set up
// by an operator), so this is a read-only view of "my studio".
export function OrganizationsList() {
  const { data, isPending, isError, error } = useQuery({
    queryKey: ["organizations"],
    queryFn: () => listOrganizations(),
  });

  if (isPending) {
    return <p>Loading…</p>;
  }

  if (isError) {
    return (
      <p role="alert">
        {error instanceof Error ? error.message : "Failed to load your studio"}
      </p>
    );
  }

  const studio = data.items[0];

  return (
    <div>
      <div className="page-header">
        <h1>My studio</h1>
      </div>
      {studio ? (
        <div className="card">
          <dl>
            <dt>Name</dt>
            <dd>{studio.name}</dd>
            <dt>Timezone</dt>
            <dd>{studio.timezone}</dd>
          </dl>
        </div>
      ) : (
        <div className="card empty">You are not attached to a studio.</div>
      )}
    </div>
  );
}
