import { useQuery } from "@tanstack/react-query";
import { Link, useParams } from "react-router";
import { getPublicOrganization } from "../../api/public";
import {
  getPublicStudioClasses,
  getPublicStudioPage,
} from "../../api/studio";
import { useI18n } from "../../i18n";
import { Spinner } from "../../components/Spinner";

// Teachers can enter a bare handle or a full URL for their links; normalise
// both to an absolute href.
function instagramHref(v: string): string {
  const s = v.trim();
  return /^https?:\/\//i.test(s)
    ? s
    : `https://instagram.com/${s.replace(/^@/, "")}`;
}
function externalHref(v: string): string {
  const s = v.trim();
  return /^https?:\/\//i.test(s) ? s : `https://${s}`;
}

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

  // Carries the studio's LIFF id — present only if this studio enabled LINE
  // login, which gates the "Book with LINE" button.
  const org = useQuery({
    queryKey: ["public-organization", token],
    queryFn: () => getPublicOrganization(token as string),
    enabled: token !== undefined,
  });

  if (q.isPending) {
    return <Spinner />;
  }
  if (q.isError) {
    return <p role="alert">{t("auth.register.notFound")}</p>;
  }

  const { name, blocks } = q.data;

  return (
    <div>
      <h1>{name}</h1>
      {token && org.data?.liff_id && (
        <p style={{ marginBottom: "1rem" }}>
          <Link className="btn" to={`/liff?studio=${token}`}>
            {t("studio.bookWithLine")}
          </Link>
        </p>
      )}
      {blocks.length === 0 && <p className="muted">{t("studio.emptyPage")}</p>}
      {blocks.map((block, i) => {
        if (block.type === "hero") {
          return block.image ? (
            <div
              className="hero hero-image"
              key={i}
              style={{ backgroundImage: `url(${block.image})` }}
            >
              <div className="hero-overlay">
                <h1>{block.heading}</h1>
                <p>{block.subheading}</p>
              </div>
            </div>
          ) : (
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
                  <div className="table-wrap">
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
                  </div>
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
        if (block.type === "teachers") {
          return (
            <div key={i} style={{ marginBottom: "1.5rem" }}>
              {block.heading && <h2>{block.heading}</h2>}
              <div
                style={{
                  display: "grid",
                  gridTemplateColumns: "repeat(auto-fill, minmax(220px, 1fr))",
                  gap: "1rem",
                }}
              >
                {block.members.map((m, j) => (
                  <div className="card" key={j} style={{ marginBottom: 0 }}>
                    {m.photo && (
                      <img
                        src={m.photo}
                        alt=""
                        style={{
                          width: "100%",
                          height: 200,
                          objectFit: "cover",
                          borderRadius: 8,
                        }}
                      />
                    )}
                    <h3 style={{ marginBottom: 0 }}>{m.name}</h3>
                    {m.title && (
                      <p className="muted" style={{ marginTop: "0.25rem" }}>
                        {m.title}
                      </p>
                    )}
                    {m.bio && (
                      <p style={{ whiteSpace: "pre-wrap" }}>{m.bio}</p>
                    )}
                    {(m.instagram || m.website) && (
                      <p
                        style={{
                          display: "flex",
                          gap: "0.75rem",
                          marginBottom: 0,
                        }}
                      >
                        {m.instagram && (
                          <a
                            href={instagramHref(m.instagram)}
                            target="_blank"
                            rel="noreferrer"
                          >
                            Instagram
                          </a>
                        )}
                        {m.website && (
                          <a
                            href={externalHref(m.website)}
                            target="_blank"
                            rel="noreferrer"
                          >
                            {t("studio.teacher.website")}
                          </a>
                        )}
                      </p>
                    )}
                  </div>
                ))}
              </div>
            </div>
          );
        }
        return null;
      })}
    </div>
  );
}
