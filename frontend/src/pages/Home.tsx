import { Link } from "react-router";
import { getToken } from "../auth/token";
import { useI18n } from "../i18n";

export function Home() {
  const isAuthenticated = getToken() !== null;
  const { t } = useI18n();

  return (
    <div className="hero">
      <img src="/logo.png" className="home-logo" alt="瑜安伽 Yuan Yoga" />
      <h1 className="tagline">{t("home.title")}</h1>
      <p>{t("home.subtitle")}</p>
      {isAuthenticated ? (
        <Link className="btn" to="/classes">
          {t("home.browse")}
        </Link>
      ) : (
        <Link className="btn" to="/login">
          {t("nav.login")}
        </Link>
      )}
    </div>
  );
}
