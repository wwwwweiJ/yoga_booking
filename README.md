# 瑜安伽 Yuan Yoga

A full-stack, multi-tenant yoga studio booking app built on
[Loco](https://loco.rs) (Rust) with a React frontend, wearing the Yuan Yoga
brand (elegant earth tones — Deep Taupe / Warm Cream / Soft Warm Gray). Studios
publish classes, customize their own public page, and take (mock) payments;
members book a spot.

## Stack

- **Backend:** Loco 1.1 (Axum + Sea-ORM), PostgreSQL, JWT auth, local-disk file
  storage for uploads.
- **Frontend:** React 19 + Vite + TanStack Query + React Router, a plain-CSS
  design system, and dependency-free i18n (English + 繁體中文). Request/response
  types in `frontend/src/bindings/` are generated from the Rust DTOs by
  [ts-rs](https://github.com/Aleph-Alpha/ts-rs) on `cargo test`, so the wire
  contract can't silently drift.

## Roles

Three tiers (`users.role`):

- **member** (student) — the default for everyone who registers. Browses and
  books classes, pays for and cancels their own bookings.
- **staff** (teacher) — manages their studio's classes (create/edit/delete),
  uploads instructor photos, and edits the studio's public page.
- **admin** (operator) — the cross-studio backoffice (`/admin`): create studios
  and mint teacher accounts.

Registration always creates a member; staff/admin are minted out-of-band (the
admin backoffice or the `user:create` task).

## Domain

```
organization (a studio) ── has many ──▶ users (member / staff / admin)
  ├── page   (customizable blocks: hero / about / gallery)
  └── class  (a scheduled, priced, bookable session; optional instructor photo)
        └── booking (a member's seat; payment_status pending → paid)
```

- **Organizations** — `name`, `timezone` (IANA), a non-guessable `public_id`
  token, and a JSON `page`. Created by an operator, never through the public API.
- **Users** — each belongs to exactly **one** studio (`organization_id`, set at
  registration) and cannot cross to another; a different studio means a new
  account.
- **Classes** — belong to a studio; `title`, `instructor`, `starts_at`,
  `duration_minutes`, `capacity`, `price` (0 = free), optional instructor photo.
  The API also returns `spots_left` and a `photo_url`.
- **Bookings** — a `(user, class)` pair. A user can't book the same class twice
  (unique index), can't book one at capacity, and can't book one that has
  already started. A booking starts `pending` and a mock payment flips it to
  `paid`. You only see, pay for, and cancel your own bookings.
- **Studio page** — an ordered list of blocks (`hero` / `about` / `gallery`) a
  teacher arranges; rendered publicly at `/studio/<token>`.

**Tenancy:** every request is scoped to the caller's studio. You only see and
manage your own studio's classes, book only there, and cross-studio data is
indistinguishable from "not found" (404). Class management is staff-only (403
for students).

## API

Everything requires a JWT (`Authorization: Bearer <token>`) except the
`/api/auth/*` flow and the `/api/public/*` endpoints.

| Method | Path | Notes |
| --- | --- | --- |
| `POST` | `/api/auth/register` | join a studio (`{ …, organization_token }`); dup email → 409 |
| `POST` | `/api/auth/login` | returns a JWT |
| `POST` | `/api/auth/line` | **public** — LINE (LIFF) login into a studio (`{ id_token, organization_token }`) |
| `GET` | `/api/auth/current` | the caller (incl. `role`) |
| `GET` | `/api/public/config` | **public** — runtime `{ liff_id }` for the SPA |
| `GET` | `/api/public/organizations/{token}` | **public** — a studio's `{ name }` |
| `GET` | `/api/public/organizations/{token}/page` | **public** — a studio's page (name + blocks) |
| `GET` | `/api/organizations` · `/api/organizations/{id}` | your studio only |
| `GET/POST` | `/api/classes` | list your studio's / create (staff) |
| `GET/PUT/DELETE` | `/api/classes/{id}` | fetch / replace / delete (staff, your studio) |
| `POST` | `/api/classes/{id}/photo` | upload instructor photo (staff, multipart) |
| `GET` | `/api/classes/{id}/photo` | **public** — the image bytes |
| `GET/POST` | `/api/bookings` | my bookings / book a class (`{ class_id }`) |
| `POST` | `/api/bookings/{id}/pay` | mock payment → `paid` |
| `DELETE` | `/api/bookings/{id}` | cancel my booking |
| `GET/PUT` | `/api/studio/page` | my studio's page (staff) |
| `GET/POST` | `/api/admin/organizations` | admin — list / create studios |
| `POST` | `/api/admin/staff` | admin — create a teacher for a studio |
| `GET` | `/api/admin/users` | admin — a studio's users (`?organization_id=`) |
| `POST` | `/api/admin/users/{pid}/role` | admin — set a user's role (member ↔ staff) |

Status codes: `201` create, `204` delete, `400` invalid input / bad reference,
`403` wrong role, `404` absent or another studio's, `409` conflict (dup email,
double booking).

## Getting started

One command brings up the whole stack — Postgres, the API on :5150, and the
Vite frontend on :5173 (Ctrl-C stops the API and frontend; Postgres is left
running):

```sh
./dev.sh
```

Or run the pieces by hand:

```sh
docker compose up -d                 # PostgreSQL on :5432
cargo loco start                     # API on :5150 (migrates on boot)
cd frontend && corepack pnpm dev     # frontend on :5173 (proxies /api)
```

### Bootstrap an operator

Studios and staff are operator-created. Make a studio and an admin to run the
backoffice:

```sh
cargo loco task organization:create name:"Sunrise Yoga" timezone:"Asia/Taipei"
# → prints the studio id and its /register/<token> link
cargo loco task user:create \
  email:you@dev.com name:"Dev" password:secret12 organization_id:1 role:admin
```

Log in as that admin → the **Admin** nav link opens `/admin`, where you create
studios and add teachers (each studio shows its `/register/<token>` link).
Teachers open classes, set prices, upload photos, and arrange their studio page
(`/studio/edit`); students register via a studio's `/register/<token>` link,
book classes, and pay. A studio's public page lives at `/studio/<token>`.

### LINE login (optional)

Students and teachers can sign in with **LINE** instead of email/password: a
studio page shows a "Book with LINE" button (`/liff?studio=<token>`) that runs
the LIFF flow, verifies the LINE id token server-side, and finds-or-creates the
user in that studio (as a `member`). Teachers sign in the same way, then an
admin promotes them under **/admin → Members**. Enable it by setting
`LINE_CHANNEL_ID` + `LINE_LIFF_ID` — see [DEPLOY.md](DEPLOY.md) Step 5 for the
LINE Developers console setup. Unset, the feature is simply hidden.

## Testing

Tests run against a `yoga_booking_test` database — create it once:

```sh
createdb -h localhost -U loco yoga_booking_test   # password: loco
cargo test
```

Model logic lives under `src/models/`, HTTP handlers under `src/controllers/`,
typed JSON DTOs under `src/dtos/`, and tests under `tests/`. See `AGENTS.md` for
the Loco conventions this project follows.
