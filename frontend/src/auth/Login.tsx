import { useState } from "react";
import type { FormEvent } from "react";
import { useNavigate } from "react-router";
import { ApiClientError, post } from "../api/client";
import { useI18n } from "../i18n";
import { setToken } from "./token";

interface LoginResponse {
  token: string;
  pid: string;
  name: string;
  is_verified: boolean;
}

export function Login() {
  const navigate = useNavigate();
  const { t } = useI18n();
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [isPending, setIsPending] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function handleSubmit(e: FormEvent<HTMLFormElement>) {
    e.preventDefault();
    setError(null);
    setIsPending(true);
    try {
      const res = await post<LoginResponse>("/api/auth/login", {
        email,
        password,
      });
      setToken(res.token);
      navigate("/");
    } catch (err) {
      setError(
        err instanceof ApiClientError ? err.message : t("auth.login.failed"),
      );
    } finally {
      setIsPending(false);
    }
  }

  return (
    <div className="auth">
      <div className="card">
      <h1>{t("auth.login.title")}</h1>
      <form onSubmit={handleSubmit}>
        <div>
          <label htmlFor="email">{t("auth.email")}</label>
          <input
            id="email"
            name="email"
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
            name="password"
            type="password"
            autoComplete="current-password"
            required
            value={password}
            onChange={(e) => setPassword(e.target.value)}
          />
        </div>
        <button type="submit" disabled={isPending}>
          {isPending ? t("auth.login.submitting") : t("auth.login.submit")}
        </button>
      </form>
      {error && <p role="alert">{error}</p>}
      <p className="switch">{t("auth.login.switch")}</p>
      </div>
    </div>
  );
}
