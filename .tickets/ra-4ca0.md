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

Allow the user to have a list of "locations". A location is an object that has:

- Point (stored as lat/long)
- Maidenhead
- Address (a full street address)
- An owner (an app_user)

When stored/updated the system should resolve an address to a lat/long and a
maidenhead to a lat/long. Only an address, a maidenhead or a lat/long can be
specified.

Requirements:

- Show the list of locations on "my page"
- Allow navigating from the location on "my page" to the map
