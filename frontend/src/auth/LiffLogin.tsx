import { useEffect, useRef, useState } from "react";
import { useNavigate, useSearchParams } from "react-router";
import liff from "@line/liff";
import { getPublicConfig } from "../api/config";
import { lineLogin } from "../api/line";
import { useI18n } from "../i18n";
import { setToken } from "./token";

// Where the studio (its public token) is stashed across the LINE login
// redirect. `liff.login()` returns to the bare endpoint URL, so a query param
// alone can be dropped — we persist it and read it back on the way in.
const STUDIO_KEY = "liff_studio";

// The LIFF entry point, opened as `/liff?studio=<public_id>` from a studio's
// LINE link. It initialises LIFF, ensures the visitor is logged in to LINE,
// exchanges their id token for our JWT, and lands them in the app.
export function LiffLogin() {
  const navigate = useNavigate();
  const { t } = useI18n();
  const [params] = useSearchParams();
  const [error, setError] = useState<string | null>(null);
  // React StrictMode runs effects twice in dev, and a stray re-render must not
  // kick off a second login — run the flow exactly once.
  const started = useRef(false);

  useEffect(() => {
    if (started.current) return;
    started.current = true;

    async function run() {
      const studio =
        params.get("studio") ?? window.sessionStorage.getItem(STUDIO_KEY);
      if (!studio) {
        setError(t("auth.register.invalidLink"));
        return;
      }

      let liffId: string;
      try {
        liffId = (await getPublicConfig()).liff_id;
      } catch {
        setError(t("auth.line.failed"));
        return;
      }
      if (!liffId) {
        setError(t("auth.line.notConfigured"));
        return;
      }

      try {
        await liff.init({ liffId });

        if (!liff.isLoggedIn()) {
          // Stash the studio, then hand off to LINE. We come back to this same
          // URL (which also still carries ?studio=) already logged in.
          window.sessionStorage.setItem(STUDIO_KEY, studio);
          liff.login({ redirectUri: window.location.href });
          return;
        }

        const idToken = liff.getIDToken();
        if (!idToken) {
          // No id token means the LIFF app lacks the `openid` scope.
          setError(t("auth.line.notConfigured"));
          return;
        }

        const res = await lineLogin({
          id_token: idToken,
          organization_token: studio,
        });
        setToken(res.token);
        window.sessionStorage.removeItem(STUDIO_KEY);
        navigate("/");
      } catch {
        setError(t("auth.line.failed"));
      }
    }

    void run();
  }, [params, navigate, t]);

  return (
    <div className="auth">
      <div className="card">
        {error ? (
          <p role="alert">{error}</p>
        ) : (
          <p>{t("auth.line.loggingIn")}</p>
        )}
      </div>
    </div>
  );
}
