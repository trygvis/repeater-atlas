---
id: ra-3ab5
status: open
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
