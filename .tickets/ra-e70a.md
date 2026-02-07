---
id: ra-e70a
status: open
deps: []
links: []
created: 2026-02-07T09:13:51Z
type: task
priority: 2
assignee: Trygve Laugstøl
---

# Remove resolve_site_fields

## Notes

**2026-02-07T09:18:10Z**

Removed resolve_site_fields; templates now handle '-' fallbacks. Map/list/detail
views use stored latitude/longitude directly; maidenhead shown as optional
metadata. Docs updated (design + repeater detail). Ran just all, just test
(outside sandbox), just db-init, just db-setup.
