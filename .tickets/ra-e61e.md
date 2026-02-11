---
id: ra-e61e
status: closed
deps: []
links: []
created: 2026-02-09T21:51:39Z
type: task
priority: 2
assignee: Trygve Laugstøl
---

# Design and implement cache for Geocoder

To improve performance, reproducability of the data processing and to be able to
change the results a little bit, implement a cache for the geocode service.

It shall

- Load the cache file (data/geocoder.csv) when creting the Geocoder.
- When geocoding a name, look in the cache first. If found, return it.
- If not, do the normal lookup. Save successful lookups to the in-memory cache
  and file.

## Done

- Implemented CSV-backed cache in the geocoder (load on init, check before
  lookup, append successful lookups) and documented the cache file in the
  implementation notes.
