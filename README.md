# Yoga Booking

A small full-stack yoga class booking app built on [Loco](https://loco.rs)
(Rust) with a React frontend. Studios publish classes; members book a spot.

## Stack

- **Backend:** Loco 1.1 (Axum + Sea-ORM), PostgreSQL, JWT auth.
- **Frontend:** React 19 + Vite + TanStack Query + React Router, talking to the
  API through a typed client. The request/response types in
  `frontend/src/bindings/` are generated from the Rust DTOs by
  [ts-rs](https://github.com/Aleph-Alpha/ts-rs) on `cargo test`, so the wire
  contract can't silently drift.

## Domain

```
organization (a studio)  ── has many ──▶ users (members)
  └── class (a scheduled, bookable session; has a capacity)
        └── booking (a member's seat in a class)
```

- **Organizations** — `name`, `timezone` (IANA). Created out-of-band by an
  operator (the `organization:create` task or a seed), not through the API.
- **Users** — each belongs to exactly **one** studio (`organization_id`, set at
  registration) and cannot cross to another; using a different studio means a
  new account.
- **Classes** — belong to a studio; `title`, `instructor`, `starts_at`,
  `duration_minutes`, `capacity`. The API also returns `spots_left`
  (`capacity` minus current bookings). Deleting a studio cascades to its classes.
- **Bookings** — a `(user, class)` pair. A user can't book the same class
  twice (enforced by a unique index), can't book one that's already at capacity,
  and can't book one that has already started. You only see and cancel your own
  bookings.

**Tenancy:** every request is scoped to the caller's studio. You only ever see
and manage your own studio's classes, can only book classes there, and the
organizations endpoint returns just your studio — another studio's data is
indistinguishable from "not found" (404).

## API

All endpoints require a JWT (`Authorization: Bearer <token>`) except the
`/api/auth/*` flow.

All paths below are scoped to the caller's studio, except the one public
endpoint used by the signed-out register page.

| Method | Path | Notes |
| --- | --- | --- |
| `GET` | `/api/public/organizations/{token}` | **public** — a studio's `{name}` by its register token |
| `GET` | `/api/organizations` | your studio (a one-item page) |
| `GET` | `/api/organizations/{id}` | your studio, else 404 |
| `GET/POST` | `/api/classes` | list your studio's / create (studio implicit) |
| `GET/PUT/DELETE` | `/api/classes/{id}` | fetch / replace / delete (your studio only) |
| `GET/POST` | `/api/bookings` | my bookings / book a class (`{ class_id }`) |
| `DELETE` | `/api/bookings/{id}` | cancel my booking |

Registration takes an `organization_id` (the studio to join, which must exist).

Status codes: `201` on create, `204` on delete, `400` on invalid input or a bad
reference, `404` when absent or in another studio, `409` when booking a class
twice.

## Getting started

One command brings up the whole stack — Postgres, the API on :5150, and the
Vite frontend on :5173 (Ctrl-C stops the API and frontend; Postgres is left
running):

```sh
./dev.sh
```

Or run the pieces by hand:

```sh
docker compose up -d          # PostgreSQL on :5432
cargo loco db migrate         # apply migrations (also runs on boot)
cargo loco start              # API on :5150
cd frontend && corepack pnpm dev   # frontend on :5173
```

Studios are operator-created; make one and note its register link:

```sh
cargo loco task organization:create name:"Sunrise Yoga" timezone:"Asia/Taipei"
```

Each studio has its own register page — `/register/<token>`, where the token is
the studio's non-guessable `public_id` (printed by `organization:create`).
Registration (`POST /api/auth/register`) takes that `organization_token`, never
a numeric id, so a member can only join a studio whose link they were given —
editing the URL to another number just 404s. There is no global directory of
studios. Operators can also create members directly with
`cargo loco task user:create ... organization_id:<id>`.

Frontend (separate terminal):

```sh
cd frontend
corepack pnpm install
corepack pnpm dev             # :5173, proxies /api to :5150
```

Register + verify + log in through the UI, then use the nav to manage
Organizations, Classes, and My Bookings.

## Testing

Tests run against a `yoga_booking_test` database — create it once:

```sh
createdb -h localhost -U loco yoga_booking_test   # password: loco
cargo test
```

Model logic lives under `src/models/`, HTTP handlers under
`src/controllers/`, the typed JSON DTOs under `src/dtos/`, and tests under
`tests/`. See `AGENTS.md` for the Loco conventions this project follows.
