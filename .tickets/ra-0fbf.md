---
id: ra-0fbf
status: closed
deps: []
links: []
parent: ra-0eb0
created: 2026-02-04T21:38:37Z
type: task
priority: 2
assignee: Trygve Laugstøl
---

# Decide PostGIS data model

Choose between geography column vs expression index on lat/lon; document
rationale and constraints.

## Notes

**2026-02-05T07:54:47Z**

Chose to keep latitude/longitude columns and use a PostGIS geography expression
index for range queries (no separate geography column).
