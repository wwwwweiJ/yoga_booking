import { useEffect, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import type { Block } from "../../bindings/Block";
import type { TeacherIntro } from "../../bindings/TeacherIntro";
import { ApiClientError } from "../../api/client";
import {
  getMyStudioPage,
  updateStudioPage,
  uploadStudioImage,
} from "../../api/studio";
import { useCurrentUser } from "../../auth/useCurrentUser";
import { useI18n } from "../../i18n";
import { Spinner } from "../../components/Spinner";

const emptyTeacher = (): TeacherIntro => ({
  name: "",
  photo: null,
  title: "",
  bio: "",
  instagram: "",
  website: "",
});

function defaultBlock(type: Block["type"]): Block {
  switch (type) {
    case "hero":
      return { type: "hero", heading: "", subheading: "", image: null };
    case "about":
      return { type: "about", text: "" };
    case "gallery":
      return { type: "gallery", images: [] };
    case "schedule":
      return { type: "schedule", heading: "" };
    case "teachers":
      return { type: "teachers", heading: "", members: [] };
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
  const [dragIndex, setDragIndex] = useState<number | null>(null);

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
    return <Spinner />;
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
  // Drag `from` to sit at position `to` (used by native drag-and-drop).
  const reorder = (from: number, to: number) => {
    if (from === to) return;
    setBlocks((bs) => {
      const copy = [...bs];
      const [moved] = copy.splice(from, 1);
      copy.splice(to, 0, moved);
      return copy;
    });
  };
  const removeBlock = (i: number) =>
    setBlocks((bs) => bs.filter((_, idx) => idx !== i));
  const add = (type: Block["type"]) =>
    setBlocks((bs) => [...bs, defaultBlock(type)]);

  // Helpers for editing the members of a `teachers` block at index `i`.
  const setMemberField = (i: number, j: number, patch: Partial<TeacherIntro>) =>
    setBlocks((bs) =>
      bs.map((b, idx) =>
        idx === i && b.type === "teachers"
          ? {
              ...b,
              members: b.members.map((m, k) => (k === j ? { ...m, ...patch } : m)),
            }
          : b,
      ),
    );
  const addMember = (i: number) =>
    setBlocks((bs) =>
      bs.map((b, idx) =>
        idx === i && b.type === "teachers"
          ? { ...b, members: [...b.members, emptyTeacher()] }
          : b,
      ),
    );
  const removeMember = (i: number, j: number) =>
    setBlocks((bs) =>
      bs.map((b, idx) =>
        idx === i && b.type === "teachers"
          ? { ...b, members: b.members.filter((_, k) => k !== j) }
          : b,
      ),
    );

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
        <div
          className={`card${dragIndex === i ? " block-dragging" : ""}`}
          key={i}
          style={{ marginBottom: "1rem" }}
          onDragOver={(e) => {
            if (dragIndex !== null) e.preventDefault();
          }}
          onDrop={() => {
            if (dragIndex !== null) reorder(dragIndex, i);
            setDragIndex(null);
          }}
        >
          <div
            className="page-header block-handle"
            style={{ marginBottom: "0.75rem" }}
            draggable
            onDragStart={() => setDragIndex(i)}
            onDragEnd={() => setDragIndex(null)}
          >
            <strong>
              <span className="drag-dots" aria-hidden="true">
                ⠿
              </span>{" "}
              {t(`studio.block.${block.type}`)}
            </strong>
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
              {block.image && (
                <div style={{ margin: "0.5rem 0" }}>
                  <img
                    src={block.image}
                    alt=""
                    className="instructor-photo"
                    style={{ width: "100%", height: 100 }}
                  />
                </div>
              )}
              <div style={{ display: "flex", gap: "0.5rem" }}>
                <label className="btn btn-secondary" style={{ margin: 0 }}>
                  {t("studio.uploadBackground")}
                  <input
                    type="file"
                    accept="image/*"
                    style={{ display: "none" }}
                    onChange={async (e) => {
                      const file = e.target.files?.[0];
                      if (!file) return;
                      try {
                        const { url } = await uploadStudioImage(file);
                        setBlocks((bs) =>
                          bs.map((b, idx) =>
                            idx === i && b.type === "hero"
                              ? { ...b, image: url }
                              : b,
                          ),
                        );
                      } catch {
                        setStatus(t("studio.uploadFailed"));
                      }
                    }}
                  />
                </label>
                {block.image && (
                  <button
                    type="button"
                    className="btn-danger"
                    onClick={() => setBlock(i, { ...block, image: null })}
                  >
                    {t("studio.removeImage")}
                  </button>
                )}
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
              <div
                style={{ display: "flex", gap: "0.5rem", alignItems: "center" }}
              >
                <button
                  type="button"
                  className="btn-secondary"
                  onClick={() =>
                    setBlock(i, { ...block, images: [...block.images, ""] })
                  }
                >
                  {t("studio.addImage")}
                </button>
                <label className="btn btn-secondary" style={{ margin: 0 }}>
                  {t("studio.uploadImage")}
                  <input
                    type="file"
                    accept="image/*"
                    style={{ display: "none" }}
                    onChange={async (e) => {
                      const file = e.target.files?.[0];
                      if (!file) return;
                      try {
                        const { url } = await uploadStudioImage(file);
                        setBlocks((bs) =>
                          bs.map((b, idx) =>
                            idx === i && b.type === "gallery"
                              ? { ...b, images: [...b.images, url] }
                              : b,
                          ),
                        );
                      } catch {
                        setStatus(t("studio.uploadFailed"));
                      }
                    }}
                  />
                </label>
              </div>
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

          {block.type === "teachers" && (
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
              {block.members.map((m, j) => (
                <div key={j} className="card" style={{ marginTop: "0.5rem" }}>
                  <div
                    className="page-header"
                    style={{ marginBottom: "0.5rem" }}
                  >
                    <strong>{m.name || t("studio.teacher.new")}</strong>
                    <button
                      type="button"
                      className="btn-danger"
                      onClick={() => removeMember(i, j)}
                    >
                      {t("studio.remove")}
                    </button>
                  </div>
                  {m.photo && (
                    <div style={{ margin: "0.5rem 0" }}>
                      <img src={m.photo} alt="" className="instructor-photo" />
                    </div>
                  )}
                  <div
                    style={{
                      display: "flex",
                      gap: "0.5rem",
                      marginBottom: "0.5rem",
                    }}
                  >
                    <label className="btn btn-secondary" style={{ margin: 0 }}>
                      {t("studio.teacher.photo")}
                      <input
                        type="file"
                        accept="image/*"
                        style={{ display: "none" }}
                        onChange={async (e) => {
                          const file = e.target.files?.[0];
                          if (!file) return;
                          try {
                            const { url } = await uploadStudioImage(file);
                            setMemberField(i, j, { photo: url });
                          } catch {
                            setStatus(t("studio.uploadFailed"));
                          }
                        }}
                      />
                    </label>
                    {m.photo && (
                      <button
                        type="button"
                        className="btn-danger"
                        onClick={() => setMemberField(i, j, { photo: null })}
                      >
                        {t("studio.removeImage")}
                      </button>
                    )}
                  </div>
                  <div>
                    <label>{t("studio.teacher.name")}</label>
                    <input
                      value={m.name}
                      onChange={(e) =>
                        setMemberField(i, j, { name: e.target.value })
                      }
                    />
                  </div>
                  <div>
                    <label>{t("studio.teacher.title")}</label>
                    <input
                      value={m.title}
                      onChange={(e) =>
                        setMemberField(i, j, { title: e.target.value })
                      }
                    />
                  </div>
                  <div>
                    <label>{t("studio.teacher.bio")}</label>
                    <textarea
                      rows={3}
                      value={m.bio}
                      onChange={(e) =>
                        setMemberField(i, j, { bio: e.target.value })
                      }
                    />
                  </div>
                  <div>
                    <label>{t("studio.teacher.instagram")}</label>
                    <input
                      value={m.instagram}
                      placeholder="@handle"
                      onChange={(e) =>
                        setMemberField(i, j, { instagram: e.target.value })
                      }
                    />
                  </div>
                  <div>
                    <label>{t("studio.teacher.website")}</label>
                    <input
                      value={m.website}
                      placeholder="https://…"
                      onChange={(e) =>
                        setMemberField(i, j, { website: e.target.value })
                      }
                    />
                  </div>
                </div>
              ))}
              <button
                type="button"
                className="btn-secondary"
                style={{ marginTop: "0.5rem" }}
                onClick={() => addMember(i)}
              >
                {t("studio.teacher.add")}
              </button>
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
        <button
          type="button"
          className="btn-secondary"
          onClick={() => add("teachers")}
        >
          {t("studio.addTeachers")}
        </button>
      </div>

      {status && <p className="muted" style={{ marginTop: "1rem" }}>{status}</p>}
    </div>
  );
}
