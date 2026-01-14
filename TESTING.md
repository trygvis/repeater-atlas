# Testing Strategy: NRRL Repeater Atlas

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

## Tooling (Proposed)
- `cargo test` for unit/integration tests.
- Local Postgres pointed to by `TEST_DATABASE_URL`.

## Acceptance Signals
- All tests pass on a clean checkout.
- Migrations succeed on empty database.
- Public pages render without login.
