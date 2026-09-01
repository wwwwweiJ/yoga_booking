import { useQuery } from "@tanstack/react-query";
import { useParams } from "react-router";
import { getPublicStudioPage } from "../../api/studio";
import { useI18n } from "../../i18n";

// The signed-out, shareable studio page — renders the blocks the teacher
// arranged in the editor.
export function StudioPublicPage() {
  const { token } = useParams();
  const { t } = useI18n();

  const q = useQuery({
    queryKey: ["public-studio-page", token],
    queryFn: () => getPublicStudioPage(token as string),
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
        return null;
      })}
    </div>
  );
}
