import { useState } from "react";
import type { FormEvent } from "react";
import { useQuery } from "@tanstack/react-query";
import { Link, useNavigate, useParams } from "react-router";
import { ApiClientError, post } from "../api/client";
import { getPublicOrganization } from "../api/public";
import { useI18n } from "../i18n";
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
  const { t } = useI18n();

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
        err instanceof ApiClientError ? err.message : t("auth.register.failed"),
      );
    } finally {
      setIsPending(false);
    }
  }

  if (token === undefined) {
    return <p role="alert">{t("auth.register.invalidLink")}</p>;
  }
  if (studio.isPending) {
    return <p>{t("common.loading")}</p>;
  }
  if (studio.isError) {
    return <p role="alert">{t("auth.register.notFound")}</p>;
  }

  return (
    <div className="auth">
      <div className="card">
      <h1>{t("auth.register.join", { studio: studio.data.name })}</h1>
      <form onSubmit={handleSubmit}>
        <div>
          <label htmlFor="name">{t("auth.name")}</label>
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
          <label htmlFor="email">{t("auth.email")}</label>
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
          <label htmlFor="password">{t("auth.password")}</label>
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
          {isPending
            ? t("auth.register.submitting")
            : t("auth.register.submit")}
        </button>
      </form>
      {error && <p role="alert">{error}</p>}
      <p className="switch">
        {t("auth.register.haveAccount")}{" "}
        <Link to="/login">{t("common.loginLink")}</Link>
      </p>
      </div>
    </div>
  );
}
