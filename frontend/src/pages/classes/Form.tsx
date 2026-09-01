import { useEffect, useState } from "react";
import type { FormEvent } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useNavigate, useParams } from "react-router";
import { ApiClientError } from "../../api/client";
import { createClass, getClass, updateClass } from "../../api/classes";
import { useI18n } from "../../i18n";

// The API speaks RFC 3339 (UTC); <input type="datetime-local"> speaks a naive
// local "YYYY-MM-DDTHH:mm". Convert at the edges so the wire format stays the
// contract and the input stays friendly.
function toInputValue(iso: string): string {
  const d = new Date(iso);
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}T${pad(
    d.getHours(),
  )}:${pad(d.getMinutes())}`;
}

export function ClassForm() {
  const { id } = useParams();
  const classId = id === undefined ? null : Number(id);
  const isEdit = classId !== null;

  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const { t } = useI18n();

  // The owning studio is implicit — always the logged-in user's — so it is not
  // part of the form.
  const [title, setTitle] = useState("");
  const [instructor, setInstructor] = useState("");
  const [startsAt, setStartsAt] = useState("");
  const [durationMinutes, setDurationMinutes] = useState(60);
  const [capacity, setCapacity] = useState(20);
  const [error, setError] = useState<string | null>(null);

  const existing = useQuery({
    queryKey: ["classes", classId],
    queryFn: () => getClass(classId as number),
    enabled: isEdit,
  });

  useEffect(() => {
    if (existing.data) {
      setTitle(existing.data.title);
      setInstructor(existing.data.instructor);
      setStartsAt(toInputValue(existing.data.starts_at));
      setDurationMinutes(existing.data.duration_minutes);
      setCapacity(existing.data.capacity);
    }
  }, [existing.data]);

  const save = useMutation({
    mutationFn: () => {
      const startsAtIso = new Date(startsAt).toISOString();
      const body = {
        title,
        instructor,
        starts_at: startsAtIso,
        duration_minutes: durationMinutes,
        capacity,
      };
      return isEdit ? updateClass(classId as number, body) : createClass(body);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["classes"] });
      navigate("/classes");
    },
    onError: (err) => {
      setError(
        err instanceof ApiClientError
          ? err.message
          : t("classes.form.saveFailed"),
      );
    },
  });

  function handleSubmit(e: FormEvent<HTMLFormElement>) {
    e.preventDefault();
    setError(null);
    save.mutate();
  }

  if (isEdit && existing.isPending) {
    return <p>{t("common.loading")}</p>;
  }

  return (
    <div>
      <div className="page-header">
        <h1>
          {isEdit ? t("classes.form.editTitle") : t("classes.form.newTitle")}
        </h1>
      </div>
      <div className="card">
      <form onSubmit={handleSubmit}>
        <div>
          <label htmlFor="title">{t("classes.col.title")}</label>
          <input
            id="title"
            type="text"
            required
            minLength={2}
            value={title}
            onChange={(e) => setTitle(e.target.value)}
          />
        </div>
        <div>
          <label htmlFor="instructor">{t("classes.col.instructor")}</label>
          <input
            id="instructor"
            type="text"
            required
            minLength={2}
            value={instructor}
            onChange={(e) => setInstructor(e.target.value)}
          />
        </div>
        <div>
          <label htmlFor="startsAt">{t("classes.form.startsAt")}</label>
          <input
            id="startsAt"
            type="datetime-local"
            required
            value={startsAt}
            onChange={(e) => setStartsAt(e.target.value)}
          />
        </div>
        <div>
          <label htmlFor="duration">{t("classes.form.duration")}</label>
          <input
            id="duration"
            type="number"
            required
            min={1}
            value={durationMinutes}
            onChange={(e) => setDurationMinutes(Number(e.target.value))}
          />
        </div>
        <div>
          <label htmlFor="capacity">{t("classes.form.capacity")}</label>
          <input
            id="capacity"
            type="number"
            required
            min={1}
            value={capacity}
            onChange={(e) => setCapacity(Number(e.target.value))}
          />
        </div>
        <button type="submit" disabled={save.isPending}>
          {save.isPending ? t("classes.form.saving") : t("classes.form.save")}
        </button>
      </form>
      {error && <p role="alert">{error}</p>}
      </div>
    </div>
  );
}
