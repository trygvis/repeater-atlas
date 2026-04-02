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

Allow logged-in users to mark repeaters as favorites.

## Star button on repeater detail page

- A star icon is always visible next to the repeater name in the heading.
- Filled star: repeater is a favorite. Outlined star: not a favorite (or user is
  not logged in).
- Clicking the star:
  - If logged in: toggles the favorite state via HTMX — no full page reload,
    only the star updates.
  - If not logged in: redirects to the login page, then returns to the repeater
    page after login.

## Favorites list on "my page"

- A section on "my page" lists all favorited repeaters.
- Each entry links to the repeater detail page.

## Schema

New table `favorite_repeater`:

- Reference to `app_user`
- Reference to `repeater_system`
- Unique constraint on (user, repeater)

## Implementation tasks

- Add `favorite_repeater` migration.
- DAO: insert, delete, list by user, check existence.
- Repeater detail page: star button with HTMX toggle endpoint.
- "My page": favorites list section.
- Update relevant design documents and page specs.
