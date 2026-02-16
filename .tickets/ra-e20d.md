---
id: ra-e20d
status: open
deps: []
links: []
created: 2026-02-16T21:37:47Z
type: epic
priority: 2
assignee: Trygve Laugstøl
---

# New feature: Favorite repeaters

User story: As a user I want to be able to mark a repeater as a "favorite".

Details:

- A start should always be visible on a repeater page, next to the repeater name
  in the heading.
- The star has two "modes", filled and outlined. The filled mode is used to
  indicate that this is personal favorite repeater, the outlined if it is not a
  favorite repeater (or the user is not logged in).
- When I click the star:
  - if logged in, the repeaters favorite state is toggled (not favorite to
    favorite, or favorite to not favorite)
  - If not logged in, show the login screen.

Tasks:

- Ensure that the issue is clear, coherent and complete. As questions until all
  questions are resolved.
- The schema needs a new table: `favorite_repeater` with a reference to a User
  (`app_user` table) and a Repeater (`repeater_system`).
- Update relevant design documents, including page-specific design documents.
