---
id: ra-bd38
status: closed
deps: []
links: []
created: 2026-02-05T09:30:07Z
type: task
priority: 2
assignee: Trygve Laugstøl
---
# Improve select_with_other_call_sign

This uses two queries, but should be using ideally a single query, or at worst a
union instead.

## Notes

**2026-02-06T07:42:53Z**

Replaced select_with_other_call_sign two-query pattern with a single SQL query using CASE and a join, ordered by other call sign.
