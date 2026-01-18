# Testing Strategy: Repeater Atlas

## Goals
- Catch regressions in SSR pages, filtering, and admin flows.
- Validate database interactions and migrations.
- Keep tests lightweight and easy to run locally.

## Test Types
### Unit Tests (Rust)
- Validation helpers (callsign, coordinates, mode lists).
- Auth/session logic (login, expiration).
- Utility functions (distance sorting, formatting).

### Template Rendering Tests
- Render Askama templates with sample data.
- Assert presence of critical fields (callsign, frequencies, status).
- Verify SEO tags for public pages.

### HTTP Integration Tests
- Spin up Axum app in test mode.
- Test public routes (`/`, `/repeaters`, `/repeaters/:id`).
- Test admin auth flow and permissions.
- Verify HTMX partial responses for filters.

### Database Integration Tests
- Apply Diesel migrations against a dedicated test database.
- Load a large fixture (defined in Rust) once per test run; tests assume it is present.
- Fixture loader can drop and recreate the test database before running migrations.
- Tests commit changes so triggers run; avoid transactional rollbacks.
- CRUD smoke tests for repeater, club, user, membership, changelog.

## Test Data
- Base fixture: sizable dataset covering modes, linked networks, and clubs.
- Tests can create random data derived from the fixture.
- Tests must not modify fixture rows in place.

## Tooling
- After refactoring, always build the entire codebase with `cargo build --all-targets` and fix any problems before continuing.
- `make test` for unit/integration tests.
- Generate migrations via `diesel migration generate`, not by hand.
  - Requires `diesel_cli` installed (`cargo install diesel_cli --no-default-features --features postgres`).
  - Apply the migration with `diesel migration run`, it will automatically update `src/schema.rs`.
- Local Postgres pointed to by `DATABASE_URL` in `.env`.

## Acceptance Signals
- All tests pass on a clean checkout.
- Migrations succeed on empty database.
- Public pages render without login.
