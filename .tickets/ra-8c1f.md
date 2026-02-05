---
id: ra-8c1f
status: closed
deps: []
links: []
created: 2026-02-05T20:13:51Z
type: bug
priority: 2
assignee: Trygve Laugstøl
---

# Repeater query withing radius doesn't work

The query does not return any repeaters within 50km of Kristiansand for LA4ARR.
If the distance is increased to 50000000 (1000 times bigger), it reaches Fauske.

Something is wrong.

## Notes

**2026-02-05T21:05:51Z**

Derived lat/lon from Maidenhead when geocoding is skipped during import; enriched location now returns optional address/maidenhead/coords and normalizes/derives coordinates from Maidenhead when present. Reloaded DB and verified LA4ARR radius query via psql. Tests: direnv exec . make test.
