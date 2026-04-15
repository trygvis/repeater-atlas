---
id: ra-9a5d
status: closed
deps: []
links: []
created: 2026-04-15T09:17:55Z
type: feature
priority: 2
assignee: Trygve Laugstøl
tags: [ui, map]
---

# Map: show nearest repeaters list in side pane on zoom

When the user zooms to level 10 or above, the side pane should show a list of
the closest repeaters to the map center. The list refreshes on every pan or zoom
while at threshold or above.

## Design

Layout: the existing left pane (#map-left-pane, visually on the right side)
gains a new mode.

Zoom threshold: constant, currently set to 10.

Pane states (replacing the current 'select a repeater' section):

- Below threshold: show a hint, e.g. 'Zoom in to see nearby repeaters.' (exact
  wording to be adjusted later).
- At or above threshold: show a sorted list of up to 20 repeaters closest to the
  map center. Each entry is the call sign linked to /{call_sign}. Sorted
  ascending by distance from center.

Triggering: list is computed and rendered on every `moveend` event (covers both
pan and zoom) when zoom >= threshold. When zoom drops below threshold, switch to
the hint message.

Data: all repeater data is already available client-side in the data array
(lat/lon + call_sign). Distance can be computed with the Haversine formula or
Leaflet's map.distance(). No backend changes needed.

Relationship with repeater details: the closest-repeaters list replaces the
'select a repeater on the map' empty state. The populated repeater-details
section (shown on marker click) is unchanged — it still appears when a marker is
clicked.

## Acceptance Criteria

- [ ] Zooming to level >= 10 shows a list of up to 20 call signs closest to the
      map center, each linked to its detail page.
- [ ] List is sorted by distance ascending.
- [ ] List refreshes when the user pans or zooms (while at threshold).
- [ ] Zooming below level 10 replaces the list with a hint message.
- [ ] Clicking a marker still shows full repeater details in the pane (existing
      behavior unchanged).
- [ ] Pane toggle still works correctly in all states.
- [ ] docs/pages/map.md is updated to reflect the new nearest-repeaters pane
      behavior.

## Notes

**2026-04-15T10:11:15Z**

Implemented in 74ca3ad. All acceptance criteria met. Design doc updated in docs/pages/map.md.
