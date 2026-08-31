import { useState } from "react";
import type { FormEvent } from "react";
import { useQuery } from "@tanstack/react-query";
import { Link, useNavigate, useParams } from "react-router";
import { ApiClientError, post } from "../api/client";
import { getPublicOrganization } from "../api/public";
import { setToken } from "./token";

interface LoginResponse {
  token: string;
  pid: string;
  name: string;
  is_verified: boolean;
}

// Each studio hands its members a link to /register/<token>, where the token is
// a non-guessable UUID. The page confirms which studio you're joining, then
// binds the new account to it via that same token.
export function Register() {
  const { token } = useParams();
  const navigate = useNavigate();

  const studio = useQuery({
    queryKey: ["public-organization", token],
    queryFn: () => getPublicOrganization(token as string),
    enabled: token !== undefined,
  });

  const [name, setName] = useState("");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [isPending, setIsPending] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function handleSubmit(e: FormEvent<HTMLFormElement>) {
    e.preventDefault();
    setError(null);
    setIsPending(true);
    try {
      await post<unknown>("/api/auth/register", {
        name,
        email,
        password,
        organization_token: token,
      });
      // Log straight in on success so the new member lands in the app.
      const login = await post<LoginResponse>("/api/auth/login", {
        email,
        password,
      });
      setToken(login.token);
      navigate("/");
    } catch (err) {
      setError(
        err instanceof ApiClientError ? err.message : "Failed to register",
      );
    } finally {
      setIsPending(false);
    }
  }

  if (token === undefined) {
    return <p role="alert">Invalid studio link.</p>;
  }
  if (studio.isPending) {
    return <p>Loading…</p>;
  }
  if (studio.isError) {
    return (
      <p role="alert">
        This studio could not be found — check the register link your studio
        gave you.
      </p>
    );
  }

  return (
    <div>
      <h1>Join {studio.data.name}</h1>
      <form onSubmit={handleSubmit}>
        <div>
          <label htmlFor="name">Name</label>
          <input
            id="name"
            type="text"
            required
            minLength={2}
            value={name}
            onChange={(e) => setName(e.target.value)}
          />
        </div>
        <div>
          <label htmlFor="email">Email</label>
          <input
            id="email"
            type="email"
            autoComplete="email"
            required
            value={email}
            onChange={(e) => setEmail(e.target.value)}
          />
        </div>
        <div>
          <label htmlFor="password">Password</label>
          <input
            id="password"
            type="password"
            autoComplete="new-password"
            required
            value={password}
            onChange={(e) => setPassword(e.target.value)}
          />
        </div>
        <button type="submit" disabled={isPending}>
          {isPending ? "Creating…" : "Create account"}
        </button>
      </form>
      {error && <p role="alert">{error}</p>}
      <p>
        Already have an account? <Link to="/login">Log in</Link>
      </p>
    </div>
  );
}
