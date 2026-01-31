---
id: ra-b1d9
status: closed
deps: []
links: []
created: 2026-01-31T17:51:04Z
type: epic
priority: 2
assignee: Trygve Laugstøl
---

# Imrove organization page

On /call-sign/LA4O, implement an organization page.

- It should list all repeaters in an alphabetical order
  - List each repeater with a heading. For each repeater, list each service with
    all service attributes
- There should be a map with all repeaters visible.

Look at what needs to be done and make subtasks as needed.

## Notes

**2026-01-31T18:18:57Z**

Updated organization detail page to list all linked repeaters alphabetically
with full service attributes, added map rendering for all repeater locations,
and refactored service item building for reuse.

**2026-01-31T18:36:01Z**

Implemented organization detail page for /call-sign/:call_sign to show
alphabetized repeaters with full service details and a map covering all linked
repeater sites.
