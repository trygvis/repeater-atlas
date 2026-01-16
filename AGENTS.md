# Repeater Atlas: Agent Notes

## Quick Context
- Purpose: Public repeater directory and map with club-managed data.
- Users: Public readers; club admins/editors manage repeater entries.
- Deployment: Single-instance service (no horizontal scaling in MVP).
- Data access: Public read; admin editing only on main site domain.
- Club views: CNAME-based read-only views; embeddable minimal layout.

## Stack
- Rust + Axum (SSR) + Askama templates.
- HTMX for progressive enhancement.
- PostgreSQL + Diesel (including migrations).
- Auth: signed/encrypted session cookies; in-memory session store.
- Map: Leaflet + OSM tiles (provider swappable).

## Conventions
- Generate migrations via `diesel migration generate`, not by hand.

## Documentation
These must be read before doing any work.

- `docs/CODE.md`
- `docs/DATA_MODEL.md`
- `docs/DESIGN.md`
- `docs/IMPLEMENTATION.md`
- `docs/TESTING.md`

## Testing Notes
- Access the database via `bin/psql`. It will connect to the correct database automatically.
- Integration tests use a dedicated local Postgres via `DATABASE_URL`.
- Fixture loader can drop/recreate the test DB and run migrations.
- Tests commit changes (no transaction rollback).
