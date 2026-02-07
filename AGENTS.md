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

- All commands have to be run with `direnv exec .` as a prefix to get a working
  environment.
- Generate migrations via `diesel migration generate`, not by hand.
- Before closing an issue, include a note on what was done. Summarize code and
  design changes and anything related that was done.
- _NEVER_ commit any code without explicit instructions. Ask if a commit should
  be made, only if everything builds and all tests pass.
- `schema.rs` should never be edited manually. Run the database migrations with
  `make db-init`, it will reinitialize the database and run the migrations.
  Diesel will generate `schema.rs` automatically.
- There are pre-commit hooks installed that will be executed automatically. If
  these fail, there is an issue that has to be fixed before continuing. DO NOT
  run with `--no-verify` as a workaround.
- Commit messages:
  - All commit messages should be put into a file called `commit.tmp`, and used
    with the `-F` flag to `git commit`. After putting the message into the file,
    run `bin/prettier commit.tmp` to get it properly formatted.
  - The commit message should be a very short summary of what changed. It SHOULD
    be less than 50 characters, MUST NOT be more than 75. After the first line,
    there should be a blank line, then the rest of the messages.
- Before working, make a note of the current Git commit SHA. Might be useful
  later if asked to review incoming changes.
- When working on bugs:
  - Review the bug to make sure that everything is clear. Have a low bar to ask
    for more details.
  - Make a plan for yourself. It SHOULD include these steps
    - A testing step to run all tests
    - A step to update the documentation under `/docs`
  - When working on an epic, each bug should follow this process.
- Follow these steps when changing Makefile files (`*.md`, including
  `.tickets/*.md`):
  - Run `prettier --write $FILE` to reformat the file after editing. This
    ensures consistent formatting of all Markdown files.
  - The `tk` command will edit Markdown files for you, those should also be run
    through `prettier` after editing via `tk`.
- Follow these steps when changing code:
  1. Run `make all` to make sure that everything compiles. This must pass before
     continuing.
  2. Run `make test` to make sure that all tests pass before continuing.
  3. Check that the database migrations work with `make db-init`. This will
     clean the database and run all migrations
  4. Check that the data loading still works with `make db-setup`. This will do
     `db-init` again, but also load the data.

## Documentation

These must be read before doing any work.

- `docs/CODE.md`
- `docs/DATA_MODEL.md`
- `docs/DESIGN.md`
- `docs/IMPLEMENTATION.md`
- `docs/TESTING.md`

## CLI tools and commands

- Read the entire Makefile, look at all targets and comments. Learn the
  top-level targets as commands to run.
- This project uses a CLI ticket system for task management. Run `tk help` when
  you need to use it.
- Generate migrations via `diesel migration generate`, not by hand.
  - Requires `diesel_cli` installed
    (`cargo install diesel_cli --no-default-features --features postgres`).
  - Apply the migration with `diesel migration run`, it will automatically
    update `src/schema.rs`.

## Testing Notes

- Access the database via `psql`. Use `repeater_atlast` as database and username
  for normal database inspections.
- Integration tests use a dedicated local Postgres via `DATABASE_URL`.
- Fixture loader can drop/recreate the test DB and run migrations.
- Tests commit changes (no transaction rollback).
