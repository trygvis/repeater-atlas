---
id: ra-eb76
status: closed
deps: []
links: []
parent: ra-0eb0
created: 2026-02-04T21:38:37Z
type: task
priority: 2
assignee: Trygve Laugstøl
---

# Use PostGIS for range queries

Update DAO/web queries to use ST_DWithin/ST_Distance instead of Rust distance
calculations.

## Notes

**2026-02-05T07:54:49Z**

Switched repeater detail range lookup to ST_DWithin via select_within_radius DAO
query.
