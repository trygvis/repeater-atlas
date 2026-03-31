---
id: ra-3ab5
status: closed
deps: []
links: []
created: 2026-03-30T17:17:03Z
type: task
priority: 2
assignee: Trygve Laugstøl
---

# Refactor integration tests to use the standard Application

Right now the integration tests re-create the Tower application, but there
should only be a single assembly of the application. The code should call the
normal initialization of the application object, and then use oneshot() on that
instead.

## Notes

**2026-03-31T07:06:50Z**

Added `create_router(state: AppState) -> Router` to `src/web/mod.rs` as the
single assembly point for all routes. Updated `src/main.rs` to use it (merged
with static file serving). Updated `tests/web_routes.rs` to call `create_router`
instead of manually constructing partial routers — both test functions now use
the same router as production.
