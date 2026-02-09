---
id: ra-b0fb
status: closed
deps: []
links: []
created: 2026-02-09T21:08:57Z
type: epic
priority: 2
assignee: Trygve Laugstøl
---

# Create search feature

Creeate tickets for these steps, make sure that all of them are closed when
committing.

Ask for details if required.

- Create docs/page/map.md with the page design for the Map page.
- Make a design for this new search feature:
  - Add a search field after the login button, inside the same overlay. Show
    search result in a new big modal.
  - The search feature only searches call signs, both organizations and
    repeaters.
  - Document the generic search feature in docs/
- Make sure that the page design is updated afterwards.

## Notes

**2026-02-09T21:21:08Z**

Created and closed sub-tickets for map page spec, search UI design, generic
search docs, and page design index updates.

**2026-02-09T21:40:14Z**

Implemented read-only call sign search (query, endpoint, map UI) via
ra-73ce/ra-e6f3/ra-6363.
