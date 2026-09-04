import { useI18n } from "../i18n";

// A centered loading spinner, used in place of a page's transient "Loading…"
// text so navigation doesn't flash a line of text.
export function Spinner() {
  const { t } = useI18n();
  return (
    <div className="spinner-wrap" role="status" aria-live="polite">
      <span className="spinner" aria-hidden="true" />
      <span className="sr-only">{t("common.loading")}</span>
    </div>
  );
}
