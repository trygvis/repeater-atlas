# Repeater Atlas: Agent Notes

## Quick Context

- Purpose: Public repeater directory and map with club-managed data.
- Users: Public readers; club admins/editors manage repeater entries.
- Deployment: Single-instance service (no horizontal scaling in MVP).
- Data access: Public read; admin editing only on the main site domain.
- Club views: CNAME-based read-only views; embeddable minimal layout.

## Stack

- Rust + Axum (SSR) + Askama templates.
- HTMX for progressive enhancement.
- PostgreSQL and Diesel (including migrations).
- Auth: signed/encrypted session cookies; in-memory session store.
- Map: Leaflet + OSM tiles (provider swappable).

## Conventions

- Generate migrations via `diesel migration generate`, not by hand.
- Before closing an issue, include a note on what was done. Summarize code and
  design changes and anything related that was done.
- _NEVER_ commit any code without explicit instructions. Ask if a commit should
  be made, only if everything builds and all tests pass.
- `schema.rs` should never be edited manually. Run the database migrations with
  `make db-init`, it will reinitialize the database and run the migrations.
  Diesel will generate `schema.rs` automatically.

## Documentation

These must be read before doing any work.

- `docs/CODE.md`
- `docs/DATA_MODEL.md`
- `docs/DESIGN.md`
- `docs/IMPLEMENTATION.md`
- `docs/TESTING.md`

## CLI commands used while developing and refactoring

- Read the entire Makefile, look at all targets and comments. Learn the
  top-level targets as commands to run.
- Generate migrations via `diesel migration generate`, not by hand.
  - Requires `diesel_cli` installed
    (`cargo install diesel_cli --no-default-features --features postgres`).
  - Apply the migration with `diesel migration run`, it will automatically
    update `src/schema.rs`.
- Local Postgres pointed to by `DATABASE_URL` in `.env`.
- When testing new features, run these commands in this order:
  - Build the entire codebase: `make all`.
  - Run all unit and integration tests with: `make test`
  - Check that the database migrations work with `make db-init`. This will clean
    the database and run all migrations
  - Load the data with `cargo run --bin load-data`.

## Testing Notes

- Access the database via `bin/psql`. It will connect to the correct database
  automatically.
- Integration tests use a dedicated local Postgres via `DATABASE_URL`.
- Fixture loader can drop/recreate the test DB and run migrations.
- Tests commit changes (no transaction rollback).

# Issue tracking

This project uses a CLI ticket system for task management. Run `tk help` when
you need to use it.
