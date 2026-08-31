import { Link } from "react-router";
import { getToken } from "../auth/token";

export function Home() {
  const isAuthenticated = getToken() !== null;

  return (
    <div className="hero">
      <h1>Breathe. Book. Flow.</h1>
      <p>
        Reserve your place in classes at your yoga studio — see what&apos;s on,
        grab a spot, and keep track of your bookings.
      </p>
      {isAuthenticated ? (
        <Link className="btn" to="/classes">
          Browse classes
        </Link>
      ) : (
        <Link className="btn" to="/login">
          Log in
        </Link>
      )}
    </div>
  );
}
