---
id: ra-d8f8
status: open
deps: []
links: []
created: 2026-02-01T19:49:26Z
type: task
priority: 2
assignee: Trygve Laugstøl
---

# Clean up repeater_system.status

Right not is is a non-null String, but should be an enum. I think only "active"
and "inactive" are the possible values. Any other state can be put as a note on
the system.
