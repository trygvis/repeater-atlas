---
id: ra-4ca0
status: open
deps: []
links: [ra-7c74]
created: 2026-03-02T18:51:40Z
type: task
priority: 2
assignee: Trygve Laugstøl
---

# New feature: Locations ("QTHs")

Allow logged-in users to maintain a personal list of named locations (QTHs).

## Data model

A `user_location` record has:

- Owner (`app_user` reference)
- Address (full street address)
- Lat/long (stored as separate columns)
- Maidenhead locator

The user provides at least one of: address, maidenhead, or lat/long. The system
resolves and fills in all three fields before storing:

- Address → geocode to lat/long, derive maidenhead from lat/long.
- Maidenhead → derive lat/long from center of the square, no address resolution.
- Lat/long → derive maidenhead, no address resolution.

All three fields should always be populated in the database for easy access.

## "My page" section

- A dedicated section lists all the user's locations.
- Each location can be edited in place (update any field; re-resolution is
  triggered on save) or deleted.
- A form in the section allows adding a new location by providing address,
  maidenhead, or lat/long.
- Each location has a "show on map" link that opens the main map centered on
  that location.

## Implementation tasks

- Migration: `user_location` table.
- DAO: insert, update, delete, list by user.
- Resolution logic: reuse existing geocoder and maidenhead utilities.
- "My page": locations section with add/edit/delete and map link.
- Update relevant design documents and page specs.

## Linked

- ra-7c74 [closed] Add a way to go to "my position"
