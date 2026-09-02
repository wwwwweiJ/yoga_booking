import { useEffect, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import type { Block } from "../../bindings/Block";
import { ApiClientError } from "../../api/client";
import { getMyStudioPage, updateStudioPage } from "../../api/studio";
import { useCurrentUser } from "../../auth/useCurrentUser";
import { useI18n } from "../../i18n";

function defaultBlock(type: Block["type"]): Block {
  switch (type) {
    case "hero":
      return { type: "hero", heading: "", subheading: "" };
    case "about":
      return { type: "about", text: "" };
    case "gallery":
      return { type: "gallery", images: [] };
    case "schedule":
      return { type: "schedule", heading: "" };
  }
}

export function StudioEditor() {
  const { t } = useI18n();
  const { isStaff, isPending } = useCurrentUser();
  const queryClient = useQueryClient();

  const page = useQuery({
    queryKey: ["studio-page"],
    queryFn: getMyStudioPage,
    enabled: isStaff,
  });

  const [blocks, setBlocks] = useState<Block[]>([]);
  const [status, setStatus] = useState<string | null>(null);

  useEffect(() => {
    if (page.data) {
      setBlocks(page.data.blocks);
    }
  }, [page.data]);

  const save = useMutation({
    mutationFn: () => updateStudioPage(blocks),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["studio-page"] });
      setStatus(t("studio.saved"));
    },
    onError: (err) =>
      setStatus(
        err instanceof ApiClientError ? err.message : t("studio.saveFailed"),
      ),
  });

  if (isPending || (isStaff && page.isPending)) {
    return <p>{t("common.loading")}</p>;
  }
  if (!isStaff) {
    return <p role="alert">{t("admin.forbidden")}</p>;
  }

  const setBlock = (i: number, block: Block) =>
    setBlocks((bs) => bs.map((b, idx) => (idx === i ? block : b)));
  const move = (i: number, dir: -1 | 1) => {
    const j = i + dir;
    if (j < 0 || j >= blocks.length) return;
    const copy = [...blocks];
    [copy[i], copy[j]] = [copy[j], copy[i]];
    setBlocks(copy);
  };
  const removeBlock = (i: number) =>
    setBlocks((bs) => bs.filter((_, idx) => idx !== i));
  const add = (type: Block["type"]) =>
    setBlocks((bs) => [...bs, defaultBlock(type)]);

  return (
    <div>
      <div className="page-header">
        <h1>{t("studio.pageTitle")}</h1>
        <button
          type="button"
          disabled={save.isPending}
          onClick={() => {
            setStatus(null);
            save.mutate();
          }}
        >
          {save.isPending ? t("studio.saving") : t("studio.save")}
        </button>
      </div>

      {blocks.length === 0 && (
        <p className="muted">{t("studio.noBlocks")}</p>
      )}

      {blocks.map((block, i) => (
        <div className="card" key={i} style={{ marginBottom: "1rem" }}>
          <div className="page-header" style={{ marginBottom: "0.75rem" }}>
            <strong>{t(`studio.block.${block.type}`)}</strong>
            <div style={{ display: "flex", gap: "0.35rem" }}>
              <button
                type="button"
                className="btn-secondary"
                onClick={() => move(i, -1)}
              >
                ↑
              </button>
              <button
                type="button"
                className="btn-secondary"
                onClick={() => move(i, 1)}
              >
                ↓
              </button>
              <button
                type="button"
                className="btn-danger"
                onClick={() => removeBlock(i)}
              >
                {t("studio.remove")}
              </button>
            </div>
          </div>

          {block.type === "hero" && (
            <>
              <div>
                <label>{t("studio.heading")}</label>
                <input
                  value={block.heading}
                  onChange={(e) =>
                    setBlock(i, { ...block, heading: e.target.value })
                  }
                />
              </div>
              <div>
                <label>{t("studio.subheading")}</label>
                <input
                  value={block.subheading}
                  onChange={(e) =>
                    setBlock(i, { ...block, subheading: e.target.value })
                  }
                />
              </div>
            </>
          )}

          {block.type === "about" && (
            <div>
              <label>{t("studio.text")}</label>
              <textarea
                rows={4}
                value={block.text}
                onChange={(e) => setBlock(i, { ...block, text: e.target.value })}
              />
            </div>
          )}

          {block.type === "gallery" && (
            <>
              {block.images.map((img, j) => (
                <div key={j} style={{ display: "flex", gap: "0.5rem" }}>
                  <input
                    placeholder={t("studio.imageUrl")}
                    value={img}
                    onChange={(e) => {
                      const images = [...block.images];
                      images[j] = e.target.value;
                      setBlock(i, { ...block, images });
                    }}
                  />
                  <button
                    type="button"
                    className="btn-danger"
                    onClick={() =>
                      setBlock(i, {
                        ...block,
                        images: block.images.filter((_, k) => k !== j),
                      })
                    }
                  >
                    ×
                  </button>
                </div>
              ))}
              <button
                type="button"
                className="btn-secondary"
                onClick={() =>
                  setBlock(i, { ...block, images: [...block.images, ""] })
                }
              >
                {t("studio.addImage")}
              </button>
            </>
          )}

          {block.type === "schedule" && (
            <>
              <div>
                <label>{t("studio.heading")}</label>
                <input
                  value={block.heading}
                  onChange={(e) =>
                    setBlock(i, { ...block, heading: e.target.value })
                  }
                />
              </div>
              <p className="muted" style={{ margin: 0 }}>
                {t("studio.scheduleHint")}
              </p>
            </>
          )}
        </div>
      ))}

      <div style={{ display: "flex", gap: "0.5rem", marginTop: "1rem" }}>
        <button type="button" className="btn-secondary" onClick={() => add("hero")}>
          {t("studio.addHero")}
        </button>
        <button
          type="button"
          className="btn-secondary"
          onClick={() => add("about")}
        >
          {t("studio.addAbout")}
        </button>
        <button
          type="button"
          className="btn-secondary"
          onClick={() => add("gallery")}
        >
          {t("studio.addGallery")}
        </button>
        <button
          type="button"
          className="btn-secondary"
          onClick={() => add("schedule")}
        >
          {t("studio.addSchedule")}
        </button>
      </div>

      {status && <p className="muted" style={{ marginTop: "1rem" }}>{status}</p>}
    </div>
  );
}
