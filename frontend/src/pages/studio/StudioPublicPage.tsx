import { useQuery } from "@tanstack/react-query";
import { Link, useParams } from "react-router";
import {
  getPublicStudioClasses,
  getPublicStudioPage,
} from "../../api/studio";
import { useI18n } from "../../i18n";

// The signed-out, shareable studio page — renders the blocks the teacher
// arranged in the editor.
export function StudioPublicPage() {
  const { token } = useParams();
  const { t, locale } = useI18n();

  const q = useQuery({
    queryKey: ["public-studio-page", token],
    queryFn: () => getPublicStudioPage(token as string),
    enabled: token !== undefined,
  });

  // Only actually needed when a schedule block is present, but it's cheap and
  // keeps rendering simple.
  const classes = useQuery({
    queryKey: ["public-studio-classes", token],
    queryFn: () => getPublicStudioClasses(token as string),
    enabled: token !== undefined,
  });

  if (q.isPending) {
    return <p>{t("common.loading")}</p>;
  }
  if (q.isError) {
    return <p role="alert">{t("auth.register.notFound")}</p>;
  }

  const { name, blocks } = q.data;

  return (
    <div>
      <h1>{name}</h1>
      {blocks.length === 0 && <p className="muted">{t("studio.emptyPage")}</p>}
      {blocks.map((block, i) => {
        if (block.type === "hero") {
          return (
            <div className="hero" key={i}>
              <h1>{block.heading}</h1>
              <p>{block.subheading}</p>
            </div>
          );
        }
        if (block.type === "about") {
          return (
            <div className="card" key={i} style={{ marginBottom: "1rem" }}>
              <p style={{ margin: 0, whiteSpace: "pre-wrap" }}>{block.text}</p>
            </div>
          );
        }
        if (block.type === "gallery") {
          return (
            <div className="studio-gallery" key={i}>
              {block.images.map((img, j) => (
                <img key={j} src={img} alt="" />
              ))}
            </div>
          );
        }
        if (block.type === "schedule") {
          const list = classes.data ?? [];
          return (
            <div key={i} style={{ marginBottom: "1.5rem" }}>
              {block.heading && <h2>{block.heading}</h2>}
              {list.length === 0 ? (
                <p className="muted">{t("studio.noUpcoming")}</p>
              ) : (
                <div className="card">
                  <table>
                    <tbody>
                      {list.map((klass, k) => (
                        <tr key={k}>
                          <td>
                            <span className="instructor-cell">
                              {klass.photo_url && (
                                <img
                                  src={klass.photo_url}
                                  alt=""
                                  className="instructor-thumb"
                                />
                              )}
                              <strong>{klass.title}</strong>
                            </span>
                          </td>
                          <td>{klass.instructor}</td>
                          <td>
                            {new Date(klass.starts_at).toLocaleString(locale)}
                          </td>
                          <td>
                            {klass.price === 0 ? t("classes.free") : klass.price}
                          </td>
                          <td>
                            {klass.spots_left <= 0 ? (
                              <span className="badge badge-full">
                                {t("classes.badge.full")}
                              </span>
                            ) : (
                              <span className="badge badge-open">
                                {klass.spots_left}
                              </span>
                            )}
                          </td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                  <p style={{ marginBottom: 0, marginTop: "1rem" }}>
                    <Link className="btn" to={`/register/${token}`}>
                      {t("studio.registerToBook")}
                    </Link>
                  </p>
                </div>
              )}
            </div>
          );
        }
        return null;
      })}
    </div>
  );
}
